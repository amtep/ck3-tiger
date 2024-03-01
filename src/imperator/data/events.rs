use std::path::PathBuf;
use std::str::FromStr;

use fnv::FnvHashMap;

use crate::block::{Block, Field};
use crate::context::{Reason, ScopeContext};
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_ai_chance, validate_modifiers_with_base, ListType};
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct ImperatorEvents {
    events: FnvHashMap<(String, u16), Event>,
    namespaces: FnvHashMap<String, Token>,
}

impl ImperatorEvents {
    fn load_event(&mut self, key: Token, block: Block) {
        if let Some((key_a, key_b)) = key.as_str().split_once('.') {
            if let Ok(id) = u16::from_str(key_b) {
                if let Some(other) = self.get_event(key.as_str()) {
                    dup_error(&key, &other.key, "event");
                }
                self.events.insert((key_a.to_string(), id), Event::new(key, block));
                return;
            }
        }
        let msg = "Event names should be in the form NAMESPACE.NUMBER";
        let info ="where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of up to 4 digits.";
        warn(ErrorKey::EventNamespace).msg(msg).info(info).loc(key).push();
    }

    pub fn get_event(&self, key: &str) -> Option<&Event> {
        if let Some((namespace, id)) = key.split_once('.') {
            if let Ok(id) = u16::from_str(id) {
                return self.events.get(&(namespace.to_string(), id));
            }
        }
        None
    }

    pub fn check_scope(&self, token: &Token, sc: &mut ScopeContext) {
        if let Some(event) = self.get_event(token.as_str()) {
            sc.expect(event.expects_scope, &Reason::Token(token.clone()));
        }
    }

    pub fn namespace_exists(&self, key: &str) -> bool {
        self.namespaces.contains_key(key)
    }

    pub fn iter_namespace_keys(&self) -> impl Iterator<Item = &Token> {
        self.namespaces.values()
    }

    pub fn exists(&self, key: &str) -> bool {
        if let Some((namespace, id)) = key.split_once('.') {
            if let Ok(id) = u16::from_str(id) {
                if self.events.contains_key(&(namespace.to_string(), id)) {
                    return true;
                }
            }
        }
        false
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.events.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.events.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for ImperatorEvents {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("events")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for item in block.drain() {
            if let Some(Field(key, _, bv)) = item.expect_into_field() {
                if key.is("namespace") {
                    if let Some(value) = bv.expect_into_value() {
                        self.namespaces.insert(value.to_string(), value);
                    }
                } else if let Some(block) = bv.into_block() {
                    self.load_event(key, block);
                } else {
                    let msg = "unknown setting in event files";
                    err(ErrorKey::UnknownField).msg(msg).loc(key).push();
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    key: Token,
    block: Block,
    expects_scope: Scopes,
    expects_from_token: Token,
}

const EVENT_TYPES: &[&str] = &[
    "character_event",
    "minor_character_event",
    "country_event",
    "minor_country_event",
    "major_country_event",
    "state_event",
    "province_event",
];

impl Event {
    pub fn new(key: Token, block: Block) -> Self {
        let mut expects_scope = Scopes::Country;
        let mut expects_from_token = key.clone();

        if let Some(event_type) = block.get_field_value("type") {
            match event_type.as_str() {
                "minor_character_event" | "character_event" => {
                    expects_scope = Scopes::Character;
                    expects_from_token = event_type.clone();
                }
                "country_event" | "minor_country_event" | "major_country_event" => {
                    expects_from_token = event_type.clone();
                }
                "state_event" => {
                    expects_scope = Scopes::State;
                    expects_from_token = event_type.clone();
                }
                "province_event" => {
                    expects_scope = Scopes::Province;
                    expects_from_token = event_type.clone();
                }
                _ => (),
            }
        }

        Self { key, block, expects_scope, expects_from_token }
    }
    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        let mut tooltipped_immediate = Tooltipped::Past;
        let mut tooltipped = Tooltipped::Yes;
        if let Some((namespace, _)) = self.key.as_str().split_once('.') {
            if !data.item_exists(Item::EventNamespace, namespace) {
                let msg = format!("event file should start with `namespace = {namespace}`");
                let info = "otherwise the event won't be found in-game";
                err(ErrorKey::EventNamespace).msg(msg).info(info).loc(&self.key).push();
            }
        }

        let mut sc = ScopeContext::new(self.expects_scope, &self.expects_from_token);
        sc.set_strict_scopes(false);

        vd.field_choice("type", EVENT_TYPES);
        vd.field_bool("hidden");
        vd.field_bool("interface_lock");
        vd.field_bool("fire_only_once");
        vd.field_item_or_target("goto_location", &mut sc, Item::Province, Scopes::Province);

        vd.field_validated_sc("title", &mut sc, validate_desc);
        vd.field_validated_sc("desc", &mut sc, validate_desc);

        let hidden = self.block.field_value_is("hidden", "yes");
        if hidden {
            tooltipped_immediate = Tooltipped::No;
            tooltipped = Tooltipped::No;
        }

        let mut minor_event = false;
        if self.block.field_value_is("type", "minor_character_event")
            || self.block.field_value_is("type", "minor_country_event")
        {
            minor_event = true;
        }

        if !hidden && !minor_event {
            vd.req_field("picture");
        }
        vd.field_item("picture", Item::EventPicture);

        for field in &["left_portrait", "right_portrait"] {
            let mut count = 0;
            vd.multi_field_validated_value(field, |_, mut vd| {
                count += 1;
                vd.target_ok_this(&mut sc, Scopes::Character);
                if count == 4 {
                    let msg = format!("Event has more than 3 {field} attributes.");
                    let info = "Events can only have up to 3 portraits displayed at a time.";
                    warn(ErrorKey::Validation).msg(msg).info(info).loc(&self.key).push();
                }
            });
        }

        vd.field_validated_block_sc("weight_multiplier", &mut sc, validate_modifiers_with_base);

        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("immediate", |block, data| {
            validate_effect(block, data, &mut sc, tooltipped_immediate);
        });

        if !hidden {
            vd.req_field("option");
        }
        vd.multi_field_validated_block("option", |block, data| {
            validate_event_option(block, data, &mut sc, tooltipped);
        });

        vd.field_validated_block("after", |block, data| {
            validate_effect(block, data, &mut sc, tooltipped_immediate);
        });
    }
}

fn validate_event_option(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_sc("name", sc, validate_desc);

    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });

    vd.field_bool("exclusive");
    vd.field_bool("highlight");
    vd.field_validated_sc("ai_chance", sc, validate_ai_chance);
    validate_effect_internal(
        &Lowercase::new_unchecked("option"),
        ListType::None,
        block,
        data,
        sc,
        vd,
        tooltipped,
    );
}
