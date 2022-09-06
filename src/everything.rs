use anyhow::Result;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;

use crate::block::Block;
use crate::data::characters::Characters;
use crate::data::decisions::Decisions;
use crate::data::dynasties::Dynasties;
use crate::data::events::Events;
use crate::data::gameconcepts::GameConcepts;
use crate::data::houses::Houses;
use crate::data::interactions::Interactions;
use crate::data::localization::Localization;
use crate::data::namelists::Namelists;
use crate::data::prov_history::ProvinceHistories;
use crate::data::provinces::Provinces;
use crate::data::relations::Relations;
use crate::data::religions::Religions;
use crate::data::scripted_effects::Effects;
use crate::data::scripted_lists::ScriptedLists;
use crate::data::scripted_triggers::Triggers;
use crate::data::scriptvalues::ScriptValues;
use crate::data::titles::Titles;
use crate::data::traits::Traits;
use crate::errorkey::ErrorKey;
use crate::errors::{ignore_key, ignore_key_for, warn};
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::pdxfile::PdxFile;
use crate::token::Loc;

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
    pub fileset: Fileset,

    /// Processed localization files
    pub localization: Localization,

    pub scripted_lists: ScriptedLists,

    /// Processed event files
    pub events: Events,

    /// Processed decision files
    pub decisions: Decisions,

    /// Processed character interaction files
    pub interactions: Interactions,

    /// Processed map data
    pub provinces: Provinces,

    /// Processed history/provinces data
    pub province_histories: ProvinceHistories,

    /// Processed game concepts
    pub game_concepts: GameConcepts,

    /// Religions and faiths
    pub religions: Religions,

    /// Landed titles
    pub titles: Titles,

    pub dynasties: Dynasties,
    pub houses: Houses,
    pub characters: Characters,

    /// Cultural name lists
    pub namelists: Namelists,

    /// Scripted relations
    pub relations: Relations,

    pub scriptvalues: ScriptValues,

    pub triggers: Triggers,
    pub effects: Effects,

    pub traits: Traits,
}

impl Everything {
    pub fn new(
        vanilla_root: &Path,
        mod_root: &Path,
        replace_paths: Vec<PathBuf>,
    ) -> Result<Self, FilesError> {
        let mut fileset = Fileset::new(
            vanilla_root.to_path_buf(),
            mod_root.to_path_buf(),
            replace_paths,
        );

        // Abort if whole directories are unreadable, because then we don't have
        // a full map of vanilla's or the mod's contents and might give bad advice.
        fileset.scan(vanilla_root, FileKind::Vanilla).map_err(|e| {
            FilesError::VanillaUnreadable {
                path: vanilla_root.to_path_buf(),
                source: e,
            }
        })?;
        fileset
            .scan(mod_root, FileKind::Mod)
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
            Block::new(Loc::for_file(Rc::new(config_file), FileKind::Mod))
        };

        fileset.config(config.clone());

        Ok(Everything {
            fileset,
            config,
            localization: Localization::default(),
            scripted_lists: ScriptedLists::default(),
            events: Events::default(),
            decisions: Decisions::default(),
            interactions: Interactions::default(),
            provinces: Provinces::default(),
            province_histories: ProvinceHistories::default(),
            game_concepts: GameConcepts::default(),
            religions: Religions::default(),
            titles: Titles::default(),
            dynasties: Dynasties::default(),
            houses: Houses::default(),
            characters: Characters::default(),
            namelists: Namelists::default(),
            relations: Relations::default(),
            scriptvalues: ScriptValues::default(),
            triggers: Triggers::default(),
            effects: Effects::default(),
            traits: Traits::default(),
        })
    }

    fn _read_config(path: &Path) -> Result<Block> {
        PdxFile::read_no_bom(path, FileKind::Mod, path)
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

        self.fileset.handle(&mut self.localization);
        self.fileset.handle(&mut self.scripted_lists);
        self.fileset.handle(&mut self.events);
        self.fileset.handle(&mut self.decisions);
        self.fileset.handle(&mut self.interactions);
        self.fileset.handle(&mut self.provinces);
        self.fileset.handle(&mut self.province_histories);
        self.fileset.handle(&mut self.game_concepts);
        self.fileset.handle(&mut self.religions);
        self.fileset.handle(&mut self.titles);
        self.fileset.handle(&mut self.dynasties);
        self.fileset.handle(&mut self.houses);
        self.fileset.handle(&mut self.characters);
        self.fileset.handle(&mut self.namelists);
        self.fileset.handle(&mut self.relations);
        self.fileset.handle(&mut self.scriptvalues);
        self.fileset.handle(&mut self.triggers);
        self.fileset.handle(&mut self.effects);
        self.fileset.handle(&mut self.traits);
    }

    pub fn validate_all(&mut self) {
        self.fileset.validate(self);
        self.localization.validate(self);
        self.scripted_lists.validate(self);
        self.events.validate(self);
        self.decisions.validate(self);
        self.interactions.validate(self);
        self.provinces.validate(self);
        self.province_histories.validate(self);
        self.game_concepts.validate(self);
        self.religions.validate(self);
        self.titles.validate(self);
        self.dynasties.validate(self);
        self.houses.validate(self);
        self.characters.validate(self);
        self.namelists.validate(self);
        self.relations.validate(self);
        self.scriptvalues.validate(self);
        self.triggers.validate(self);
        self.effects.validate(self);
        self.traits.validate(self);
    }

    pub fn check_pod(&mut self) {
        self.province_histories
            .check_pod_faiths(&self.religions, &self.titles);
    }
}
