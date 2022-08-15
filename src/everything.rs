use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;
use walkdir::WalkDir;

use crate::localization::Localization;
use crate::scope::{Loc, Token};

pub trait FileHandler {
    fn handle_file(&mut self, entry: &FileEntry);
}

#[derive(Debug, Error)]
pub enum FilesError {
    #[error("Could not read CK3 game files at {ck3path}")]
    VanillaUnreadable {
        ck3path: PathBuf,
        source: walkdir::Error,
    },
    #[error("Could not read mod files at {modpath}")]
    ModUnreadable {
        modpath: PathBuf,
        source: walkdir::Error,
    },
}

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

#[derive(Clone, Debug)]
pub struct Everything {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod directory
    mod_root: PathBuf,

    /// The CK3 and mod files in the order the game would load them
    ordered_files: Vec<FileEntry>,

    /// Processed localization files
    localization: Localization,
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
        Loc::for_file(Rc::new(entry.path().to_path_buf()))
    }
}

impl From<&FileEntry> for Token {
    fn from(entry: &FileEntry) -> Self {
        Token::from(Loc::from(entry))
    }
}

impl Everything {
    pub fn new(vanilla_root: PathBuf, mod_root: PathBuf) -> Result<Self, FilesError> {
        let mut files = Vec::new();

        // Abort if whole directories are unreadable, because then we don't have
        // a full map of vanilla's or the mod's contents and might give bad advice.
        Everything::_scan(&vanilla_root, FileKind::VanillaFile, &mut files).map_err(|e| {
            FilesError::VanillaUnreadable {
                ck3path: vanilla_root.clone(),
                source: e,
            }
        })?;
        Everything::_scan(&mod_root, FileKind::ModFile, &mut files).map_err(|e| {
            FilesError::ModUnreadable {
                modpath: mod_root.clone(),
                source: e,
            }
        })?;
        files.sort();

        Ok(Everything {
            vanilla_root,
            mod_root,
            ordered_files: files,
            localization: Localization::default(),
        })
    }

    fn _scan(
        path: &PathBuf,
        kind: FileKind,
        files: &mut Vec<FileEntry>,
    ) -> Result<(), walkdir::Error> {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.depth() == 0 || !entry.file_type().is_file() {
                continue;
            }
            // unwrap is safe here because WalkDir gives us paths with this prefix.
            let inner_path = entry.path().strip_prefix(path).unwrap();
            files.push(FileEntry::new(inner_path.to_path_buf(), kind));
        }
        Ok(())
    }

    pub fn get_files_under<'a>(&'a self, subpath: &'a Path) -> Files<'a> {
        Files {
            iter: self.ordered_files.iter(),
            subpath,
        }
    }

    pub fn load_localizations(&mut self) {
        let subpath = PathBuf::from("localization");
        // TODO: the borrow checker won't let us call get_files_under() here because
        // it sees the whole of self as borrowed.
        let iter = Files {
            iter: self.ordered_files.iter(),
            subpath: &subpath,
        };
        for entry in iter {
            self.localization.handle_file(entry);
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
