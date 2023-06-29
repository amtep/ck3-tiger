use anyhow::Result;
use fnv::FnvHashSet;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use walkdir::WalkDir;

use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::{add_loaded_mod_root, error, warn_abbreviated, warn_header, will_log};
use crate::everything::Everything;
use crate::everything::FilesError;
use crate::modfile::ModFile;
use crate::token::{Loc, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileKind {
    Vanilla,
    LoadedMod(u16), // 0-based indexing
    Mod,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileEntry {
    /// Pathname components below the mod directory or the vanilla game dir
    /// Must not be empty.
    path: PathBuf,
    /// Whether it's a vanilla or mod file
    kind: FileKind,
}

impl FileEntry {
    pub fn new(path: PathBuf, kind: FileKind) -> Self {
        assert!(path.file_name().is_some());
        Self { path, kind }
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Convenience function
    /// Won't panic because `FileEntry` with empty filename is not allowed.
    #[allow(clippy::missing_panics_doc)]
    pub fn filename(&self) -> &OsStr {
        self.path.file_name().unwrap()
    }
}

impl Display for FileEntry {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.path.display())
    }
}

impl From<&FileEntry> for Loc {
    fn from(entry: &FileEntry) -> Self {
        Loc::for_file(Rc::new(entry.path().to_path_buf()), entry.kind)
    }
}

impl From<&FileEntry> for Token {
    fn from(entry: &FileEntry) -> Self {
        Token::from(Loc::from(entry))
    }
}

/// A trait for a submodule that can process files.
pub trait FileHandler {
    /// The `FileHandler` can read settings it needs from the ck3-tiger config.
    fn config(&mut self, _config: &Block) {}

    /// Which files this handler is interested in.
    /// This is a directory prefix of files it wants to handle,
    /// relative to the mod or vanilla root.
    fn subpath(&self) -> PathBuf;

    /// This is called for each matching file in turn, in lexical order.
    /// That's the order in which the CK3 game engine loads them too.
    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path);

    /// This is called after all files have been handled.
    /// The `FileHandler` can generate indexes, perform full-data checks, etc.
    fn finalize(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct LoadedMod {
    /// The `FileKind` to use for file entries from this mod.
    kind: FileKind,

    /// The tag used for this mod in error messages.
    label: String,

    /// The location of this mod in the filesystem.
    root: PathBuf,

    /// A list of directories that should not be read from vanilla or previous mods.
    replace_paths: Vec<PathBuf>,
}

impl LoadedMod {
    fn new_main_mod(root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        Self {
            kind: FileKind::Mod,
            label: "MOD".to_string(),
            root,
            replace_paths,
        }
    }

    fn new(kind: FileKind, label: String, root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        Self {
            kind,
            label,
            root,
            replace_paths,
        }
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

#[derive(Clone, Debug)]
pub struct Fileset {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod being analyzed
    the_mod: LoadedMod,

    /// Other mods to be loaded before `mod`, in order
    pub loaded_mods: Vec<LoadedMod>,

    /// The ck3-tiger config
    config: Option<Block>,

    /// The CK3 and mod files in arbitrary order (will be empty after `finalize`)
    files: Vec<FileEntry>,

    /// The CK3 and mod files in the order the game would load them
    ordered_files: Vec<FileEntry>,

    /// All filenames from ordered_files, for quick lookup
    filenames: FnvHashSet<PathBuf>,

    used: RefCell<FnvHashSet<String>>,
}

impl Fileset {
    pub fn new(vanilla_root: PathBuf, mod_root: PathBuf, replace_paths: Vec<PathBuf>) -> Self {
        Fileset {
            vanilla_root,
            the_mod: LoadedMod::new_main_mod(mod_root, replace_paths),
            loaded_mods: Vec::new(),
            config: None,
            files: Vec::new(),
            ordered_files: Vec::new(),
            filenames: FnvHashSet::default(),
            used: RefCell::new(FnvHashSet::default()),
        }
    }

    pub fn config(&mut self, config: Block) {
        for block in config.get_field_blocks("load_mod") {
            let mod_idx;
            if let Ok(idx) = u16::try_from(self.loaded_mods.len()) {
                mod_idx = idx;
            } else {
                let msg = "too many loaded mods, cannot process more";
                error(block, ErrorKey::Config, msg);
                break;
            }

            let default_label = || format!("MOD{mod_idx}");
            let label = block
                .get_field_value("label")
                .map_or_else(default_label, |t| t.to_string());
            if let Some(path) = block.get_field_value("modfile") {
                let path = PathBuf::from(path.as_str());
                if let Ok(modfile) = ModFile::read(&path) {
                    eprintln!(
                        "Loading secondary mod {label} from: {}{}",
                        modfile.modpath().display(),
                        modfile.display_name_ext()
                    );
                    let kind = FileKind::LoadedMod(mod_idx);
                    let loaded_mod = LoadedMod::new(
                        kind,
                        label.clone(),
                        modfile.modpath().to_path_buf(),
                        modfile.replace_paths(),
                    );
                    add_loaded_mod_root(label, loaded_mod.root.to_path_buf());
                    self.loaded_mods.push(loaded_mod);
                }
            } else {
                let msg = "could not load secondary mod from config; missing `modfile` field";
                error(block, ErrorKey::Config, msg);
            }
        }
        self.config = Some(config);
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
            let inner_dir = inner_path.parent().unwrap_or_else(|| Path::new(""));
            if self.should_replace(inner_dir, kind) {
                continue;
            }
            self.files
                .push(FileEntry::new(inner_path.to_path_buf(), kind));
        }
        Ok(())
    }

    pub fn scan_all(&mut self) -> Result<(), FilesError> {
        self.scan(&self.vanilla_root.clone(), FileKind::Vanilla)
            .map_err(|e| FilesError::VanillaUnreadable {
                path: self.vanilla_root.clone(),
                source: e,
            })?;
        // loaded_mods is cloned here for the borrow checker
        for loaded_mod in &self.loaded_mods.clone() {
            self.scan(loaded_mod.root(), loaded_mod.kind())
                .map_err(|e| FilesError::ModUnreadable {
                    path: loaded_mod.root().to_path_buf(),
                    source: e,
                })?;
        }
        self.scan(&self.the_mod.root().to_path_buf(), FileKind::Mod)
            .map_err(|e| FilesError::ModUnreadable {
                path: self.the_mod.root().to_path_buf(),
                source: e,
            })?;
        Ok(())
    }

    pub fn finalize(&mut self) {
        // This places `Mod` entries after `Vanilla` entries and `LoadedMod` entries between them in order
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

        for entry in &self.ordered_files {
            self.filenames.insert(entry.path.clone());
        }
    }

    pub fn get_files_under<'a>(&'a self, subpath: &'a Path) -> Files<'a> {
        let start = self
            .ordered_files
            .partition_point(|entry| entry.path < subpath);
        Files {
            iter: self.ordered_files.iter().skip(start),
            subpath,
        }
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        match entry.kind {
            FileKind::Vanilla => self.vanilla_root.join(entry.path()),
            FileKind::LoadedMod(idx) => self.loaded_mods[idx as usize].root.join(entry.path()),
            FileKind::Mod => self.the_mod.root.join(entry.path()),
        }
    }

    pub fn handle<H: FileHandler>(&self, handler: &mut H) {
        if let Some(config) = &self.config {
            handler.config(config);
        }
        let subpath = handler.subpath();
        for entry in self.get_files_under(&subpath) {
            handler.handle_file(entry, &self.fullpath(entry));
        }
        handler.finalize();
    }

    pub fn mark_used(&self, file: &str) {
        self.used.borrow_mut().insert(file.to_string());
    }

    pub fn exists(&self, key: &str) -> bool {
        let filepath = PathBuf::from(key);
        self.filenames.contains(&filepath)
    }

    pub fn verify_exists(&self, file: &Token) {
        self.mark_used(file.as_str());
        let filepath = PathBuf::from(file.as_str());
        if !self.filenames.contains(&filepath) {
            let msg = "referenced file does not exist";
            error(file, ErrorKey::MissingFile, msg);
        }
    }

    pub fn verify_exists_crashes(&self, file: &Token) {
        self.mark_used(file.as_str());
        let filepath = PathBuf::from(file.as_str());
        if !self.filenames.contains(&filepath) {
            let msg = "referenced file does not exist";
            error(file, ErrorKey::Crash, msg);
        }
    }

    pub fn verify_exists_implied(&self, file: &str, t: &Token) {
        self.mark_used(file);
        let filepath = PathBuf::from(file);
        if !self.filenames.contains(&filepath) {
            let msg = format!("file {file} does not exist");
            error(t, ErrorKey::MissingFile, &msg);
        }
    }

    pub fn verify_exists_implied_crashes(&self, file: &str, t: &Token) {
        self.mark_used(file);
        let filepath = PathBuf::from(file);
        if !self.filenames.contains(&filepath) {
            let msg = format!("file {file} does not exist");
            error(t, ErrorKey::Crash, &msg);
        }
    }

    pub fn validate(&self, _data: &Everything) {
        // Check the files in directories in common/ to make sure they are in known directories
        let mut warned: Vec<&Path> = Vec::new();
        'outer: for entry in &self.ordered_files {
            if !entry.path.starts_with("common") || !entry.path.to_string_lossy().ends_with(".txt")
            {
                continue;
            }
            let dirname = entry.path.parent().unwrap();
            if warned.contains(&dirname) {
                continue;
            }
            // TODO: check if subdirectories are ok in the different common/ directories
            for valid in COMMON_DIRS {
                if entry.path.starts_with(valid) {
                    continue 'outer;
                }
            }
            if entry.path.starts_with("common/scripted_values") {
                let msg = "file should be in common/script_values/";
                error(entry, ErrorKey::Filename, msg);
            } else if entry.path.starts_with("common/on_actions") {
                let msg = "file should be in common/on_action/";
                error(entry, ErrorKey::Filename, msg);
            } else {
                let msg = "file in unexpected directory";
                error(entry, ErrorKey::Filename, msg);
            }
            warned.push(dirname);
        }
    }

    pub fn check_unused_dds(&self, _data: &Everything) {
        let mut vec = Vec::new();
        for entry in &self.ordered_files {
            // TODO: avoid the to_string here
            let path = entry.path.to_string_lossy().to_string();
            if path.ends_with(".dds")
                && !path.starts_with("gfx/interface/illustrations/loading_screens")
                && !self.used.borrow().contains(&path)
            {
                vec.push(entry);
            }
        }
        let mut printed_header = false;
        for entry in vec {
            if !printed_header && will_log(entry, ErrorKey::UnusedFile) {
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

/// LAST UPDATED VERSION 1.9.1
const COMMON_DIRS: &[&str] = &[
    "common/accolade_icons",
    "common/accolade_names",
    "common/accolade_types",
    "common/achievement_groups.txt", // exception for this file
    "common/achievements",
    "common/activities/activity_locales",
    "common/activities/activity_types",
    "common/activities/guest_invite_rules",
    "common/activities/intents",
    "common/activities/pulse_actions",
    "common/ai_goaltypes",
    "common/ai_war_stances",
    "common/artifacts/blueprints",
    "common/artifacts/feature_groups",
    "common/artifacts/features",
    "common/artifacts/slots",
    "common/artifacts/templates",
    "common/artifacts/types",
    "common/artifacts/visuals",
    "common/bookmark_portraits",
    "common/bookmarks/bookmarks",
    "common/bookmarks/groups",
    "common/buildings",
    "common/casus_belli_groups",
    "common/casus_belli_types",
    "common/character_backgrounds",
    "common/character_interaction_categories",
    "common/character_interactions",
    "common/character_memory_types",
    "common/coat_of_arms/coat_of_arms",
    "common/coat_of_arms/dynamic_definitions",
    "common/coat_of_arms/options",
    "common/coat_of_arms/template_lists",
    "common/combat_effects",
    "common/combat_phase_events",
    "common/console_groups",
    "common/council_positions",
    "common/council_tasks",
    "common/court_amenities",
    "common/courtier_guest_management",
    "common/court_positions/categories",
    "common/court_positions/types",
    "common/court_types",
    "common/culture/aesthetics_bundles",
    "common/culture/creation_names",
    "common/culture/cultures",
    "common/culture/eras",
    "common/culture/innovations",
    "common/culture/name_equivalency",
    "common/culture/name_lists",
    "common/culture/pillars",
    "common/culture/traditions",
    "common/customizable_localization",
    "common/deathreasons",
    "common/decisions",
    "common/defines",
    "common/diarchies/diarchy_mandates",
    "common/diarchies/diarchy_types",
    "common/dna_data",
    "common/dynasties",
    "common/dynasty_house_motto_inserts",
    "common/dynasty_house_mottos",
    "common/dynasty_houses",
    "common/dynasty_legacies",
    "common/dynasty_perks",
    "common/effect_localization",
    "common/ethnicities",
    "common/event_backgrounds",
    "common/event_themes",
    "common/event_transitions",
    "common/factions",
    "common/flavorization",
    "common/focuses",
    "common/game_concepts",
    "common/game_rules",
    "common/genes",
    "common/governments",
    "common/guest_system",
    "common/holdings",
    "common/hook_types",
    "common/important_actions",
    "common/inspirations",
    "common/landed_titles",
    "common/laws",
    "common/lease_contracts",
    "common/lifestyle_perks",
    "common/lifestyles",
    "common/men_at_arms_types",
    "common/messages",
    "common/modifier_definition_formats",
    "common/modifier_icons",
    "common/modifiers",
    "common/named_colors",
    "common/nicknames",
    "common/on_action",
    "common/opinion_modifiers",
    "common/playable_difficulty_infos",
    "common/pool_character_selectors",
    "common/province_terrain",
    "common/religion/doctrines",
    "common/religion/fervor_modifiers",
    "common/religion/holy_sites",
    "common/religion/religion_families",
    "common/religion/religions",
    "common/schemes",
    "common/scripted_animations",
    "common/scripted_character_templates",
    "common/scripted_costs",
    "common/scripted_effects",
    "common/scripted_guis",
    "common/scripted_lists",
    "common/scripted_modifiers",
    "common/scripted_relations",
    "common/scripted_rules",
    "common/scripted_triggers",
    "common/script_values",
    "common/secret_types",
    "common/story_cycles",
    "common/struggle/catalysts",
    "common/struggle/struggles",
    "common/succession_election",
    "common/suggestions",
    "common/terrain_types",
    "common/traits",
    "common/travel/point_of_interest_types",
    "common/travel/travel_options",
    "common/trigger_localization",
    "common/tutorial_lesson_chains",
    "common/tutorial_lessons",
    "common/vassal_contracts",
    "common/vassal_stances",
];
