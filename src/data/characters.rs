use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, Date};
use crate::data::dynasties::Dynasties;
use crate::data::houses::Houses;
use crate::data::localization::Localization;
use crate::data::religions::Religions;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, will_log, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Characters {
    config_only_born: Option<Date>,

    characters: FnvHashMap<String, Character>,
}

impl Characters {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.characters.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind
                && will_log(key, ErrorKey::Duplicate)
                && other.born_by(self.config_only_born)
            {
                error(
                    key,
                    ErrorKey::Duplicate,
                    "character redefines an existing character",
                );
                info(
                    &other.key,
                    ErrorKey::Duplicate,
                    "the other character is here",
                );
            }
        }
        self.characters
            .insert(key.to_string(), Character::new(key.clone(), block.clone()));
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        for character in self.characters.values() {
            if character.born_by(self.config_only_born) {
                character.check_have_locas(locas);
            }
        }
    }

    pub fn check_have_dynasties(&self, houses: &Houses, dynasties: &Dynasties) {
        for character in self.characters.values() {
            if character.born_by(self.config_only_born) {
                character.check_have_dynasty(houses, dynasties);
            }
        }
    }

    pub fn check_have_faiths(&self, religions: &Religions) {
        for character in self.characters.values() {
            if character.born_by(self.config_only_born) {
                character.check_have_faith(religions);
            }
        }
    }

    pub fn verify_have_character(&self, ch: &Token) {
        if !self.characters.contains_key(ch.as_str()) {
            error(
                ch,
                ErrorKey::MissingItem,
                "character not defined in history/characters/",
            );
        }
    }

    fn finalize_history(&self, b: &Block) {
        if let Some(ch) = b.get_field_value("employer") {
            self.verify_have_character(ch);
        }
        if let Some(ch) = b.get_field_value("add_spouse") {
            self.verify_have_character(ch);
        }
        if let Some(ch) = b.get_field_value("remove_spouse") {
            self.verify_have_character(ch);
        }
        if let Some(ch) = b.get_field_value("add_matrilineal_spouse") {
            self.verify_have_character(ch);
        }
        if let Some(ch) = b.get_field_value("add_same_sex_spouse") {
            self.verify_have_character(ch);
        }
        if let Some(ch) = b.get_field_value("add_concubine") {
            self.verify_have_character(ch);
        }
    }
}

impl FileHandler for Characters {
    fn config(&mut self, config: &Block) {
        if let Some(block) = config.get_field_block("characters") {
            if let Some(born) = block.get_field_value("only_born") {
                if let Ok(date) = Date::try_from(born) {
                    self.config_only_born = Some(date);
                }
            }
        }
    }

    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/characters")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

        let block = match PdxFile::read(entry.path(), entry.kind(), fullpath) {
            Ok(block) => block,
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{:#}", e),
                );
                return;
            }
        };

        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }

    fn finalize(&mut self) {
        for character in self.characters.values() {
            if !character.born_by(self.config_only_born) {
                continue;
            }
            if let Some(ch) = character.block.get_field_value("father") {
                self.verify_have_character(ch);
            }
            if let Some(ch) = character.block.get_field_value("mother") {
                self.verify_have_character(ch);
            }
            for (k, b) in character.block.iter_pure_definitions() {
                if Date::try_from(k).is_ok() {
                    self.finalize_history(b);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Character {
    key: Token,
    block: Block,
}

impl Character {
    pub fn new(key: Token, block: Block) -> Self {
        Self::validate(&block);
        Self { key, block }
    }

    pub fn born_by(&self, born_by: Option<Date>) -> bool {
        if let Some(date) = born_by {
            self.block.get_field_at_date("birth", date).is_some()
        } else {
            true
        }
    }

    pub fn validate_history(block: &Block) {
        let mut vd = Validator::new(block);
        vd.opt_field_value("name");
        vd.opt_field_value("birth");
        vd.opt_field("death"); // can be { death_reason = }
        vd.opt_field_value("religion");
        vd.opt_field_value("faith");
        vd.opt_field_value("employer");
        vd.opt_field_value("set_house");
        vd.opt_field_value("culture");
        vd.opt_field_value("set_culture");
        vd.opt_field_value("set_character_faith_no_effect");
        vd.opt_field_values("trait");
        vd.opt_field_values("add_trait");
        vd.opt_field_values("remove_trait");
        vd.opt_fields("add_character_flag"); // can be { flag = }
        vd.opt_field_values("add_pressed_claim"); // TODO: check title exists
        vd.opt_field_values("remove_claim"); // TODO: check title exists
        vd.opt_field_values("capital"); // TODO: check title exists. This one is without title:
        vd.opt_field_values("add_spouse");
        vd.opt_field_values("add_matrilineal_spouse");
        vd.opt_field_values("add_same_sex_spouse");
        vd.opt_field_values("add_concubine");
        vd.opt_field_values("remove_spouse");
        vd.opt_field_blocks("add_secret");
        vd.opt_field_value("give_nickname");
        vd.opt_field_blocks("create_alliance");
        vd.opt_field_value("dynasty");
        vd.opt_field_value("dynasty_house");
        vd.opt_field_integer("set_immortal_age");
        // At this point it seems that just about any effect can be here
        // without an effect block around it.
        vd.opt_field_integer("add_gold");
        vd.opt_field_blocks("effect");
        vd.warn_remaining();
    }

    pub fn validate(block: &Block) {
        let mut vd = Validator::new(block);

        vd.req_field_value("name");
        vd.opt_field_value("dna");
        vd.opt_field_bool("female");
        vd.opt_field_integer("martial");
        vd.opt_field_integer("prowess");
        vd.opt_field_integer("diplomacy");
        vd.opt_field_integer("intrigue");
        vd.opt_field_integer("stewardship");
        vd.opt_field_integer("learning");
        vd.opt_field_values("trait");
        vd.opt_field_value("father");
        vd.opt_field_value("mother");
        vd.opt_field_bool("disallow_random_traits");
        vd.opt_field_value("religion");
        vd.opt_field_value("faith");
        vd.opt_field_value("culture");
        vd.opt_field_value("dynasty");
        vd.opt_field_value("dynasty_house");
        vd.opt_field_value("give_nickname");
        vd.opt_field_value("sexuality");
        vd.opt_field_value("health");
        vd.opt_field_value("fertility");
        vd.opt_field_block("portrait_override");
        vd.validate_history_blocks(Self::validate_history);
        vd.warn_remaining();
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        if let Some(loca) = self.block.get_field_value("name") {
            locas.verify_have_key(loca.as_str(), loca, "character");
        }
        for (k, b) in self.block.iter_pure_definitions() {
            if Date::try_from(k).is_ok() {
                if let Some(loca) = b.get_field_value("name") {
                    locas.verify_have_key(loca.as_str(), loca, "character");
                }
            }
        }
    }

    pub fn check_have_dynasty(&self, houses: &Houses, dynasties: &Dynasties) {
        if let Some(dynasty) = self.block.get_field_value("dynasty") {
            dynasties.verify_have_dynasty(dynasty);
        }
        if let Some(house) = self.block.get_field_value("dynasty_house") {
            houses.verify_have_house(house);
        }
        for (k, b) in self.block.iter_pure_definitions() {
            if Date::try_from(k).is_ok() {
                if let Some(house) = b.get_field_value("set_house") {
                    houses.verify_have_house(house);
                }
                if let Some(house) = b.get_field_value("dynasty_house") {
                    houses.verify_have_house(house);
                }
                if let Some(dynasty) = b.get_field_value("dynasty") {
                    dynasties.verify_have_dynasty(dynasty);
                }
            }
        }
    }

    pub fn check_have_faith(&self, religions: &Religions) {
        if let Some(faith) = self.block.get_field_value("religion") {
            religions.verify_have_faith(faith);
        }
        if let Some(faith) = self.block.get_field_value("faith") {
            religions.verify_have_faith(faith);
        }
        for (k, b) in self.block.iter_pure_definitions() {
            if Date::try_from(k).is_ok() {
                if let Some(faith) = b.get_field_value("religion") {
                    religions.verify_have_faith(faith);
                }
                if let Some(faith) = b.get_field_value("faith") {
                    religions.verify_have_faith(faith);
                }
                if let Some(faith) = b.get_field_value("set_character_faith_no_effect") {
                    religions.verify_have_faith(faith);
                }
            }
        }
    }
}
