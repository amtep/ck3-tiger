use anyhow::Result;
use fnv::FnvHashSet;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;

use crate::block::Block;
use crate::data::characters::Characters;
use crate::data::courtpos::CourtPositions;
use crate::data::courtpos_categories::CourtPositionCategories;
use crate::data::decisions::Decisions;
use crate::data::defines::Defines;
use crate::data::doctrines::Doctrines;
use crate::data::dynasties::Dynasties;
use crate::data::events::Events;
use crate::data::gameconcepts::GameConcepts;
use crate::data::houses::Houses;
use crate::data::interaction_cats::InteractionCategories;
use crate::data::interactions::Interactions;
use crate::data::lifestyles::Lifestyles;
use crate::data::localization::Localization;
use crate::data::maa::MenAtArmsTypes;
use crate::data::namelists::Namelists;
use crate::data::prov_history::ProvinceHistories;
use crate::data::provinces::Provinces;
use crate::data::relations::Relations;
use crate::data::religions::Religions;
use crate::data::scripted_effects::{Effect, Effects};
use crate::data::scripted_lists::ScriptedLists;
use crate::data::scripted_triggers::{Trigger, Triggers};
use crate::data::scriptvalues::ScriptValues;
use crate::data::terrain::Terrains;
use crate::data::title_history::TitleHistories;
use crate::data::titles::Titles;
use crate::data::traits::Traits;
use crate::errorkey::ErrorKey;
use crate::errors::{error, ignore_key, ignore_key_for, ignore_path, warn};
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::rivers::Rivers;
use crate::token::{Loc, Token};

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
    ConfigUnreadable { path: PathBuf },
}

#[derive(Clone, Debug)]
pub struct Everything {
    /// Config from file
    config: Block,

    warned_defines: RefCell<FnvHashSet<String>>,

    /// The CK3 and mod files
    pub fileset: Fileset,

    /// Processed localization files
    pub localization: Localization,

    pub scripted_lists: ScriptedLists,

    pub defines: Defines,

    /// Processed event files
    pub events: Events,

    /// Processed decision files
    pub decisions: Decisions,

    /// Processed character interaction files
    pub interactions: Interactions,
    pub interaction_cats: InteractionCategories,

    /// Processed map data
    pub provinces: Provinces,

    /// Processed history/provinces data
    pub province_histories: ProvinceHistories,

    /// Processed game concepts
    pub gameconcepts: GameConcepts,

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

    pub lifestyles: Lifestyles,

    pub terrains: Terrains,

    pub courtpos_categories: CourtPositionCategories,
    pub courtpos: CourtPositions,

    pub title_history: TitleHistories,

    pub doctrines: Doctrines,

    pub menatarmstypes: MenAtArmsTypes,
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

        let config_file = mod_root.join("ck3-tiger.conf");
        let config = if config_file.is_file() {
            Self::_read_config(&config_file)
                .ok_or(FilesError::ConfigUnreadable { path: config_file })?
        } else {
            Block::new(Loc::for_file(Rc::new(config_file), FileKind::Mod))
        };

        fileset.config(config.clone());

        Ok(Everything {
            fileset,
            config,
            warned_defines: RefCell::new(FnvHashSet::default()),
            localization: Localization::default(),
            scripted_lists: ScriptedLists::default(),
            defines: Defines::default(),
            events: Events::default(),
            decisions: Decisions::default(),
            interactions: Interactions::default(),
            interaction_cats: InteractionCategories::default(),
            provinces: Provinces::default(),
            province_histories: ProvinceHistories::default(),
            gameconcepts: GameConcepts::default(),
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
            lifestyles: Lifestyles::default(),
            terrains: Terrains::default(),
            courtpos_categories: CourtPositionCategories::default(),
            courtpos: CourtPositions::default(),
            title_history: TitleHistories::default(),
            doctrines: Doctrines::default(),
            menatarmstypes: MenAtArmsTypes::default(),
        })
    }

    fn _read_config(path: &Path) -> Option<Block> {
        let entry = FileEntry::new(path.to_path_buf(), FileKind::Mod);
        PdxFile::read_no_bom(&entry, path)
    }

    pub fn fullpath(&self, entry: &FileEntry) -> PathBuf {
        self.fileset.fullpath(entry)
    }

    pub fn load_errorkey_config(&self) {
        for block in self.config.get_field_blocks("ignore") {
            let keynames = block.get_field_values("key");

            let mut keys = Vec::new();
            for keyname in keynames {
                let key = match keyname.as_str().parse() {
                    Ok(key) => key,
                    Err(e) => {
                        warn(keyname, ErrorKey::Config, &format!("{e:#}"));
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
            } else if keys.is_empty() {
                for path in pathnames {
                    ignore_path(PathBuf::from(path.as_str()));
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
        self.fileset.handle(&mut self.defines);
        self.fileset.handle(&mut self.events);
        self.fileset.handle(&mut self.decisions);
        self.fileset.handle(&mut self.interactions);
        self.fileset.handle(&mut self.interaction_cats);
        self.fileset.handle(&mut self.provinces);
        self.fileset.handle(&mut self.province_histories);
        self.fileset.handle(&mut self.gameconcepts);
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
        self.fileset.handle(&mut self.lifestyles);
        self.fileset.handle(&mut self.terrains);
        self.fileset.handle(&mut self.courtpos_categories);
        self.fileset.handle(&mut self.courtpos);
        self.fileset.handle(&mut self.title_history);
        self.fileset.handle(&mut self.doctrines);
        self.fileset.handle(&mut self.menatarmstypes);
    }

    pub fn validate_all(&mut self) {
        self.fileset.validate(self);
        self.localization.validate(self);
        self.scripted_lists.validate(self);
        self.defines.validate(self);
        // scripted items go early because they update their scope context info
        self.scriptvalues.validate(self);
        self.triggers.validate(self);
        self.effects.validate(self);
        self.terrains.validate(self);
        self.events.validate(self);
        self.decisions.validate(self);
        self.interactions.validate(self);
        self.interaction_cats.validate(self);
        self.provinces.validate(self);
        self.province_histories.validate(self);
        self.gameconcepts.validate(self);
        self.religions.validate(self);
        self.titles.validate(self);
        self.dynasties.validate(self);
        self.houses.validate(self);
        self.characters.validate(self);
        self.namelists.validate(self);
        self.relations.validate(self);
        self.traits.validate(self);
        self.lifestyles.validate(self);
        self.courtpos_categories.validate(self);
        self.courtpos.validate(self);
        self.title_history.validate(self);
        self.doctrines.validate(self);
        self.menatarmstypes.validate(self);
    }

    pub fn check_rivers(&mut self) {
        let mut rivers = Rivers::default();
        self.fileset.handle(&mut rivers);
        rivers.validate(self);
    }

    pub fn check_pod(&mut self) {
        self.province_histories
            .check_pod_faiths(&self.religions, &self.titles);
    }

    pub fn item_exists(&self, itype: Item, key: &str) -> bool {
        match itype {
            Item::ActivityState => ACTIVITY_STATES.contains(&key),
            Item::ArtifactHistory => ARTIFACT_HISTORY.contains(&key),
            Item::Character => self.characters.exists(key),
            Item::CourtPositionCategory => self.courtpos_categories.exists(key),
            Item::DangerType => DANGER_TYPES.contains(&key),
            Item::Decision => self.decisions.exists(key),
            Item::Define => self.defines.exists(key),
            Item::Dlc => DLC.contains(&key),
            Item::DlcFeature => DLC_FEATURES.contains(&key),
            Item::Doctrine => self.doctrines.exists(key),
            Item::DoctrineParameter => self.doctrines.parameter_exists(key),
            Item::Dynasty => self.dynasties.exists(key),
            Item::Event => self.events.exists(key),
            Item::Faith => self.religions.faith_exists(key),
            Item::File => self.fileset.exists(key),
            Item::GameConcept => self.gameconcepts.exists(key),
            Item::House => self.houses.exists(key),
            Item::Holding => HOLDING_TYPES.contains(&key),
            Item::Interaction => self.interactions.exists(key),
            Item::InteractionCategory => self.interaction_cats.exists(key),
            Item::Lifestyle => self.lifestyles.exists(key),
            Item::Localization => self.localization.exists(key),
            Item::MenAtArms => self.menatarmstypes.exists(key),
            Item::MenAtArmsBase => self.menatarmstypes.base_exists(key),
            Item::NameList => self.namelists.exists(key),
            Item::PrisonType => PRISON_TYPES.contains(&key),
            Item::Province => self.provinces.exists(key),
            Item::Relation => self.relations.exists(key),
            Item::Religion => self.religions.religion_exists(key),
            Item::RewardItem => REWARD_ITEMS.contains(&key),
            Item::ScriptedEffect => self.effects.exists(key),
            Item::ScriptedList => self.scripted_lists.exists(key),
            Item::ScriptedTrigger => self.triggers.exists(key),
            Item::ScriptValue => self.scriptvalues.exists(key),
            Item::Sexuality => SEXUALITIES.contains(&key),
            Item::Skill => SKILLS.contains(&key),
            Item::Terrain => self.terrains.exists(key),
            Item::Title => self.titles.exists(key),
            Item::TitleHistory => self.title_history.exists(key),
            Item::TitleHistoryType => TITLE_HISTORY_TYPES.contains(&key),
            Item::Trait => self.traits.exists(key),
            Item::TraitCategory => TRAIT_CATEGORIES.contains(&key),
            _ => true,
        }
    }

    pub fn verify_exists(&self, itype: Item, token: &Token) {
        self.verify_exists_implied(itype, token.as_str(), token);
    }

    pub fn verify_exists_implied(&self, itype: Item, key: &str, token: &Token) {
        match itype {
            Item::File => self.fileset.verify_exists_implied(key, token),
            Item::Localization => self.localization.verify_exists_implied(key, token),
            Item::Province => self.provinces.verify_exists_implied(key, token),
            _ => {
                if !self.item_exists(itype, key) {
                    let msg = format!("{} {} not defined in {}", itype, key, itype.path());
                    error(token, ErrorKey::MissingItem, &msg);
                }
            }
        }
    }

    pub fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        if let Some(trigger) = self.triggers.get(key.as_str()) {
            Some(trigger)
        } else if let Some(trigger) = self.events.get_trigger(key) {
            Some(trigger)
        } else {
            None
        }
    }

    pub fn get_effect(&self, key: &Token) -> Option<&Effect> {
        if let Some(effect) = self.effects.get(key.as_str()) {
            Some(effect)
        } else if let Some(effect) = self.events.get_effect(key) {
            Some(effect)
        } else {
            None
        }
    }

    pub fn get_defined_string(&self, key: &str) -> Option<&Token> {
        self.defines.get_string(key)
    }

    pub fn get_defined_string_warn(&self, token: &Token, key: &str) -> Option<&Token> {
        let result = self.get_defined_string(key);
        let mut cache = self.warned_defines.borrow_mut();
        if result.is_none() && !cache.contains(key) {
            warn(
                token,
                ErrorKey::MissingItem,
                &format!("{key} not defined in common/defines/"),
            );
            cache.insert(key.to_string());
        }
        result
    }
}

/// LAST UPDATED VERSION 1.9.0.2
const ACTIVITY_STATES: &[&str] = &["passive", "travel", "active"];

/// LAST UPDATED VERSION 1.9.0.2
const ARTIFACT_HISTORY: &[&str] = &[
    "created_before_history",
    "created",
    "prize_created",
    "discovered",
    "creator_discovered",
    "claimed_by_house",
    "given",
    "stolen",
    "inherited",
    "conquest",
    "taken_in_siege",
    "taken_in_battle",
    "won_in_duel",
    "purchased",
    "prize_awarded",
    "ransomed",
    "reforged",
];

/// LAST UPDATED VERSION 1.9.0.2
// TODO: parse it from dlc_metadata/ ? Unfortunately Tours and Tournaments
// is an exception.
const DLC: &[&str] = &[
    "Fashion of the Abbasid Court",
    "The Northern Lords",
    "Garments of the Holy Roman Empire",
    "The Fate of Iberia",
    "The Royal Court",
    "Friends and Foes",
    "tours_and_tournaments",
    "Elegance of the Empire",
];

/// LAST UPDATED VERSION 1.9.0.2
const DLC_FEATURES: &[&str] = &[
    "garments_of_the_hre",
    "fashion_of_the_abbasid_court",
    "the_northern_lords",
    "hybridize_culture",
    "diverge_culture",
    "royal_court",
    "reform_culture",
    "court_artifacts",
    "the_fate_of_iberia",
    "friends_and_foes",
    "tours_and_tournaments",
    "advanced_activities",
    "accolades",
    "elegance_of_the_empire",
];

// TODO: load this from common/holdings/
/// LAST UPDATED VERSION 1.9.0.2
const HOLDING_TYPES: &[&str] = &[
    "castle_holding",
    "city_holding",
    "church_holding",
    "tribal_holding",
];

/// LAST UPDATED VERSION 1.9.0.2
const REWARD_ITEMS: &[&str] = &["newsletter_crown"];

/// LAST UPDATED VERSION 1.9.0.2
const PRISON_TYPES: &[&str] = &["dungeon", "house_arrest"];

/// LAST UPDATED VERSION 1.9.0.2
const SKILLS: &[&str] = &[
    "diplomacy",
    "intrigue",
    "learning",
    "martial",
    "prowess",
    "stewardship",
];

/// LAST UPDATED VERSION 1.9.0.2
const SEXUALITIES: &[&str] = &["heterosexual", "homosexual", "bisexual", "asexual", "none"];

/// LAST UPDATED VERSION 1.9.0.2
const TITLE_HISTORY_TYPES: &[&str] = &[
    "conquest",
    "conquest_holy_war",
    "conquest_claim",
    "conquest_populist",
    "election",
    "inheritance",
    "abdication",
    "created",
    "destroyed",
    "usurped",
    "granted",
    "revoked",
    "independency",
    "leased_out",
    "lease_revoked",
    "returned",
    "faction_demand",
    "swear_fealty",
];

/// LAST UPDATED VERSION 1.9.0.2",
const TRAIT_CATEGORIES: &[&str] = &[
    "personality",
    "education",
    "childhood",
    "commander",
    "winter_commander",
    "lifestyle",
    "court_type",
    "fame",
    "health",
];

/// LAST UPDATED VERSION 1.9.0.2",
const DANGER_TYPES: &[&str] = &[
    "default",
    "battle",
    "raid",
    "siege",
    "army",
    "occupation",
    "county_control",
    "county_opinion",
    "owner_opinion",
];
