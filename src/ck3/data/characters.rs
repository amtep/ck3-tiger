//! Registry and validation of all characters defined in history/characters/

use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::Ordering;

use atomic_enum::atomic_enum;

use crate::block::{BV, Block, Comparator, Eq::*};
use crate::ck3::data::houses::House;
use crate::ck3::validate::validate_portrait_modifier_overrides;
use crate::context::ScopeContext;
use crate::date::Date;
use crate::effect::{validate_effect, validate_effect_field};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{ErrorKey, Severity, err, fatal, untidy, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    fn from_female_bool(b: bool) -> Self {
        if b { Gender::Female } else { Gender::Male }
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

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct Characters {
    config_only_born: Option<Date>,

    characters: TigerHashMap<&'static str, Character>,

    /// These are characters with duplicate ids. We can't put them in the `characters` map because of the ids,
    /// but we do want to validate them.
    duplicates: Vec<Character>,
}

impl Characters {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.characters.get(key.as_str()) {
            if self.config_only_born.is_none()
                || self
                    .config_only_born
                    .and_then(|date| block.get_field_at_date("birth", date))
                    .is_some()
            {
                err(ErrorKey::DuplicateCharacter)
                    .strong()
                    .msg("duplicate character id")
                    .info("this will create two characters with the same id")
                    .loc(&other.key)
                    .loc_msg(&key, "duplicate")
                    .push();
                self.duplicates.push(Character::new(key, block));
            }
        } else {
            self.characters.insert(key.as_str(), Character::new(key, block));
        }
    }

    pub fn verify_exists_gender(&self, item: &Token, gender: Gender) {
        if let Some(ch) = self.characters.get(item.as_str()) {
            if gender != ch.gender() {
                let msg = format!("character is not {gender}");
                err(ErrorKey::WrongGender).msg(msg).loc(item).push();
            }
        } else {
            let msg = format!("character {item} not defined in history/characters/");
            err(ErrorKey::MissingItem).msg(msg).loc(item).push();
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.characters.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.characters.values().map(|ch| &ch.key).chain(self.duplicates.iter().map(|ch| &ch.key))
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
            let msg = format!("{item} is not alive on {date}");
            warn(ErrorKey::History).msg(msg).loc(item).push();
        }
    }

    pub fn get_dynasty<'a>(
        &'a self,
        id: &Token,
        date: Date,
        data: &'a Everything,
    ) -> Option<&'a Token> {
        self.characters.get(id.as_str()).and_then(|ch| {
            ch.get_dynasty(date).or_else(|| {
                ch.get_house(date).and_then(|house| House::get_dynasty(house.as_str(), data))
            })
        })
    }

    pub fn get_house(&self, id: &Token, date: Date) -> Option<&Token> {
        self.characters.get(id.as_str()).and_then(|ch| ch.get_house(date))
    }

    pub fn get_culture(&self, id: &Token, date: Date) -> Option<&Token> {
        self.characters.get(id.as_str()).and_then(|ch| ch.get_culture(date))
    }

    pub fn get_faith(&self, id: &Token, date: Date) -> Option<&Token> {
        self.characters.get(id.as_str()).and_then(|ch| ch.get_faith(date))
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.characters.values() {
            if item.born_by(self.config_only_born) {
                item.validate(data);
            }
        }
        for item in &self.duplicates {
            if item.born_by(self.config_only_born) {
                item.validate(data);
            }
        }
    }

    // Recursive depth-first search of ancestor graph to see if there are any cycles.
    // If a cycle is found, print a warning about it that includes all the characters in the cycle.
    //
    // If there are multiple cycles, this algorithm might not find them all, because that's an
    // NP-hard problem. Fortunately the user can just re-run after fixing the known cycles.
    //
    // Returns None if `ch` is not part of any cycles.
    // Returns a vector of `Token` if `ch` is part of a cycle.
    fn check_ancestor_cycles(&self, ch: &Character) -> Option<Vec<Token>> {
        match ch.ancestor_state.load(Ordering::Acquire) {
            AncestorState::Unchecked => {
                ch.ancestor_state.store(AncestorState::Checking, Ordering::Release);
            }
            AncestorState::Checking => {
                // Found a cycle
                return Some(vec![ch.key.clone()]);
            }
            AncestorState::Checked => {
                // Reached a char that is known not to be part of any cycles
                // (or is part of a cycle that has already been reported)
                return None;
            }
        }

        for field in &["father", "mother"] {
            if let Some(token) = ch.block.get_field_value(field) {
                if let Some(parent) = self.characters.get(token.as_str()) {
                    if let Some(mut cycle_vec) = self.check_ancestor_cycles(parent) {
                        // unwrap is safe because the vec is always created with one element
                        if &ch.key == cycle_vec.first().unwrap() {
                            let msg = "character is their own ancestor";
                            cycle_vec.reverse();
                            cycle_vec.pop();
                            let mut report = fatal(ErrorKey::Crash).msg(msg).loc(&ch.key);
                            for token in cycle_vec {
                                report = report.loc_msg(token, "from here");
                            }
                            report.push();
                        } else {
                            cycle_vec.push(token.clone());
                            // Returning here means if the father is in a cycle we won't check the
                            // mother, but that's ok because we're not promising to find all
                            // cycles anyway.
                            ch.ancestor_state.store(AncestorState::Checked, Ordering::Release);
                            return Some(cycle_vec);
                        }
                    }
                }
            }
        }
        ch.ancestor_state.store(AncestorState::Checked, Ordering::Release);
        None
    }

    pub fn check_pod_flags(&self, data: &Everything) {
        for item in self.characters.values() {
            if item.born_by(self.config_only_born) {
                item.check_pod_flags(data);
            }
        }
    }
}

impl FileHandler<Block> for Characters {
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

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }

    fn finalize(&mut self) {
        // Find loops in the ancestry tree. These will crash the game.
        for item in self.characters.values() {
            let opt_cycle_vec = self.check_ancestor_cycles(item);
            // It shouldn't be possible that a cycle vec is passed back beyond the first character.
            assert!(opt_cycle_vec.is_none());
        }
    }
}

#[atomic_enum]
enum AncestorState {
    Unchecked,
    Checking,
    Checked,
}

#[derive(Debug)]
pub struct Character {
    key: Token,
    block: Block,
    /// Storage for the ancestor cycle check algorithm.
    ancestor_state: AtomicAncestorState,
}

impl Character {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block, ancestor_state: AtomicAncestorState::new(AncestorState::Unchecked) }
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

    pub fn get_dynasty(&self, date: Date) -> Option<&Token> {
        self.block.get_field_value_at_date("dynasty", date)
    }

    pub fn get_house(&self, date: Date) -> Option<&Token> {
        self.block.get_field_value_at_date("dynasty_house", date)
    }

    pub fn get_culture(&self, date: Date) -> Option<&Token> {
        self.block.get_field_value_at_date("culture", date)
    }

    pub fn get_faith(&self, date: Date) -> Option<&Token> {
        self.block
            .get_field_value_at_date("faith", date)
            .or_else(|| self.block.get_field_value_at_date("religion", date))
    }

    fn validate_life_event(
        date: Date,
        gender: Gender,
        key: &Token,
        bv: &BV,
        data: &Everything,
        sc: &mut ScopeContext,
    ) -> Option<(LifeEventType, Token)> {
        use LifeEventType::*;

        match bv {
            BV::Value(value) => {
                if matches!(key.as_str(), "trait" | "add_trait") && value.as_str() == "saint" {
                    return Some((Posthumous, value.clone()));
                }

                match key.as_str() {
                    "name" => {
                        data.localization.verify_name_exists(value, Severity::Warning);
                        return None;
                    }
                    "birth" => {
                        if !value.is("yes") && Date::from_str(value.as_str()).is_err() {
                            let msg = "expected `yes` or a date";
                            err(ErrorKey::Validation).msg(msg).loc(value).push();
                        }
                        return Some((Birth, key.clone()));
                    }
                    "death" => {
                        if !value.is("yes") && !value.is_date() {
                            data.verify_exists(Item::DeathReason, value);
                        }
                        return Some((Death, key.clone()));
                    }
                    // religion and faith both mean faith here
                    "religion" | "faith" => {
                        data.verify_exists(Item::Faith, value);
                        return None;
                    }
                    "culture" => {
                        data.verify_exists(Item::Culture, value);
                        return None;
                    }
                    "trait" => {
                        data.verify_exists(Item::Trait, value);
                        return None;
                    }
                    "employer" => {
                        if value.is("0") {
                            return Some((Unemployed, key.clone()));
                        }
                        data.verify_exists(Item::Character, value);
                        if data.item_exists(Item::Character, value.as_str()) {
                            data.characters.verify_alive(value, date);
                        }
                        return Some((Employed, key.clone()));
                    }
                    "moved_to_pool" => {
                        if !value.is("yes") {
                            let msg = "expected `yes`";
                            err(ErrorKey::Validation).msg(msg).loc(value).push();
                        }
                        return Some((Unemployed, key.clone()));
                    }
                    "give_council_position" => {
                        data.verify_exists(Item::CouncilPosition, value);
                        return None;
                    }
                    "capital" => {
                        data.verify_exists(Item::Title, value);
                        if !value.as_str().starts_with("c_") {
                            err(ErrorKey::Validation)
                                .msg("capital must be a county")
                                .loc(value)
                                .push();
                        }
                        return None;
                    }
                    "add_spouse" | "add_matrilineal_spouse" => {
                        data.characters.verify_exists_gender(value, gender.flip());
                        if data.item_exists(Item::Character, value.as_str()) {
                            data.characters.verify_alive(value, date);
                        }
                        return Some((AddSpouse, value.clone()));
                    }
                    "add_same_sex_spouse" => {
                        data.characters.verify_exists_gender(value, gender);
                        if data.item_exists(Item::Character, value.as_str()) {
                            data.characters.verify_alive(value, date);
                        }
                        return Some((AddSpouse, value.clone()));
                    }
                    "add_concubine" => {
                        data.characters.verify_exists_gender(value, gender.flip());
                        if data.item_exists(Item::Character, value.as_str()) {
                            data.characters.verify_alive(value, date);
                        }
                        return None;
                    }
                    "remove_spouse" => return Some((RemoveSpouse, value.clone())),
                    "dynasty" => {
                        data.verify_exists(Item::Dynasty, value);
                        return None;
                    }
                    "dynasty_house" => {
                        data.verify_exists(Item::House, value);
                        return None;
                    }
                    _ => (),
                }
            }
            BV::Block(block) => match key.as_str() {
                "death" => {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("death_reason");
                    vd.field_item("death_reason", Item::DeathReason);
                    vd.field_item("killer", Item::Character);
                    return Some((Death, key.clone()));
                }
                "effect" => {
                    validate_effect(block, data, sc, Tooltipped::No);
                    return None;
                }
                _ => (),
            },
        };

        // unknown effect field
        validate_effect_field(
            Lowercase::empty(),
            key,
            Comparator::Equals(Single),
            bv,
            data,
            sc,
            Tooltipped::No,
        );

        None
    }

    fn validate_life(character: &Token, life_events: Vec<LifeEvent>) {
        let mut birth = None;
        let mut death = None;
        let mut spouses = TigerHashSet::<Token>::default();
        let mut employed = false;

        for LifeEvent { date, index: _, token, event } in life_events {
            use LifeEventType::*;

            if birth.is_none() && event != Birth {
                let msg = format!("{character} was not born yet on {date}");
                let mut loc = token.loc;
                loc.column = 0;
                warn(ErrorKey::History).msg(msg).loc(loc).push();
            }

            if let Some((death_date, death_loc)) = death {
                if event != Posthumous {
                    let msg = format!(
                        "{character} was not alive on {date}, had already died on {death_date}"
                    );
                    let mut loc = token.loc;
                    loc.column = 0;
                    warn(ErrorKey::History)
                        .msg(msg)
                        .loc(loc)
                        .loc_msg(death_loc, "from here")
                        .push();
                }
            }

            match event {
                Birth => {
                    let mut loc = token.loc;
                    loc.column = 0;

                    if let Some((birth_date, birth_loc)) = birth {
                        let msg = format!(
                            "{character} couldn't be born again on {date}, was born already on {birth_date}"
                        );
                        warn(ErrorKey::History)
                            .msg(msg)
                            .loc(loc)
                            .loc_msg(birth_loc, "from here")
                            .push();
                    }
                    birth = Some((date, loc));
                }
                AddSpouse => {
                    if !spouses.insert(token.clone()) {
                        let msg = format!("{character} already had {token} as a spouse on {date}");
                        let curr_token = spouses.get(&token).unwrap();
                        warn(ErrorKey::History)
                            .msg(msg)
                            .loc(token)
                            .loc_msg(curr_token, "from here")
                            .push();
                    }
                }
                RemoveSpouse => {
                    if !spouses.remove(&token) {
                        let msg = format!("{character} did not have {token} as a spouse on {date}");
                        warn(ErrorKey::History).msg(msg).loc(token).push();
                    }
                }
                Employed => employed = true,
                Unemployed => {
                    if !employed {
                        let msg = format!("{character} was unemployed anyway on {date}");
                        untidy(ErrorKey::History).msg(msg).loc(token).push();
                    }
                    employed = false;
                }
                Death => {
                    let mut loc = token.loc;
                    loc.column = 0;
                    death = Some((date, loc));
                }
                Posthumous => {
                    if death.is_none() {
                        let msg = format!("{character} had not died yet on {date}");
                        warn(ErrorKey::History).msg(msg).loc(token).push();
                    }
                }
            }
        }
    }

    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Character, &self.key);

        if self.key.as_str().contains('.') {
            let msg =
                format!("`character:{}` will not work because of the dot in the id", &self.key);
            let info = "script code will not be able to refer to this character";
            warn(ErrorKey::CharacterId).msg(msg).info(info).loc(&self.key).push();
        }

        vd.req_field("name");
        if let Some(name) = vd.field_value("name") {
            data.localization.verify_name_exists(name, Severity::Warning);
        }

        vd.field_item("dna", Item::Dna);
        vd.field_bool("female");
        vd.field_integer("martial");
        vd.field_integer("prowess");
        vd.field_integer("diplomacy");
        vd.field_integer("intrigue");
        vd.field_integer("stewardship");
        vd.field_integer("learning");
        vd.multi_field_item("trait", Item::Trait);

        if let Some(ch) = vd.field_value("father") {
            data.characters.verify_exists_gender(ch, Gender::Male);
        }

        if let Some(ch) = vd.field_value("mother") {
            data.characters.verify_exists_gender(ch, Gender::Female);
        }

        vd.field_bool("disallow_random_traits");

        // religion and faith both mean faith here
        vd.field_item("religion", Item::Faith);
        vd.field_item("faith", Item::Faith);

        vd.field_item("culture", Item::Culture);

        vd.field_item("dynasty", Item::Dynasty);
        vd.field_item("dynasty_house", Item::House);

        vd.field_item("give_nickname", Item::Nickname);
        vd.field_item("sexuality", Item::Sexuality);
        vd.field_numeric("health");
        vd.field_numeric("fertility");

        vd.field_validated_block("portrait_override", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block(
                "portrait_modifier_overrides",
                validate_portrait_modifier_overrides,
            );
            vd.field_validated_block("hair", validate_color);
        });

        let mut life_events = Vec::new();
        let gender = Gender::from_female_bool(self.block.get_field_bool("female").unwrap_or(false));
        vd.validate_history_blocks(|date, _key, block, data| {
            for (index, (key, bv)) in block.iter_assignments_and_definitions_warn().enumerate() {
                if let Some((life_event_type, token)) =
                    Self::validate_life_event(date, gender, key, bv, data, &mut sc)
                {
                    life_events.push(LifeEvent { date, index, event: life_event_type, token });
                }
            }
        });

        life_events.sort_unstable();
        Self::validate_life(&self.key, life_events);
    }

    fn check_pod_flags(&self, _data: &Everything) {
        if self.block.has_key("dna")
            && self.has_trait("nosferatu")
            && !self.has_flag("had_POD_character_nosferatu_looks")
            && !self.key.is("791762")
        {
            let msg = "nosferatu with predefined dna lacks had_POD_character_nosferatu_looks";
            err(ErrorKey::PrincesOfDarkness).msg(msg).loc(&self.key).push();
        }
    }

    fn has_flag(&self, flag: &str) -> bool {
        for (key, block) in self.block.iter_definitions() {
            if key.is_date() {
                if block_has_flag(block, flag) {
                    return true;
                }
                for block in block.get_field_blocks("effect") {
                    if block_has_flag(block, flag) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn has_trait(&self, tr: &str) -> bool {
        for token in self.block.get_field_values("trait") {
            if token.is(tr) {
                return true;
            }
        }
        for (key, block) in self.block.iter_definitions() {
            if key.is_date() {
                for token in block.get_field_values("add_trait") {
                    if token.is(tr) {
                        return true;
                    }
                }
                if let Some(block) = block.get_field_block("effect") {
                    for token in block.get_field_values("add_trait") {
                        if token.is(tr) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

fn block_has_flag(block: &Block, flag: &str) -> bool {
    for token in block.get_field_values("add_character_flag") {
        if token.is(flag) {
            return true;
        }
    }
    for block in block.get_field_blocks("add_character_flag") {
        if block.field_value_is("flag", flag) {
            return true;
        }
    }
    false
}

#[derive(Debug)]
enum LifeEventType {
    /// All other events must happen after birth
    Birth,
    AddSpouse,
    RemoveSpouse,
    Employed,
    /// Must be employed already
    Unemployed,
    /// All other events must happen before death
    Death,
    Posthumous,
    // TODO add Effect validation, e.g. `add_trait`
}

impl PartialEq for LifeEventType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for LifeEventType {}

#[derive(Debug)]
struct LifeEvent {
    date: Date,
    index: usize,
    event: LifeEventType,
    token: Token,
}

impl PartialEq for LifeEvent {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date && self.index == other.index
    }
}

impl Eq for LifeEvent {}

impl Ord for LifeEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.date.cmp(&other.date) {
            std::cmp::Ordering::Equal => self.index.cmp(&other.index),
            other => other,
        }
    }
}

impl PartialOrd for LifeEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
