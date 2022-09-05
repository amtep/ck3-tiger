use fnv::FnvHashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, Date};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::token::Token;

const SEXUALITIES: &[&str] = &["heterosexual", "homosexual", "bisexual", "asexual"];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    fn from_female_bool(b: bool) -> Self {
        if b {
            Gender::Female
        } else {
            Gender::Male
        }
    }

    fn flip(self) -> Self {
        match self {
            Gender::Male => Gender::Female,
            Gender::Female => Gender::Male,
        }
    }
}

impl Display for Gender {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Gender::Male => write!(f, "male"),
            Gender::Female => write!(f, "female"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Characters {
    config_only_born: Option<Date>,

    characters: FnvHashMap<String, Character>,
}

impl Characters {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.characters.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind && other.born_by(self.config_only_born) {
                dup_error(key, &other.key, "character");
            }
        }
        self.characters
            .insert(key.to_string(), Character::new(key.clone(), block.clone()));
    }

    pub fn verify_exists_gender(&self, item: &Token, gender: Gender) {
        if let Some(ch) = self.characters.get(item.as_str()) {
            if gender != ch.gender() {
                let msg = format!("character is not {}", gender);
                error(item, ErrorKey::WrongGender, &msg);
            }
        } else {
            error(
                item,
                ErrorKey::MissingItem,
                "character not defined in history/characters/",
            );
        }
    }

    pub fn verify_exists(&self, item: &Token) {
        if !self.characters.contains_key(item.as_str()) {
            error(
                item,
                ErrorKey::MissingItem,
                "character not defined in history/characters/",
            );
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vec = self.characters.values().collect::<Vec<&Character>>();
        vec.sort_unstable_by_key(|item| &item.key.loc);
        for item in vec {
            if item.born_by(self.config_only_born) {
                item.validate(data);
            }
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
}

#[derive(Clone, Debug)]
pub struct Character {
    key: Token,
    block: Block,
}

impl Character {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn born_by(&self, born_by: Option<Date>) -> bool {
        if let Some(date) = born_by {
            self.block.get_field_at_date("birth", date).is_some()
        } else {
            true
        }
    }

    pub fn gender(&self) -> Gender {
        Gender::from_female_bool(self.block.get_field_bool("female").unwrap_or(false))
    }

    pub fn validate_history(block: &Block, parent: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_value_loca("name");

        vd.field_value("birth"); // TODO: can be "yes" or a date
        vd.field("death"); // TODO: can be "yes" or { death_reason = }
                           // religion and faith both mean faith here
        data.religions
            .verify_faith_exists_opt(vd.field_value("religion"));
        data.religions
            .verify_faith_exists_opt(vd.field_value("faith"));

        if let Some(token) = vd.field_value("set_character_faith") {
            // At some point, this should probably be part of general effect-and-scope processing
            if let Some(faith) = token.as_str().strip_prefix("faith:") {
                data.religions.verify_implied_faith_exists(faith, token);
            } else {
                warn(
                    token,
                    ErrorKey::Scopes,
                    "faith should start with `faith:` here",
                );
            }
        }

        if let Some(token) = vd.field_value("employer") {
            data.characters.verify_exists(token);
        }
        vd.field_value("culture");
        vd.field_value("set_culture");
        vd.field_values("trait");
        vd.field_values("add_trait");
        vd.field_values("remove_trait");
        vd.fields("add_character_flag"); // TODO: can be flag name or { flag = }
        for token in vd.field_values("add_pressed_claim") {
            data.titles.verify_exists_prefix(token);
        }
        for token in vd.field_values("remove_claim") {
            data.titles.verify_exists_prefix(token);
        }
        if let Some(token) = vd.field_value("capital") {
            data.titles.verify_exists(token);
            if !token.as_str().starts_with("c_") {
                error(token, ErrorKey::Validation, "capital must be a county");
            }
        }

        let gender = Gender::from_female_bool(parent.get_field_bool("female").unwrap_or(false));
        for token in vd.field_values("add_spouse") {
            data.characters.verify_exists_gender(token, gender.flip());
        }
        for token in vd.field_values("add_matrilineal_spouse") {
            data.characters.verify_exists_gender(token, gender.flip());
        }
        for token in vd.field_values("add_same_sex_spouse") {
            data.characters.verify_exists_gender(token, gender);
        }
        for token in vd.field_values("add_concubine") {
            data.characters.verify_exists_gender(token, gender.flip());
        }
        for token in vd.field_values("remove_spouse") {
            // TODO: also check that they were a spouse
            data.characters.verify_exists_gender(token, gender.flip());
        }
        vd.field_blocks("add_secret");
        vd.field_value("give_nickname");
        vd.field_blocks("create_alliance");

        data.dynasties.verify_exists_opt(vd.field_value("dynasty"));
        data.houses
            .verify_exists_opt(vd.field_value("dynasty_house"));

        vd.field_integer("set_immortal_age");
        // At this point it seems that just about any effect can be here
        // without an effect block around it.
        vd.field_integer("add_gold");
        vd.field_blocks("effect");
        vd.warn_remaining();
    }

    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("name");
        vd.field_value_loca("name");

        vd.field_value("dna");
        vd.field_bool("female");
        vd.field_integer("martial");
        vd.field_integer("prowess");
        vd.field_integer("diplomacy");
        vd.field_integer("intrigue");
        vd.field_integer("stewardship");
        vd.field_integer("learning");
        vd.field_values("trait");

        if let Some(ch) = vd.field_value("father") {
            data.characters.verify_exists_gender(ch, Gender::Male);
        }

        if let Some(ch) = vd.field_value("mother") {
            data.characters.verify_exists_gender(ch, Gender::Female);
        }

        vd.field_bool("disallow_random_traits");

        // religion and faith both mean faith here
        data.religions
            .verify_faith_exists_opt(vd.field_value("religion"));
        data.religions
            .verify_faith_exists_opt(vd.field_value("faith"));

        vd.field_value("culture");

        data.dynasties.verify_exists_opt(vd.field_value("dynasty"));
        data.houses
            .verify_exists_opt(vd.field_value("dynasty_house"));

        vd.field_value("give_nickname");
        vd.field_choice("sexuality", SEXUALITIES);
        vd.field_numeric("health");
        vd.field_numeric("fertility");
        vd.field_block("portrait_override");

        vd.validate_history_blocks(|b, data| Self::validate_history(b, &self.block, data));
        vd.warn_remaining();
    }
}
