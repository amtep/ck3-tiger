use fnv::FnvHashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, DefinitionItem};
use crate::context::ScopeContext;
use crate::data::scripted_effects::Effect;
use crate::data::scripted_triggers::Trigger;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_normal_effect};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn, warn_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::{scope_from_snake_case, Scopes};
use crate::token::Token;
use crate::trigger::{validate_normal_trigger, validate_target};
use crate::validate::{
    validate_ai_chance, validate_cooldown, validate_modifiers_with_base, validate_theme_background,
    validate_theme_icon, validate_theme_sound, validate_theme_transition, ListType,
};

#[derive(Clone, Debug, Default)]
pub struct Events {
    events: FnvHashMap<(String, u16), Event>,
    triggers: FnvHashMap<(PathBuf, String), Trigger>,
    effects: FnvHashMap<(PathBuf, String), Effect>,

    // These events are known to exist, so don't warn about them not being found,
    // but they had errors on validation.
    error_events: FnvHashMap<String, Token>,
}

impl Events {
    fn load_event(&mut self, key: &Token, block: &Block, namespaces: &[&str]) {
        if namespaces.is_empty() {
            let msg = "Event files must start with a namespace declaration";
            error(key, ErrorKey::EventNamespace, msg);
            return;
        }
        if let Some((key_a, key_b)) = key.as_str().split_once('.') {
            if let Ok(id) = u16::from_str(key_b) {
                if namespaces.contains(&key_a) {
                    if let Some(other) = self.get_event(key.as_str()) {
                        dup_error(key, &other.key, "event");
                    }
                    self.events.insert(
                        (key_a.to_string(), id),
                        Event::new(key.clone(), block.clone()),
                    );
                    return;
                }
                warn_info(key, ErrorKey::EventNamespace, "Event name should start with namespace", "If the event doesn't match its namespace, the game can't properly find the event when triggering it.");
            } else {
                warn_info(key, ErrorKey::EventNamespace, "Event names should be in the form NAMESPACE.NUMBER", "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of up to 4 digits.");
            }
        } else {
            warn_info(key, ErrorKey::EventNamespace, "Event names should be in the form NAMESPACE.NUMBER", "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of up to 4 digits.");
        }

        self.error_events.insert(key.to_string(), key.clone());
    }

    fn load_scripted_trigger(&mut self, key: Token, block: &Block) {
        let index = (key.loc.pathname.to_path_buf(), key.to_string());
        if let Some(other) = self.triggers.get(&index) {
            dup_error(&key, &other.key, "scripted trigger");
        }
        self.triggers
            .insert(index, Trigger::new(key, block.clone()));
    }

    fn load_scripted_effect(&mut self, key: Token, block: &Block) {
        let index = (key.loc.pathname.to_path_buf(), key.to_string());
        if let Some(other) = self.effects.get(&index) {
            dup_error(&key, &other.key, "scripted effect");
        }
        self.effects.insert(index, Effect::new(key, block.clone()));
    }

    pub fn trigger_exists(&self, key: &Token) -> bool {
        let index = (key.loc.pathname.to_path_buf(), key.to_string());
        self.triggers.contains_key(&index)
    }

    pub fn get_trigger(&self, key: &Token) -> Option<&Trigger> {
        let index = (key.loc.pathname.to_path_buf(), key.to_string());
        self.triggers.get(&index)
    }

    pub fn effect_exists(&self, key: &Token) -> bool {
        let index = (key.loc.pathname.to_path_buf(), key.to_string());
        self.effects.contains_key(&index)
    }

    pub fn get_effect(&self, key: &Token) -> Option<&Effect> {
        let index = (key.loc.pathname.to_path_buf(), key.to_string());
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
            sc.expect(event.expects_scope, token);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        if let Some((namespace, id)) = key.split_once('.') {
            if let Ok(id) = u16::from_str(id) {
                if self.events.contains_key(&(namespace.to_string(), id)) {
                    return true;
                }
            }
        }
        self.error_events.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        let mut vec = self.effects.values().collect::<Vec<&Effect>>();
        vec.sort_unstable_by_key(|item| &item.key.loc);
        for item in vec {
            item.validate(data);
        }

        let mut vec = self.triggers.values().collect::<Vec<&Trigger>>();
        vec.sort_unstable_by_key(|item| &item.key.loc);
        for item in vec {
            item.validate(data);
        }

        let mut vec = self.events.values().collect::<Vec<&Event>>();
        vec.sort_unstable_by_key(|item| &item.key.loc);
        for item in vec {
            item.validate(data);
        }
    }
}

impl FileHandler for Events {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("events")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        #[derive(Copy, Clone)]
        enum Expecting {
            Event,
            ScriptedTrigger,
            ScriptedEffect,
        }

        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };

        let mut namespaces = Vec::new();
        let mut expecting = Expecting::Event;

        for def in block.iter_definitions_warn() {
            match def {
                DefinitionItem::Assignment(key, value) if key.is("namespace") => {
                    namespaces.push(value.as_str());
                }
                DefinitionItem::Assignment(key, _)
                    if key.is("scripted_trigger") || key.is("scripted_effect") =>
                {
                    let msg = format!("`{key}` should be used without `=`");
                    error(key, ErrorKey::Validation, &msg);
                }
                DefinitionItem::Assignment(key, _) => {
                    error(key, ErrorKey::Validation, "unknown setting in event files");
                }
                DefinitionItem::Keyword(key)
                    if matches!(expecting, Expecting::Event) && key.is("scripted_trigger") =>
                {
                    expecting = Expecting::ScriptedTrigger;
                }
                DefinitionItem::Keyword(key)
                    if matches!(expecting, Expecting::Event) && key.is("scripted_effect") =>
                {
                    expecting = Expecting::ScriptedEffect;
                }
                DefinitionItem::Keyword(key) => error_info(
                    key,
                    ErrorKey::Validation,
                    "unexpected token",
                    "Did you forget an = ?",
                ),
                DefinitionItem::Definition(key, b) if key.is("namespace") => {
                    let msg = "expected namespace to have a simple string value";
                    error(b, ErrorKey::EventNamespace, msg);
                }
                DefinitionItem::Definition(key, b) => match expecting {
                    Expecting::ScriptedTrigger => {
                        self.load_scripted_trigger(key.clone(), b);
                        expecting = Expecting::Event;
                    }
                    Expecting::ScriptedEffect => {
                        self.load_scripted_effect(key.clone(), b);
                        expecting = Expecting::Event;
                    }
                    Expecting::Event => {
                        self.load_event(key, b, &namespaces);
                    }
                },
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
const WINDOW_TYPES: &[&str] = &[
    "character_event",
    "duel_event",
    "fullscreen_event",
    "letter_event",
];

impl Event {
    pub fn new(key: Token, block: Block) -> Self {
        let expects_scope = if let Some(token) = block.get_field_value("scope") {
            scope_from_snake_case(token.as_str()).unwrap_or(Scopes::non_primitive())
        } else {
            Scopes::Character
        };

        Self {
            key,
            block,
            expects_scope,
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        let evtype = self
            .block
            .get_field_value("type")
            .map_or("character_event", |t| t.as_str());
        if evtype == "empty" {
            let msg = "`type = empty` has been replaced by `scope = none`";
            error(vd.field_value("type").unwrap(), ErrorKey::Validation, msg);
        } else {
            vd.field_choice("type", EVENT_TYPES);
        }

        if evtype == "character_event" {
            vd.field_choice("window", WINDOW_TYPES);
        } else if evtype == "activity_event" {
            vd.field_value("window"); // TODO: figure out the possible values for this
        } else {
            vd.ban_field("window", || "character events");
        }

        let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());
        if let Some(token) = vd.field_value("scope") {
            if let Some(scope) = scope_from_snake_case(token.as_str()) {
                sc = ScopeContext::new_root(scope, token.clone());
            } else {
                warn(token, ErrorKey::Scopes, "unknown scope type");
            }
        }

        vd.field_bool("hidden");
        vd.field_bool("major");
        vd.field_validated_block("major_trigger", |b, data| {
            validate_normal_trigger(b, data, &mut sc, false);
        });

        vd.field_validated_block("immediate", |b, data| {
            validate_normal_effect(b, data, &mut sc, true);
        });
        vd.field_validated_block("trigger", |b, data| {
            validate_normal_trigger(b, data, &mut sc, false);
        });
        vd.field_validated_block("on_trigger_fail", |b, data| {
            validate_normal_effect(b, data, &mut sc, false);
        });
        vd.field_validated_block_sc("weight_multiplier", &mut sc, validate_modifiers_with_base);

        vd.field_validated_sc("title", &mut sc, validate_desc);
        vd.field_validated_sc("desc", &mut sc, validate_desc);

        if evtype == "letter_event" {
            if let Some(bv) = vd.field("opening") {
                validate_desc(bv, data, &mut sc);
            }
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
        vd.field_item("theme", Item::EventTheme);
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
        vd.field_validated_blocks_sc("option", &mut sc, validate_event_option);

        vd.field_validated_block("after", |b, data| {
            // TODO: check if this block is tooltipped
            validate_normal_effect(b, data, &mut sc, false);
        });
        vd.field_validated_block_sc("cooldown", &mut sc, validate_cooldown);
        vd.field_value("soundeffect");
        vd.field_bool("orphan");
        // TODO: validate widget
        vd.field("widget");
        vd.field_block("widgets");
    }
}

fn validate_event_option(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    // TODO: warn if they use desc, first_valid, random_valid, or triggered_desc directly
    // in the name or tooltip.

    let mut vd = Validator::new(block, data);
    vd.field_validated_bvs("name", |bv, data| match bv {
        BlockOrValue::Value(t) => {
            data.localization.verify_exists(t);
        }
        BlockOrValue::Block(b) => {
            let mut vd = Validator::new(b, data);
            vd.req_field("text");
            if let Some(trigger) = vd.field_block("trigger") {
                validate_normal_trigger(trigger, data, sc, false);
            }
            vd.field_validated_sc("text", sc, validate_desc);
        }
    });

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false);
    });

    vd.field_validated_block("show_as_unavailable", |b, data| {
        validate_normal_trigger(b, data, sc, false);
    });

    vd.field_validated_sc("flavor", sc, validate_desc);
    vd.field_value("reason"); // arbitrary string passed to the UI

    // "this option is available because you have the ... trait"
    vd.field_values_items("trait", Item::Trait);
    vd.field_values_items("skill", Item::Skill);

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

    validate_effect("option", ListType::None, block, data, sc, vd, true);
}

fn validate_court_scene(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.req_field("button_position_character");
    vd.field_target("button_position_character", sc, Scopes::Character);
    vd.field_bool("court_event_force_open");
    vd.field_bool("show_timeout_info");
    vd.field_bool("should_pause_time");
    vd.field_target("court_owner", sc, Scopes::Character);
    vd.field("scripted_animation");
    vd.field_validated_blocks("roles", |b, data| {
        for (key, block) in b.iter_pure_definitions_warn() {
            validate_target(key, data, sc, Scopes::Character);
            let mut vd = Validator::new(block, data);
            vd.req_field("group");
            vd.field_item("group", Item::CourtSceneGroup);
            vd.field_value("animation"); // TODO
            vd.field_validated_blocks("triggered_animation", |b, data| {
                validate_triggered_animation(b, data, sc);
            });
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
        &[
            "lower_left_portrait",
            "lower_center_portrait",
            "lower_right_portrait",
        ],
    );
    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false);
    });
}

fn validate_triggered_animation(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.req_field("trigger");
    vd.req_field("animation");
    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false);
    });
    vd.field_value("animation"); // TODO
}

fn validate_triggered_outfit(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    // trigger is apparently optional
    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false);
    });
    // TODO: check that at least one of these is set?
    vd.field_list("outfit_tags"); // TODO
    vd.field_bool("remove_default_outfit");
    vd.field_bool("hide_info");
}

fn validate_portrait(v: &BlockOrValue, data: &Everything, sc: &mut ScopeContext) {
    match v {
        BlockOrValue::Value(t) => validate_target(t, data, sc, Scopes::Character),
        BlockOrValue::Block(b) => {
            let mut vd = Validator::new(b, data);

            vd.req_field("character");
            vd.field_target("character", sc, Scopes::Character);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, false);
            });
            vd.field_value("animation"); // TODO
            vd.field("scripted_animation"); // TODO
            vd.field_validated_blocks("triggered_animation", |b, data| {
                validate_triggered_animation(b, data, sc);
            });
            vd.field_list("outfit_tags"); // TODO
            vd.field_bool("remove_default_outfit");
            vd.field_bool("hide_info");
            vd.field_validated_blocks("triggered_outfit", |b, data| {
                validate_triggered_outfit(b, data, sc);
            });
            vd.field_value("camera"); // TODO: figure out valid values

            // TODO: is this only useful when animation is prisondungeon ?
            vd.field_bool("override_imprisonment_visuals");
            vd.field_bool("animate_if_dead");
        }
    }
}
