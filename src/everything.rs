use anyhow::Result;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;

use crate::block::{Block, Loc};
use crate::decisions::Decisions;
use crate::errorkey::ErrorKey;
use crate::errors::{ignore_key, ignore_key_for, warn};
use crate::events::Events;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::gameconcepts::GameConcepts;
use crate::interactions::Interactions;
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::prov_history::ProvinceHistories;
use crate::provinces::Provinces;
use crate::religions::Religions;
use crate::titles::Titles;

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

#[derive(Clone, Debug)]
pub struct Everything {
    /// Config from file
    config: Block,

    /// The CK3 and mod files
    fileset: Fileset,

    /// Processed localization files
    localizations: Localization,

    /// Processed event files
    events: Events,

    /// Processed decision files
    decisions: Decisions,

    /// Processed character interaction files
    interactions: Interactions,

    /// Processed map data
    provinces: Provinces,

    /// Processed history/provinces data
    province_histories: ProvinceHistories,

    /// Processed game concepts
    game_concepts: GameConcepts,

    /// Religions and faiths
    religions: Religions,

    /// Landed titles
    titles: Titles,
}

impl Everything {
    pub fn new(vanilla_root: &Path, mod_root: &Path) -> Result<Self, FilesError> {
        let mut fileset = Fileset::new(vanilla_root.to_path_buf(), mod_root.to_path_buf());

        // Abort if whole directories are unreadable, because then we don't have
        // a full map of vanilla's or the mod's contents and might give bad advice.
        fileset
            .scan(vanilla_root, FileKind::VanillaFile)
            .map_err(|e| FilesError::VanillaUnreadable {
                path: vanilla_root.to_path_buf(),
                source: e,
            })?;
        fileset
            .scan(mod_root, FileKind::ModFile)
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
            Block::new(Loc::for_file(Rc::new(config_file), FileKind::ModFile))
        };

        fileset.config(config.clone());

        Ok(Everything {
            fileset,
            config,
            localizations: Localization::default(),
            events: Events::default(),
            decisions: Decisions::default(),
            interactions: Interactions::default(),
            provinces: Provinces::default(),
            province_histories: ProvinceHistories::default(),
            game_concepts: GameConcepts::default(),
            religions: Religions::default(),
            titles: Titles::default(),
        })
    }

    fn _read_config(path: &Path) -> Result<Block> {
        PdxFile::read_no_bom(path, FileKind::ModFile, path)
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        self.fileset.fullpath(entry)
    }

    pub fn load_errorkey_config(&self) {
        for block in self.config.get_field_blocks("ignore") {
            let keynames = block.get_field_values("key");
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

            let pathnames = block.get_field_values("file");
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

    pub fn load_all(&mut self) {
        self.load_errorkey_config();
        self.fileset.config(self.config.clone());

        self.fileset.handle(&mut self.localizations);
        self.fileset.handle(&mut self.events);
        self.fileset.handle(&mut self.decisions);
        self.fileset.handle(&mut self.interactions);
        self.fileset.handle(&mut self.provinces);
        self.fileset.handle(&mut self.province_histories);
        self.fileset.handle(&mut self.game_concepts);
        self.fileset.handle(&mut self.religions);
        self.fileset.handle(&mut self.titles);
    }

    pub fn check_have_localizations(&self) {
        self.decisions.check_have_locas(&self.localizations);
        self.events.check_have_locas(&self.localizations);
        self.interactions.check_have_locas(&self.localizations);
        self.game_concepts.check_have_locas(&self.localizations);
        self.religions.check_have_locas(&self.localizations);
        self.titles.check_have_locas(&self.localizations);
    }

    pub fn check_have_files(&self) {
        self.decisions.check_have_files(&self.fileset);
        self.interactions.check_have_files(&self.fileset);
        self.game_concepts.check_have_files(&self.fileset);
        self.religions.check_have_files(&self.fileset);
    }

    pub fn check_all(&mut self) {
        self.check_have_localizations();
        self.check_have_files();
    }

    pub fn check_pod(&mut self) {
        self.province_histories
            .check_pod_faiths(&self.religions, &self.titles);
    }
}
