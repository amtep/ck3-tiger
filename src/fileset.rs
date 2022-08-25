use anyhow::Result;
use fnv::FnvHashSet;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use walkdir::WalkDir;

use crate::block::{Loc, Token};
use crate::errorkey::ErrorKey;
use crate::errors::error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileKind {
    VanillaFile,
    ModFile,
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
    fn new(path: PathBuf, kind: FileKind) -> Self {
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

#[derive(Clone, Debug)]
pub struct Fileset {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod directory
    mod_root: PathBuf,

    /// The CK3 and mod files in arbitrary order (will be empty after `finalize`)
    files: Vec<FileEntry>,

    /// The CK3 and mod files in the order the game would load them
    ordered_files: Vec<FileEntry>,

    /// All filenames from ordered_files, for quick lookup
    filenames: FnvHashSet<PathBuf>,
}

impl Fileset {
    pub fn new(vanilla_root: PathBuf, mod_root: PathBuf) -> Self {
        Fileset {
            vanilla_root,
            mod_root,
            files: Vec::new(),
            ordered_files: Vec::new(),
            filenames: FnvHashSet::default(),
        }
    }

    pub fn scan(&mut self, path: &Path, kind: FileKind) -> Result<(), walkdir::Error> {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.depth() == 0 || !entry.file_type().is_file() {
                continue;
            }
            // unwrap is safe here because WalkDir gives us paths with this prefix.
            let inner_path = entry.path().strip_prefix(path).unwrap();
            self.files
                .push(FileEntry::new(inner_path.to_path_buf(), kind));
        }
        Ok(())
    }

    pub fn finalize(&mut self) {
        // This places `ModFile` after `VanillaFile`
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
        Files {
            iter: self.ordered_files.iter(),
            subpath,
        }
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        match entry.kind {
            FileKind::VanillaFile => self.vanilla_root.join(entry.path()),
            FileKind::ModFile => self.mod_root.join(entry.path()),
        }
    }

    pub fn verify_have_file(&self, file: &Token) {
        let filepath = PathBuf::from(file.as_str());
        if !self.filenames.contains(&filepath) {
            error(
                file,
                ErrorKey::MissingFile,
                "referenced file does not exist",
            );
        }
    }

    pub fn verify_have_implied_file(&self, file: &str, t: &Token) {
        let filepath = PathBuf::from(file);
        if !self.filenames.contains(&filepath) {
            error(
                t,
                ErrorKey::MissingFile,
                &format!("file {} does not exist", file),
            );
        }
    }
}

#[derive(Clone, Debug)]
pub struct Files<'a> {
    iter: std::slice::Iter<'a, FileEntry>,
    subpath: &'a Path,
}

impl<'a> Iterator for Files<'a> {
    type Item = &'a FileEntry;

    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.iter.by_ref() {
            if entry.path.starts_with(self.subpath) {
                return Some(entry);
            }
        }
        None
    }
}
