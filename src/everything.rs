use anyhow::Result;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;

use crate::decisions::Decisions;
use crate::errorkey::ErrorKey;
use crate::errors::{ignore_key, ignore_key_for, warn};
use crate::events::Events;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::scope::{Loc, Scope};

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

#[derive(Clone, Debug)]
pub struct Everything {
    /// Config from file
    config: Scope,

    /// The CK3 and mod files
    fileset: Fileset,

    /// Processed localization files
    localizations: Localization,

    /// Processed event files
    events: Events,

    /// Processed decision files
    decisions: Decisions,
}

impl Everything {
    pub fn new(vanilla_root: &Path, mod_root: &Path) -> Result<Self, FilesError> {
        let mut fileset = Fileset::new(vanilla_root.to_path_buf(), mod_root.to_path_buf());

        // Abort if whole directories are unreadable, because then we don't have
        // a full map of vanilla's or the mod's contents and might give bad advice.
        fileset
            .scan(&vanilla_root, FileKind::VanillaFile)
            .map_err(|e| FilesError::VanillaUnreadable {
                path: vanilla_root.to_path_buf(),
                source: e,
            })?;
        fileset
            .scan(&mod_root, FileKind::ModFile)
            .map_err(|e| FilesError::ModUnreadable {
                path: mod_root.to_path_buf(),
                source: e,
            })?;
        fileset.finalize();

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
            fileset,
            config,
            localizations: Localization::default(),
            events: Events::default(),
            decisions: Decisions::default(),
        })
    }

    fn _read_config(path: &Path) -> Result<Scope> {
        PdxFile::read_no_bom(path, FileKind::ModFile, path)
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        self.fileset.fullpath(entry)
    }

    pub fn load_errorkey_config(&self) {
        for scope in self.config.get_field_scopes("ignore") {
            let keynames = scope.get_field_values("key");
            if keynames.is_empty() {
                continue;
            }

            let mut keys = Vec::new();
            for keyname in keynames {
                let key = match keyname.as_str().parse() {
                    Ok(key) => key,
                    Err(e) => {
                        warn(keyname, ErrorKey::Config, &format!("{:#}", e));
                        continue;
                    }
                };
                keys.push(key);
            }

            let pathnames = scope.get_field_values("file");
            if pathnames.is_empty() {
                for key in keys {
                    ignore_key(key);
                }
            } else {
                for pathname in pathnames {
                    for &key in &keys {
                        ignore_key_for(PathBuf::from(pathname.as_str()), key);
                    }
                }
            }
        }
    }

    // TODO: these very similar functions that all rely on the FileHandler trait
    // can't be refactored with a handle_files() function because the borrow checker
    // complains that the whole of self is borrowed.

    pub fn load_localizations(&mut self) {
        self.localizations.config(&self.config);
        let subpath = self.localizations.subpath();
        for entry in self.fileset.get_files_under(&subpath) {
            self.localizations.handle_file(entry, &self.fullpath(entry));
        }
        self.localizations.finalize();
    }

    pub fn load_events(&mut self) {
        self.events.config(&self.config);
        let subpath = self.events.subpath();
        for entry in self.fileset.get_files_under(&subpath) {
            self.events.handle_file(entry, &self.fullpath(entry));
        }
        self.events.finalize();
    }

    pub fn load_decisions(&mut self) {
        self.decisions.config(&self.config);
        let subpath = self.decisions.subpath();
        for entry in self.fileset.get_files_under(&subpath) {
            self.decisions.handle_file(entry, &self.fullpath(entry));
        }
        self.decisions.finalize();
    }

    pub fn load_all(&mut self) {
        self.load_errorkey_config();
        self.load_localizations();
        self.load_events();
        self.load_decisions();
    }

    pub fn check_have_localizations(&self) {
        self.decisions.check_have_localizations(&self.localizations);
    }

    pub fn check_all(&mut self) {
        self.check_have_localizations();
    }
}
