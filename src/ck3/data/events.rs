use std::path::{Path, PathBuf};
use std::str::FromStr;

use fnv::FnvHashMap;

use crate::block::{Block, BlockItem, Field, BV};
use crate::ck3::validate::{
    validate_theme_background, validate_theme_icon, validate_theme_sound, validate_theme_transition,
};
use crate::context::{Reason, ScopeContext};
use crate::data::scripted_effects::Effect;
use crate::data::scripted_triggers::Trigger;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::pathtable::PathTableIndex;
use crate::pdxfile::PdxFile;
use crate::report::{error, error_info, old_warn, warn, warn_info, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target, validate_trigger};
use crate::validate::{
    validate_ai_chance, validate_duration, validate_modifiers_with_base, ListType,
};
use crate::validator::Validator;

#[derive(Debug, Default)]
pub struct Ck3Events {
    events: FnvHashMap<(String, u16), Event>,
    namespaces: FnvHashMap<String, Token>,
    triggers: FnvHashMap<(PathTableIndex, String), Trigger>,
    effects: FnvHashMap<(PathTableIndex, String), Effect>,
}

impl Ck3Events {
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
        warn_info(key, ErrorKey::EventNamespace, "Event names should be in the form NAMESPACE.NUMBER", "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of up to 4 digits.");
    }

    fn load_scripted_trigger(&mut self, key: Token, block: Block) {
        let index = (key.loc.path, key.to_string());
        if let Some(other) = self.triggers.get(&index) {
            dup_error(&key, &other.key, "scripted trigger");
        }
        self.triggers.insert(index, Trigger::new(key, block, None));
    }

    fn load_scripted_effect(&mut self, key: Token, block: Block) {
        let index = (key.loc.path, key.to_string());
        if let Some(other) = self.effects.get(&index) {
            dup_error(&key, &other.key, "scripted effect");
        }
        self.effects.insert(index, Effect::new(key, block, None));
    }

    pub fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        let index = (key.loc.path, key.to_string());
        self.triggers.get(&index)
    }

    pub fn get_effect(&self, key: &Token) -> Option<&Effect> {
        let index = (key.loc.path, key.to_string());
        self.effects.get(&index)
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
        for item in self.effects.values() {
            item.validate(data);
        }

        for item in self.triggers.values() {
            item.validate(data);
        }

        for item in self.events.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Ck3Events {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("events")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, fullpath)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        #[derive(Copy, Clone)]
        enum Expecting {
            Event,
            ScriptedTrigger,
            ScriptedEffect,
        }

        let mut expecting = Expecting::Event;

        for item in block.drain() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is("namespace") {
                    if let Some(value) = bv.expect_into_value() {
                        self.namespaces.insert(value.to_string(), value);
                    }
                } else if key.is("scripted_trigger") || key.is("scripted_effect") {
                    let msg = format!("`{key}` should be used without `=`");
                    error(key, ErrorKey::ParseError, &msg);
                } else if let Some(block) = bv.into_block() {
                    match expecting {
                        Expecting::ScriptedTrigger => {
                            self.load_scripted_trigger(key, block);
                            expecting = Expecting::Event;
                        }
                        Expecting::ScriptedEffect => {
                            self.load_scripted_effect(key, block);
                            expecting = Expecting::Event;
                        }
                        Expecting::Event => {
                            self.load_event(key, block);
                        }
                    };
                } else {
                    let msg = "unknown setting in event files";
                    error(key, ErrorKey::UnknownField, msg);
                }
            } else if let Some(key) = item.expect_value() {
                if matches!(expecting, Expecting::Event) && key.is("scripted_trigger") {
                    expecting = Expecting::ScriptedTrigger;
                } else if matches!(expecting, Expecting::Event) && key.is("scripted_effect") {
                    expecting = Expecting::ScriptedEffect;
                } else {
                    error_info(
                        key,
                        ErrorKey::Validation,
                        "unexpected token",
                        "Did you forget an = ?",
                    );
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
}

const EVENT_TYPES: &[&str] = &[
    "letter_event",
    "character_event",
    "court_event",
    "duel_event",
    "fullscreen_event",
    "activity_event",
];

// TODO: check if mods can add more window types to gui/event_windows/
const WINDOW_TYPES: &[&str] =
    &["character_event", "duel_event", "fullscreen_event", "letter_event"];

impl Event {
    pub fn new(key: Token, block: Block) -> Self {
        let expects_scope = if let Some(token) = block.get_field_value("scope") {
            Scopes::from_snake_case(token.as_str()).unwrap_or(Scopes::non_primitive())
        } else {
            Scopes::Character
        };

        Self { key, block, expects_scope }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        let mut tooltipped_immediate = Tooltipped::Past;
        let mut tooltipped = Tooltipped::Yes;
        if let Some((namespace, _)) = self.key.as_str().split_once('.') {
            if !data.item_exists(Item::EventNamespace, namespace) {
                let msg = format!("event file should start with `namespace = {namespace}`");
                let info = "otherwise the event won't be found in-game";
                error_info(&self.key, ErrorKey::EventNamespace, &msg, info);
            }
            if namespace == "debug" {
                // Suppress missing-localization messages caused via these debug events
                tooltipped_immediate = Tooltipped::No;
                tooltipped = Tooltipped::No;
            }
        }

        let evtype = self.block.get_field_value("type").map_or("character_event", |t| t.as_str());
        if evtype == "empty" {
            let msg = "`type = empty` has been replaced by `scope = none`";
            error(vd.field_value("type").unwrap(), ErrorKey::Validation, msg);
        } else {
            vd.field_choice("type", EVENT_TYPES);
        }

        if evtype == "character_event" {
            vd.field_choice("window", WINDOW_TYPES);
        } else if evtype == "activity_event" {
            // TODO: get the choices here from the gui code
            vd.field_choice(
                "window",
                &[
                    "activity_event",
                    "tour_arrival_event",
                    "widget_activity_locale_fullscreen_event",
                    "tournament_fullscreen_pivotal_event_widget",
                ],
            );
        } else {
            vd.ban_field("window", || "character_event or activity_event");
        }

        let mut sc = ScopeContext::new(Scopes::Character, &self.key);
        if let Some(token) = vd.field_value("scope") {
            if let Some(scope) = Scopes::from_snake_case(token.as_str()) {
                sc = ScopeContext::new(scope, token);
            } else {
                old_warn(token, ErrorKey::Scopes, "unknown scope type");
            }
        }
        sc.set_strict_scopes(false);

        // "dlc or mod this event comes from"
        vd.field_item("content_source", Item::Localization);

        vd.field_bool("hidden");
        vd.field_bool("major");
        vd.field_validated_block("major_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("immediate", |b, data| {
            validate_effect(b, data, &mut sc, tooltipped_immediate);
        });
        vd.field_validated_block("trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_trigger_fail", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block_sc("weight_multiplier", &mut sc, validate_modifiers_with_base);

        vd.field_validated_sc("title", &mut sc, validate_desc);
        vd.field_validated_sc("desc", &mut sc, validate_desc);

        if evtype == "letter_event" {
            vd.field_validated_sc("opening", &mut sc, validate_desc);
            vd.req_field("sender");
            vd.field_validated_sc("sender", &mut sc, validate_portrait);
        } else {
            vd.advice_field("opening", "only needed for letter_event");
            vd.advice_field("sender", "only needed for letter_event");
        }
        if evtype == "court_event" {
            vd.advice_field("left_portrait", "not needed for court_event");
            vd.advice_field("right_portrait", "not needed for court_event");
            vd.advice_field("center_portrait", "not needed for court_event");
        } else {
            vd.field_validated("left_portrait", |bv, data| {
                validate_portrait(bv, data, &mut sc);
            });
            vd.field_validated("right_portrait", |bv, data| {
                validate_portrait(bv, data, &mut sc);
            });
            vd.field_validated("center_portrait", |bv, data| {
                validate_portrait(bv, data, &mut sc);
            });
        }
        vd.field_validated("lower_left_portrait", |bv, data| {
            validate_portrait(bv, data, &mut sc);
        });
        vd.field_validated("lower_center_portrait", |bv, data| {
            validate_portrait(bv, data, &mut sc);
        });
        vd.field_validated("lower_right_portrait", |bv, data| {
            validate_portrait(bv, data, &mut sc);
        });
        // TODO: check that artifacts are not in the same position as a character
        vd.field_validated_blocks_sc("artifact", &mut sc, validate_artifact);
        vd.field_validated_block_sc("court_scene", &mut sc, validate_court_scene);
        if let Some(token) = vd.field_value("theme") {
            data.verify_exists(Item::EventTheme, token);
            data.validate_call(Item::EventTheme, token, &self.block, &mut sc);
        }
        // TODO: warn if more than one of each is defined with no trigger
        if evtype == "court_event" {
            vd.advice_field("override_background", "not needed for court_event");
        } else {
            vd.field_validated_bvs_sc("override_background", &mut sc, validate_theme_background);
        }
        vd.field_validated_blocks_sc("override_icon", &mut sc, validate_theme_icon);
        vd.field_validated_blocks_sc("override_sound", &mut sc, validate_theme_sound);
        vd.field_validated_blocks_sc("override_transition", &mut sc, validate_theme_transition);
        // Note: override_environment seems to be unused, and themes defined in
        // common/event_themes don't have environments. So I left it out even though
        // it's in the docs.

        if !self.block.get_field_bool("hidden").unwrap_or(false) {
            vd.req_field("option");
        }
        vd.field_validated_blocks("option", |block, data| {
            validate_event_option(block, data, &mut sc, tooltipped);
        });

        vd.field_validated_block("after", |b, data| {
            validate_effect(b, data, &mut sc, tooltipped);
        });
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);
        vd.field_value("soundeffect"); // TODO
        vd.field_bool("orphan");
        // TODO: validate widget
        vd.field("widget");
        vd.field_block("widgets");
    }
}

fn validate_event_option(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_bvs("name", |bv, data| match bv {
        BV::Value(t) => {
            data.localization.verify_exists(t);
        }
        BV::Block(b) => {
            let mut vd = Validator::new(b, data);
            vd.req_field("text");
            vd.field_validated_block("trigger", |block, data| {
                validate_trigger(block, data, sc, Tooltipped::No);
            });
            vd.field_validated_sc("text", sc, validate_desc);
            for field in &["desc", "first_valid", "random_valid", "triggered_desc"] {
                vd.advice_field(field, "use this inside `name = { text = { ... } }`");
            }
        }
    });

    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });

    vd.field_validated_block("show_as_unavailable", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });

    vd.field_validated_sc("flavor", sc, validate_desc);
    vd.field_value("reason"); // arbitrary string passed to the UI

    // "this option is available because you have the ... trait"
    vd.field_items("trait", Item::Trait);
    vd.field_items("skill", Item::Skill);

    vd.field_validated_sc("ai_chance", sc, validate_ai_chance);

    // TODO: check what this does.
    vd.field_bool("exclusive");

    // TODO: check what this does.
    vd.field_bool("is_cancel_option");

    // If fallback = yes, the option is shown despite its trigger,
    // if there would otherwise be no other option
    vd.field_bool("fallback");

    vd.field_target("highlight_portrait", sc, Scopes::Character);
    vd.field_bool("show_unlock_reason");

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

fn validate_court_scene(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.req_field("button_position_character");
    vd.field_target("button_position_character", sc, Scopes::Character);
    vd.field_bool("court_event_force_open");
    vd.field_bool("show_timeout_info");
    vd.field_bool("should_pause_time");
    vd.field_target("court_owner", sc, Scopes::Character);
    vd.field_item("scripted_animation", Item::ScriptedAnimation);
    vd.field_validated_blocks("roles", |b, data| {
        for (key, bv) in b.iter_assignments_and_definitions_warn() {
            match bv {
                BV::Block(block) => {
                    validate_target(key, data, sc, Scopes::Character);
                    let mut vd = Validator::new(block, data);
                    vd.req_field("group");
                    vd.field_item("group", Item::CourtSceneGroup);
                    vd.field_item("animation", Item::PortraitAnimation);
                    vd.field_validated_blocks("triggered_animation", |b, data| {
                        validate_triggered_animation(b, data, sc);
                    });
                }
                BV::Value(token) => {
                    data.verify_exists(Item::CourtSceneGroup, token);
                }
            }
        }
    });
}

fn validate_artifact(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.req_field("target");
    vd.req_field("position");
    vd.field_target("target", sc, Scopes::Artifact);
    vd.field_choice(
        "position",
        &["lower_left_portrait", "lower_center_portrait", "lower_right_portrait"],
    );
    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
}

fn validate_animations(vd: &mut Validator) {
    vd.field_validated_value("animation", |_, value, data| {
        if !data.item_exists(Item::PortraitAnimation, value.as_str())
            && data.item_exists(Item::ScriptedAnimation, value.as_str())
        {
            let msg = format!(
                "portrait animation {value} not defined in {}",
                Item::PortraitAnimation.path()
            );
            let info = format!("Did you mean `scripted_animation = {value}`?");
            warn(ErrorKey::MissingItem).strong().msg(msg).info(info).loc(value).push();
        } else {
            data.verify_exists(Item::PortraitAnimation, value);
        }
    });
    vd.field_validated_value("scripted_animation", |_, value, data| {
        if !data.item_exists(Item::ScriptedAnimation, value.as_str())
            && data.item_exists(Item::PortraitAnimation, value.as_str())
        {
            let msg = format!(
                "scripted animation {value} not defined in {}",
                Item::ScriptedAnimation.path()
            );
            let info = format!("Did you mean `animation = {value}`?");
            warn(ErrorKey::MissingItem).strong().msg(msg).info(info).loc(value).push();
        } else {
            data.verify_exists(Item::ScriptedAnimation, value);
        }
    });
}

fn validate_triggered_animation(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.set_max_severity(Severity::Warning);

    vd.req_field("trigger");
    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    vd.req_field_one_of(&["animation", "scripted_animation"]);
    validate_animations(&mut vd);
}

fn validate_triggered_outfit(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.set_max_severity(Severity::Warning);

    // trigger is apparently optional
    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    // TODO: check that at least one of these is set?
    vd.field_list("outfit_tags"); // TODO
    vd.field_bool("remove_default_outfit");
    vd.field_bool("hide_info");
}

fn validate_portrait(v: &BV, data: &Everything, sc: &mut ScopeContext) {
    match v {
        BV::Value(t) => validate_target(t, data, sc, Scopes::Character),
        BV::Block(b) => {
            let mut vd = Validator::new(b, data);

            vd.req_field("character");
            vd.field_target("character", sc, Scopes::Character);
            vd.field_validated_block("trigger", |b, data| {
                validate_trigger(b, data, sc, Tooltipped::No);
            });
            validate_animations(&mut vd);
            vd.field_validated_blocks("triggered_animation", |b, data| {
                validate_triggered_animation(b, data, sc);
            });
            vd.field_list("outfit_tags"); // TODO
            vd.field_bool("remove_default_outfit");
            vd.field_bool("hide_info");
            vd.field_validated_blocks("triggered_outfit", |b, data| {
                validate_triggered_outfit(b, data, sc);
            });
            vd.field_item("camera", Item::PortraitCamera);

            // TODO: is this only useful when animation is prisondungeon ?
            vd.field_bool("override_imprisonment_visuals");
            vd.field_bool("animate_if_dead");
        }
    }
}
