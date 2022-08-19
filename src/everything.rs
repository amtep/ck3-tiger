use anyhow::Result;
use itertools::Itertools;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;
use walkdir::WalkDir;

use crate::errors::{pause_logging, resume_logging};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::scope::{Loc, Scope, Token};

#[derive(Debug, Error)]
pub enum FilesError {
    #[error("Could not read CK3 game files at {path}")]
    VanillaUnreadable {
        path: PathBuf,
        source: walkdir::Error,
    },
    #[error("Could not read mod files at {path}")]
    ModUnreadable {
        path: PathBuf,
        source: walkdir::Error,
    },
    #[error("Could not read config file at {path}")]
    ConfigUnreadable {
        path: PathBuf,
        source: anyhow::Error,
    },
}

/// A trait for a submodule that can process files.
pub trait FileHandler {
    /// The `FileHandler` can read settings it needs from the mod-validator config.
    fn config(&mut self, config: &Scope);

    /// Which files this handler is interested in.
    /// This is a directory prefix of files it wants to handle,
    /// relative to the mod or vanilla root.
    fn subpath(&self) -> PathBuf;

    /// This is called for each matching file in turn, in lexical order.
    /// That's the order in which the CK3 game engine loads them too.
    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path);

    /// This is called after all files have been handled.
    /// The `FileHandler` can generate indexes, perform full-data checks, etc.
    fn finalize(&mut self);
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
pub struct Everything {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod directory
    mod_root: PathBuf,

    /// Config from file
    config: Scope,

    /// The CK3 and mod files in the order the game would load them
    ordered_files: Vec<FileEntry>,

    /// Processed localization files
    localization: Localization,
}

impl Everything {
    pub fn new(vanilla_root: PathBuf, mod_root: PathBuf) -> Result<Self, FilesError> {
        let mut files = Vec::new();

        // Abort if whole directories are unreadable, because then we don't have
        // a full map of vanilla's or the mod's contents and might give bad advice.
        Everything::_scan(&vanilla_root, FileKind::VanillaFile, &mut files).map_err(|e| {
            FilesError::VanillaUnreadable {
                path: vanilla_root.clone(),
                source: e,
            }
        })?;
        Everything::_scan(&mod_root, FileKind::ModFile, &mut files).map_err(|e| {
            FilesError::ModUnreadable {
                path: mod_root.clone(),
                source: e,
            }
        })?;
        files.sort();
        let mut files_filtered = Vec::new();
        // When there are identical paths, only keep the last entry of them.
        // TODO: this does a lot of cloning
        files.iter().circular_tuple_windows().for_each(|(e1, e2)| {
            if e1.path != e2.path {
                files_filtered.push(e1.clone());
            }
        });

        let config_file = mod_root.join("mod-validator.conf");
        let config = if config_file.is_file() {
            Self::_read_config(&config_file).map_err(|e| FilesError::ConfigUnreadable {
                path: config_file,
                source: e,
            })?
        } else {
            Scope::new(Loc::for_file(Rc::new(config_file), FileKind::ModFile))
        };

        Ok(Everything {
            vanilla_root,
            mod_root,
            ordered_files: files_filtered,
            localization: Localization::default(),
            config,
        })
    }

    fn _read_config(path: &Path) -> Result<Scope> {
        PdxFile::read(path, FileKind::ModFile)
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

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        match entry.kind {
            FileKind::VanillaFile => self.vanilla_root.join(entry.path()),
            FileKind::ModFile => self.mod_root.join(entry.path()),
        }
    }

    pub fn load_localizations(&mut self) {
        self.localization.config(&self.config);
        let subpath = self.localization.subpath();
        // TODO: the borrow checker won't let us call get_files_under() here because
        // it sees the whole of self as borrowed.
        let iter = Files {
            iter: self.ordered_files.iter(),
            subpath: &subpath,
        };
        for entry in iter {
            if entry.kind() != FileKind::ModFile {
                pause_logging();
            }
            self.localization.handle_file(entry, &self.fullpath(entry));
            if entry.kind() != FileKind::ModFile {
                resume_logging();
            }
        }
        self.localization.finalize();
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
