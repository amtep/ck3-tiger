//! Track all the files (vanilla and mods) that are relevant to the current validation.

use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::string::ToString;
use std::sync::RwLock;

use anyhow::{bail, Result};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::block::Block;
use crate::everything::{Everything, FilesError};
use crate::game::Game;
use crate::helpers::TigerHashSet;
use crate::item::Item;
#[cfg(feature = "vic3")]
use crate::mod_metadata::ModMetadata;
#[cfg(any(feature = "ck3", feature = "imperator"))]
use crate::modfile::ModFile;
use crate::parse::ParserMemory;
use crate::pathtable::{PathTable, PathTableIndex};
use crate::report::{
    add_loaded_dlc_root, add_loaded_mod_root, err, fatal, report, warn_abbreviated, warn_header,
    will_maybe_log, ErrorKey, Severity,
};
use crate::token::Token;

/// Note that ordering of these enum values matters.
/// Files later in the order will override files of the same name before them,
/// and the warnings about duplicates take that into account.
// TODO: verify the relative order of `Clausewitz` and `Jomini`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileKind {
    /// `Internal` is for parsing tiger's own data. The user should not see warnings from this.
    Internal,
    /// `Clausewitz` and `Jomini` are directories bundled with the base game.
    Clausewitz,
    Jomini,
    /// The base game files.
    Vanilla,
    /// Downloadable content present on the user's system.
    Dlc(u8),
    /// Other mods loaded as directed by the config file. 0-based indexing.
    LoadedMod(u8),
    /// The mod under scrutiny. Usually, warnings are not emitted unless they touch `Mod` files.
    Mod,
}

impl FileKind {
    pub fn counts_as_vanilla(&self) -> bool {
        match self {
            FileKind::Clausewitz | FileKind::Jomini | FileKind::Vanilla | FileKind::Dlc(_) => true,
            FileKind::Internal | FileKind::LoadedMod(_) | FileKind::Mod => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    /// Pathname components below the mod directory or the vanilla game dir
    /// Must not be empty.
    path: PathBuf,
    /// Whether it's a vanilla or mod file
    kind: FileKind,
    /// Index into the `PathTable`. Used to initialize `Loc`, which doesn't carry a copy of the pathbuf.
    /// A `FileEntry` might not have this index, because `FileEntry` needs to be usable before the (ordered)
    /// path table is created.
    idx: Option<PathTableIndex>,
    /// The full filesystem path of this entry. Not used for ordering or equality.
    fullpath: PathBuf,
}

impl FileEntry {
    pub fn new(path: PathBuf, kind: FileKind, fullpath: PathBuf) -> Self {
        assert!(path.file_name().is_some());
        Self { path, kind, idx: None, fullpath }
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn fullpath(&self) -> &Path {
        &self.fullpath
    }

    /// Convenience function
    /// Won't panic because `FileEntry` with empty filename is not allowed.
    #[allow(clippy::missing_panics_doc)]
    pub fn filename(&self) -> &OsStr {
        self.path.file_name().unwrap()
    }

    fn store_in_pathtable(&mut self) {
        assert!(self.idx.is_none());
        self.idx = Some(PathTable::store(self.path.clone(), self.fullpath.clone()));
    }

    pub fn path_idx(&self) -> Option<PathTableIndex> {
        self.idx
    }
}

impl Display for FileEntry {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.path.display())
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare idx if available (for speed), otherwise compare the paths.
        let path_ord = if self.idx.is_some() && other.idx.is_some() {
            self.idx.unwrap().cmp(&other.idx.unwrap())
        } else {
            self.path.cmp(&other.path)
        };

        // For same paths, the later [`FileKind`] wins.
        if path_ord == Ordering::Equal {
            self.kind.cmp(&other.kind)
        } else {
            path_ord
        }
    }
}

/// A trait for a submodule that can process files.
pub trait FileHandler<T: Send>: Sync + Send {
    /// The `FileHandler` can read settings it needs from the ck3-tiger config.
    fn config(&mut self, _config: &Block) {}

    /// Which files this handler is interested in.
    /// This is a directory prefix of files it wants to handle,
    /// relative to the mod or vanilla root.
    fn subpath(&self) -> PathBuf;

    /// This is called for each matching file, in arbitrary order.
    /// If a `T` is returned, it will be passed to `handle_file` later.
    /// Since `load_file` is executed multi-threaded while `handle_file`
    /// is single-threaded, try to do the heavy work in this function.
    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<T>;

    /// This is called for each matching file in turn, in lexical order.
    /// That's the order in which the CK3 game engine loads them too.
    fn handle_file(&mut self, entry: &FileEntry, loaded: T);

    /// This is called after all files have been handled.
    /// The `FileHandler` can generate indexes, perform full-data checks, etc.
    fn finalize(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct LoadedMod {
    /// The `FileKind` to use for file entries from this mod.
    kind: FileKind,

    /// The tag used for this mod in error messages.
    #[allow(dead_code)]
    label: String,

    /// The location of this mod in the filesystem.
    root: PathBuf,

    /// A list of directories that should not be read from vanilla or previous mods.
    replace_paths: Vec<PathBuf>,
}

impl LoadedMod {
    fn new_main_mod(root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        Self { kind: FileKind::Mod, label: "MOD".to_string(), root, replace_paths }
    }

    fn new(kind: FileKind, label: String, root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        Self { kind, label, root, replace_paths }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn should_replace(&self, path: &Path) -> bool {
        self.replace_paths.iter().any(|p| p == path)
    }
}

#[derive(Debug)]
pub struct Fileset {
    /// The CK3 game directory.
    vanilla_root: Option<PathBuf>,

    /// Extra CK3 directory loaded before vanilla.
    clausewitz_root: Option<PathBuf>,

    /// Extra CK3 directory loaded before vanilla.
    jomini_root: Option<PathBuf>,

    /// The mod being analyzed.
    the_mod: LoadedMod,

    /// Other mods to be loaded before `mod`, in order.
    pub loaded_mods: Vec<LoadedMod>,

    /// DLC directories to be loaded after vanilla, in order.
    loaded_dlcs: Vec<LoadedMod>,

    /// The ck3-tiger config.
    config: Option<Block>,

    /// The CK3 and mod files in arbitrary order (will be empty after `finalize`).
    files: Vec<FileEntry>,

    /// The CK3 and mod files in the order the game would load them.
    ordered_files: Vec<FileEntry>,

    /// Filename Tokens for the files in `ordered_files`.
    /// Used for [`Fileset::iter_keys()`].
    filename_tokens: Vec<Token>,

    /// All filenames from `ordered_files`, for quick lookup.
    filenames: TigerHashSet<PathBuf>,

    /// All directories that have been looked up, for quick lookup.
    directories: RwLock<TigerHashSet<PathBuf>>,

    /// Filenames that have been looked up during validation. Used to filter the --unused output.
    used: RwLock<TigerHashSet<String>>,
}

impl Fileset {
    pub fn new(vanilla_dir: Option<&Path>, mod_root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        let vanilla_root = vanilla_dir.map(|dir| dir.join("game"));
        let clausewitz_root = vanilla_dir.map(|dir| dir.join("clausewitz"));
        let jomini_root = vanilla_dir.map(|dir| dir.join("jomini"));

        Fileset {
            vanilla_root,
            clausewitz_root,
            jomini_root,
            the_mod: LoadedMod::new_main_mod(mod_root, replace_paths),
            loaded_mods: Vec::new(),
            loaded_dlcs: Vec::new(),
            config: None,
            files: Vec::new(),
            ordered_files: Vec::new(),
            filename_tokens: Vec::new(),
            filenames: TigerHashSet::default(),
            directories: RwLock::new(TigerHashSet::default()),
            used: RwLock::new(TigerHashSet::default()),
        }
    }

    pub fn config(&mut self, config: Block) -> Result<()> {
        for block in config.get_field_blocks("load_mod") {
            let mod_idx;
            if let Ok(idx) = u8::try_from(self.loaded_mods.len()) {
                mod_idx = idx;
            } else {
                bail!("too many loaded mods, cannot process more");
            }

            let default_label = || format!("MOD{mod_idx}");
            let label =
                block.get_field_value("label").map_or_else(default_label, ToString::to_string);
            if Game::is_ck3() || Game::is_imperator() {
                #[cfg(any(feature = "ck3", feature = "imperator"))]
                if let Some(path) = block.get_field_value("modfile") {
                    let path = PathBuf::from(path.as_str());
                    let modfile = ModFile::read(&path)?;
                    eprintln!(
                        "Loading secondary mod {label} from: {}{}",
                        modfile.modpath().display(),
                        modfile
                            .display_name()
                            .map_or_else(String::new, |name| format!(" \"{name}\"")),
                    );
                    let kind = FileKind::LoadedMod(mod_idx);
                    let loaded_mod = LoadedMod::new(
                        kind,
                        label.clone(),
                        modfile.modpath().clone(),
                        modfile.replace_paths(),
                    );
                    add_loaded_mod_root(label);
                    self.loaded_mods.push(loaded_mod);
                } else {
                    bail!("could not load secondary mod from config; missing `modfile` field");
                }
            } else if Game::is_vic3() {
                #[cfg(feature = "vic3")]
                if let Some(path) = block.get_field_value("mod") {
                    let pathdir = PathBuf::from(path.as_str());
                    if let Ok(metadata) = ModMetadata::read(&pathdir) {
                        eprintln!(
                            "Loading secondary mod {label} from: {}{}",
                            pathdir.display(),
                            metadata
                                .display_name()
                                .map_or_else(String::new, |name| format!(" \"{name}\"")),
                        );
                        let kind = FileKind::LoadedMod(mod_idx);
                        let loaded_mod =
                            LoadedMod::new(kind, label.clone(), pathdir, metadata.replace_paths());
                        add_loaded_mod_root(label);
                        self.loaded_mods.push(loaded_mod);
                    } else {
                        bail!("does not look like a mod dir: {}", pathdir.display());
                    }
                } else {
                    bail!("could not load secondary mod from config; missing `mod` field");
                }
            }
        }
        self.config = Some(config);
        Ok(())
    }

    fn should_replace(&self, path: &Path, kind: FileKind) -> bool {
        if kind == FileKind::Mod {
            return false;
        }
        if kind < FileKind::Mod && self.the_mod.should_replace(path) {
            return true;
        }
        for loaded_mod in &self.loaded_mods {
            if kind < loaded_mod.kind && loaded_mod.should_replace(path) {
                return true;
            }
        }
        false
    }

    fn scan(&mut self, path: &Path, kind: FileKind) -> Result<(), walkdir::Error> {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.depth() == 0 || !entry.file_type().is_file() {
                continue;
            }
            // unwrap is safe here because WalkDir gives us paths with this prefix.
            let inner_path = entry.path().strip_prefix(path).unwrap();
            if inner_path.starts_with(".git") {
                continue;
            }
            let inner_dir = inner_path.parent().unwrap_or_else(|| Path::new(""));
            if self.should_replace(inner_dir, kind) {
                continue;
            }
            self.files.push(FileEntry::new(
                inner_path.to_path_buf(),
                kind,
                entry.path().to_path_buf(),
            ));
        }
        Ok(())
    }

    pub fn scan_all(&mut self) -> Result<(), FilesError> {
        if let Some(clausewitz_root) = self.clausewitz_root.clone() {
            self.scan(&clausewitz_root.clone(), FileKind::Clausewitz).map_err(|e| {
                FilesError::VanillaUnreadable { path: clausewitz_root.clone(), source: e }
            })?;
        }
        if let Some(jomini_root) = &self.jomini_root.clone() {
            self.scan(&jomini_root.clone(), FileKind::Jomini).map_err(|e| {
                FilesError::VanillaUnreadable { path: jomini_root.clone(), source: e }
            })?;
        }
        if let Some(vanilla_root) = &self.vanilla_root.clone() {
            self.scan(&vanilla_root.clone(), FileKind::Vanilla).map_err(|e| {
                FilesError::VanillaUnreadable { path: vanilla_root.clone(), source: e }
            })?;
            let dlc_root = vanilla_root.join("dlc");
            for entry in
                WalkDir::new(dlc_root).max_depth(1).sort_by_file_name().into_iter().flatten()
            {
                if entry.depth() == 1 && entry.file_type().is_dir() {
                    let label = entry.file_name().to_string_lossy().to_string();
                    let idx =
                        u8::try_from(self.loaded_dlcs.len()).expect("more than 256 DLCs installed");
                    let dlc = LoadedMod::new(
                        FileKind::Dlc(idx),
                        label.clone(),
                        entry.path().to_path_buf(),
                        Vec::new(),
                    );
                    self.scan(dlc.root(), dlc.kind()).map_err(|e| {
                        FilesError::VanillaUnreadable { path: dlc.root().to_path_buf(), source: e }
                    })?;
                    self.loaded_dlcs.push(dlc);
                    add_loaded_dlc_root(label);
                }
            }
        }
        // loaded_mods is cloned here for the borrow checker
        for loaded_mod in &self.loaded_mods.clone() {
            self.scan(loaded_mod.root(), loaded_mod.kind()).map_err(|e| {
                FilesError::ModUnreadable { path: loaded_mod.root().to_path_buf(), source: e }
            })?;
        }
        #[allow(clippy::unnecessary_to_owned)] // borrow checker requires to_path_buf here
        self.scan(&self.the_mod.root().to_path_buf(), FileKind::Mod).map_err(|e| {
            FilesError::ModUnreadable { path: self.the_mod.root().to_path_buf(), source: e }
        })?;
        Ok(())
    }

    pub fn finalize(&mut self) {
        // This sorts by pathname but where pathnames are equal it places `Mod` entries after `Vanilla` entries
        // and `LoadedMod` entries between them in order
        self.files.sort();

        // When there are identical paths, only keep the last entry of them.
        for entry in self.files.drain(..) {
            if let Some(prev) = self.ordered_files.last_mut() {
                if entry.path == prev.path {
                    *prev = entry;
                } else {
                    self.ordered_files.push(entry);
                }
            } else {
                self.ordered_files.push(entry);
            }
        }

        for entry in &mut self.ordered_files {
            let token = Token::new(&entry.filename().to_string_lossy(), (&*entry).into());
            self.filename_tokens.push(token);
            entry.store_in_pathtable();
            self.filenames.insert(entry.path.clone());
        }
    }

    pub fn get_files_under<'a>(&'a self, subpath: &'a Path) -> &'a [FileEntry] {
        let start = self.ordered_files.partition_point(|entry| entry.path < subpath);
        let end = start
            + self.ordered_files[start..].partition_point(|entry| entry.path.starts_with(subpath));
        &self.ordered_files[start..end]
    }

    pub fn filter_map_under<F, T>(&self, subpath: &Path, f: F) -> Vec<T>
    where
        F: Fn(&FileEntry) -> Option<T> + Sync + Send,
        T: Send,
    {
        self.get_files_under(subpath).par_iter().filter_map(f).collect()
    }

    pub fn handle<T: Send, H: FileHandler<T>>(&self, handler: &mut H, parser: &ParserMemory) {
        if let Some(config) = &self.config {
            handler.config(config);
        }
        let subpath = handler.subpath();
        let entries = self.filter_map_under(&subpath, |entry| {
            handler.load_file(entry, parser).map(|loaded| (entry.clone(), loaded))
        });
        for (entry, loaded) in entries {
            handler.handle_file(&entry, loaded);
        }
        handler.finalize();
    }

    pub fn mark_used(&self, file: &str) {
        let file = file.strip_prefix('/').unwrap_or(file);
        self.used.write().unwrap().insert(file.to_string());
    }

    pub fn exists(&self, key: &str) -> bool {
        let key = key.strip_prefix('/').unwrap_or(key);
        let filepath = PathBuf::from(key);
        self.filenames.contains(&filepath)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.filename_tokens.iter()
    }

    pub fn entry_exists(&self, key: &str) -> bool {
        // file exists
        if self.exists(key) {
            return true;
        }

        // directory lookup - check if there are any files within the directory
        let dir = key.strip_prefix('/').unwrap_or(key);
        let dirpath = Path::new(dir);

        if self.directories.read().unwrap().contains(dirpath) {
            return true;
        }

        match self.ordered_files.binary_search_by_key(&dirpath, |fe| fe.path.as_path()) {
            // should be handled in `exists` already; something must be wrong
            Ok(_) => unreachable!(),
            Err(idx) => {
                // there exists a file in the given directory
                if self.ordered_files[idx].path.starts_with(dirpath) {
                    self.directories.write().unwrap().insert(dirpath.to_path_buf());
                    return true;
                }
            }
        }
        false
    }

    pub fn verify_entry_exists(&self, entry: &str, token: &Token, max_sev: Severity) {
        self.mark_used(&entry.replace("//", "/"));
        if !self.entry_exists(entry) {
            let msg = format!("file or directory {entry} does not exist");
            report(ErrorKey::MissingFile, Item::File.severity().at_most(max_sev))
                .msg(msg)
                .loc(token)
                .push();
        }
    }

    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub fn verify_exists(&self, file: &Token) {
        self.mark_used(&file.as_str().replace("//", "/"));
        if !self.exists(file.as_str()) {
            let msg = "referenced file does not exist";
            report(ErrorKey::MissingFile, Item::File.severity()).msg(msg).loc(file).push();
        }
    }

    pub fn verify_exists_implied(&self, file: &str, t: &Token, max_sev: Severity) {
        self.mark_used(&file.replace("//", "/"));
        if !self.exists(file) {
            let msg = format!("file {file} does not exist");
            report(ErrorKey::MissingFile, Item::File.severity().at_most(max_sev))
                .msg(msg)
                .loc(t)
                .push();
        }
    }

    pub fn verify_exists_implied_crashes(&self, file: &str, t: &Token) {
        self.mark_used(&file.replace("//", "/"));
        if !self.exists(file) {
            let msg = format!("file {file} does not exist");
            fatal(ErrorKey::Crash).msg(msg).loc(t).push();
        }
    }

    pub fn validate(&self, _data: &Everything) {
        let common_dirs = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::tables::misc::COMMON_DIRS,
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::tables::misc::COMMON_DIRS,
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::tables::misc::COMMON_DIRS,
        };
        let common_subdirs_ok = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::tables::misc::COMMON_SUBDIRS_OK,
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::tables::misc::COMMON_SUBDIRS_OK,
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::tables::misc::COMMON_SUBDIRS_OK,
        };
        // Check the files in directories in common/ to make sure they are in known directories
        let mut warned: Vec<&Path> = Vec::new();
        'outer: for entry in &self.ordered_files {
            if !entry.path.to_string_lossy().ends_with(".txt") {
                continue;
            }
            if entry.path == PathBuf::from("common/achievement_groups.txt") {
                continue;
            }
            let dirname = entry.path.parent().unwrap();
            if warned.contains(&dirname) {
                continue;
            }
            if !entry.path.starts_with("common") {
                // Check if the modder forgot the common/ part
                let joined = Path::new("common").join(&entry.path);
                for valid in common_dirs {
                    if joined.starts_with(valid) {
                        let msg = format!("file in unexpected directory {}", dirname.display());
                        let info = format!("did you mean common/{} ?", dirname.display());
                        err(ErrorKey::Filename).msg(msg).info(info).loc(entry).push();
                        warned.push(dirname);
                        continue 'outer;
                    }
                }
                continue;
            }

            for valid in common_subdirs_ok {
                if entry.path.starts_with(valid) {
                    continue 'outer;
                }
            }

            for valid in common_dirs {
                if <&str as AsRef<Path>>::as_ref(valid) == dirname {
                    continue 'outer;
                }
            }

            if entry.path.starts_with("common/scripted_values") {
                let msg = "file should be in common/script_values/";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else if (Game::is_ck3() || Game::is_imperator())
                && entry.path.starts_with("common/on_actions")
            {
                let msg = "file should be in common/on_action/";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else if Game::is_vic3() && entry.path.starts_with("common/on_action") {
                let msg = "file should be in common/on_actions/";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else if Game::is_vic3() && entry.path.starts_with("common/modifiers") {
                let msg = "file should be in common/static_modifiers since 1.7";
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            } else {
                let msg = format!("file in unexpected directory `{}`", dirname.display());
                err(ErrorKey::Filename).msg(msg).loc(entry).push();
            }
            warned.push(dirname);
        }
    }

    pub fn check_unused_dds(&self, _data: &Everything) {
        let mut vec = Vec::new();
        for entry in &self.ordered_files {
            let pathname = entry.path.to_string_lossy();
            if entry.path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("dds"))
                && !entry.path.starts_with("gfx/interface/illustrations/loading_screens")
                && !self.used.read().unwrap().contains(pathname.as_ref())
            {
                vec.push(entry);
            }
        }
        let mut printed_header = false;
        for entry in vec {
            if !printed_header && will_maybe_log(entry, ErrorKey::UnusedFile) {
                warn_header(ErrorKey::UnusedFile, "Unused DDS files:\n");
                printed_header = true;
            }
            warn_abbreviated(entry, ErrorKey::UnusedFile);
        }
        if printed_header {
            warn_header(ErrorKey::UnusedFile, "");
        }
    }
}

#[derive(Clone, Debug)]
pub struct Files<'a> {
    iter: std::iter::Skip<std::slice::Iter<'a, FileEntry>>,
    subpath: &'a Path,
}

impl<'a> Iterator for Files<'a> {
    type Item = &'a FileEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.iter.next() {
            if entry.path.starts_with(self.subpath) {
                return Some(entry);
            }
        }
        None
    }
}
