use fnv::{FnvHashMap, FnvHashSet};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, Date};
use crate::context::ScopeContext;
use crate::effect::{validate_effect, validate_normal_effect};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn, warn_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::{validate_prefix_reference_token, ListType};

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

    pub fn exists(&self, key: &str) -> bool {
        self.characters.contains_key(key)
    }

    pub fn is_alive(&self, item: &Token, date: Date) -> bool {
        if let Some(item) = self.characters.get(item.as_str()) {
            item.is_alive(date)
        } else {
            false
        }
    }

    pub fn verify_alive(&self, item: &Token, date: Date) {
        if !self.is_alive(item, date) {
            let msg = format!("{} is not alive on {}", item, date);
            warn(item, ErrorKey::History, &msg);
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

    // Check the ancestors of `ch` to see if `ch` is among them.
    // If a cycle is found, return a `Vec` with the ancestry from `ch` up to `ch`.
    pub fn _check_ancestors<'a>(
        &'a self,
        item: &'a Character,
        ch: &str,
        checking: &mut FnvHashSet<&'a str>,
    ) -> Vec<String> {
        let first = checking.is_empty();
        if item.key.is(ch) && !first {
            // Found a cycle
            return vec![ch.to_string()];
        }

        if checking.contains(&item.key.as_str()) {
            // not necessarily a cycle, could just be a shared ancestor
            return Vec::new();
        }
        checking.insert(item.key.as_str());

        let mut cycle_vec = Vec::new();
        if let Some(token) = item.block.get_field_value("father") {
            if let Some(parent) = self.characters.get(token.as_str()) {
                cycle_vec = self._check_ancestors(parent, ch, checking);
            }
        }
        if let Some(token) = item.block.get_field_value("mother") {
            if let Some(parent) = self.characters.get(token.as_str()) {
                cycle_vec = self._check_ancestors(parent, ch, checking);
            }
        }
        if !cycle_vec.is_empty() && !first {
            cycle_vec.insert(0, item.key.to_string());
        }
        cycle_vec
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

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
        };

        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }

    fn finalize(&mut self) {
        // Find loops in the ancestry tree. These will crash the game.
        for item in self.characters.values() {
            let mut checking = FnvHashSet::default();
            let cycle_vec = self._check_ancestors(item, item.key.as_str(), &mut checking);
            if !cycle_vec.is_empty() {
                let info = format!("via {}", cycle_vec.join(", "));
                warn_info(
                    &item.key,
                    ErrorKey::Crash,
                    "character is their own ancestor",
                    &info,
                );
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

    pub fn is_alive(&self, date: Date) -> bool {
        // TODO: figure out if we need to account for deaths triggered in effect { } blocks
        self.block.get_field_at_date("birth", date).is_some()
            && self.block.get_field_at_date("death", date).is_none()
    }

    pub fn validate_history(
        date: Date,
        block: &Block,
        parent: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        let mut vd = Validator::new(block, data);
        vd.field_value_item("name", Item::Localization);

        vd.field_value("birth"); // TODO: can be "yes" or a date
        vd.field("death"); // TODO: can be "yes" or { death_reason = }

        // religion and faith both mean faith here
        vd.field_value_item("religion", Item::Faith);
        vd.field_value_item("faith", Item::Faith);

        if let Some(token) = vd.field_value("set_character_faith") {
            validate_prefix_reference_token(token, data, "faith");
        }

        if let Some(token) = vd.field_value("employer") {
            if !token.is("0") {
                data.verify_exists(Item::Character, token);
                if data.item_exists(Item::Character, token.as_str()) {
                    data.characters.verify_alive(token, date);
                }
            }
        }
        vd.field_value("culture");
        vd.field_value("set_culture");
        vd.field_values_items("trait", Item::Trait);
        vd.field_values_items("add_trait", Item::Trait);
        vd.field_values_items("remove_trait", Item::Trait);
        vd.fields("add_character_flag"); // TODO: can be flag name or { flag = }
        for token in vd.field_values("add_pressed_claim") {
            validate_prefix_reference_token(token, data, "title");
        }
        for token in vd.field_values("remove_claim") {
            validate_prefix_reference_token(token, data, "title");
        }
        if let Some(token) = vd.field_value("capital") {
            data.verify_exists(Item::Title, token);
            if !token.as_str().starts_with("c_") {
                error(token, ErrorKey::Validation, "capital must be a county");
            }
        }

        let gender = Gender::from_female_bool(parent.get_field_bool("female").unwrap_or(false));
        for token in vd.field_values("add_spouse") {
            data.characters.verify_exists_gender(token, gender.flip());
            if data.item_exists(Item::Character, token.as_str()) {
                data.characters.verify_alive(token, date);
            }
        }
        for token in vd.field_values("add_matrilineal_spouse") {
            data.characters.verify_exists_gender(token, gender.flip());
            if data.item_exists(Item::Character, token.as_str()) {
                data.characters.verify_alive(token, date);
            }
        }
        for token in vd.field_values("add_same_sex_spouse") {
            data.characters.verify_exists_gender(token, gender);
            if data.item_exists(Item::Character, token.as_str()) {
                data.characters.verify_alive(token, date);
            }
        }
        for token in vd.field_values("add_concubine") {
            data.characters.verify_exists_gender(token, gender.flip());
            if data.item_exists(Item::Character, token.as_str()) {
                data.characters.verify_alive(token, date);
            }
        }
        for token in vd.field_values("remove_spouse") {
            // TODO: also check that they were a spouse
            data.characters.verify_exists_gender(token, gender.flip());
        }

        vd.field_value_item("dynasty", Item::Dynasty);
        vd.field_value_item("dynasty_house", Item::House);

        vd.field_value_item("give_nickname", Item::Nickname);

        vd.field_numeric("add_prestige");
        vd.field_numeric("add_piety");
        vd.field_numeric("add_gold");

        // TODO: check if they have an employer at this date?
        vd.field_value_item("give_council_position", Item::CouncilPosition);

        vd.field_validated_blocks("effect", |b, data| {
            validate_normal_effect(b, data, sc, false);
        });

        validate_effect("", ListType::None, block, data, sc, vd, false);
    }

    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());

        vd.req_field("name");
        vd.field_value_item("name", Item::Localization);

        vd.field_value("dna");
        vd.field_bool("female");
        vd.field_integer("martial");
        vd.field_integer("prowess");
        vd.field_integer("diplomacy");
        vd.field_integer("intrigue");
        vd.field_integer("stewardship");
        vd.field_integer("learning");
        vd.field_values_items("trait", Item::Trait);

        if let Some(ch) = vd.field_value("father") {
            data.characters.verify_exists_gender(ch, Gender::Male);
        }

        if let Some(ch) = vd.field_value("mother") {
            data.characters.verify_exists_gender(ch, Gender::Female);
        }

        vd.field_bool("disallow_random_traits");

        // religion and faith both mean faith here
        vd.field_value_item("religion", Item::Faith);
        vd.field_value_item("faith", Item::Faith);

        vd.field_value("culture");

        vd.field_value_item("dynasty", Item::Dynasty);
        vd.field_value_item("dynasty_house", Item::House);

        vd.field_value("give_nickname");
        vd.field_value_item("sexuality", Item::Sexuality);
        vd.field_numeric("health");
        vd.field_numeric("fertility");
        vd.field_block("portrait_override");

        vd.validate_history_blocks(|date, b, data| {
            Self::validate_history(date, b, &self.block, data, &mut sc);
        });
    }
}
