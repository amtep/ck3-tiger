use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, DefinitionItem};
use crate::desc::verify_desc_locas;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, warn_info, will_log, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::token::Token;
use crate::validate::{
    validate_cooldown, validate_theme_background, validate_theme_icon, validate_theme_sound,
};

#[derive(Clone, Debug, Default)]
pub struct Events {
    events: FnvHashMap<String, Event>,
    scripted_triggers: FnvHashMap<String, ScriptedTrigger>,
    scripted_effects: FnvHashMap<String, ScriptedEffect>,

    // These events are known to exist, so don't warn abour them not being found,
    // but they had errors on validation.
    error_events: FnvHashMap<String, Token>,
}

impl Events {
    fn load_event(&mut self, key: &Token, block: &Block, namespaces: &[&str]) {
        let mut namespace_ok = false;
        if namespaces.is_empty() {
            error(
                key,
                ErrorKey::EventNamespace,
                "Event files must start with a namespace declaration",
            );
        } else if let Some((key_a, key_b)) = key.as_str().split_once('.') {
            if key_b.chars().all(|c| c.is_ascii_digit()) {
                if namespaces.contains(&key_a) {
                    namespace_ok = true;
                } else {
                    warn_info(key, ErrorKey::EventNamespace, "Event name should start with namespace", "If the event doesn't match its namespace, the game can't properly find the event when triggering it.");
                }
            } else {
                warn_info(key, ErrorKey::EventNamespace, "Event names should be in the form NAMESPACE.NUMBER", "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of digits.");
            }
        } else {
            warn_info(key, ErrorKey::EventNamespace, "Event names should be in the form NAMESPACE.NUMBER", "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of digits.");
        }

        if namespace_ok {
            if let Some(other) = self.events.get(key.as_str()) {
                if will_log(key, ErrorKey::Duplicate) {
                    error(
                        key,
                        ErrorKey::Duplicate,
                        "event redefines an existing event",
                    );
                    info(&other.key, ErrorKey::Duplicate, "the other event is here");
                }
            }
            self.events
                .insert(key.to_string(), Event::new(key.clone(), block.clone()));
        } else {
            self.error_events.insert(key.to_string(), key.clone());
        }
    }

    fn load_scripted_trigger(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.scripted_triggers.get(key.as_str()) {
            if will_log(&key, ErrorKey::Duplicate) {
                error(
                    &key,
                    ErrorKey::Duplicate,
                    "scripted trigger redefines an existing trigger",
                );
                info(&other.key, ErrorKey::Duplicate, "the other trigger is here");
            }
        }
        self.scripted_triggers
            .insert(key.to_string(), ScriptedTrigger::new(key, block.clone()));
    }

    fn load_scripted_effect(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.scripted_effects.get(key.as_str()) {
            if will_log(&key, ErrorKey::Duplicate) {
                error(
                    &key,
                    ErrorKey::Duplicate,
                    "scripted effect redefines an existing effect",
                );
                info(&other.key, ErrorKey::Duplicate, "the other effect is here");
            }
        }
        self.scripted_effects
            .insert(key.to_string(), ScriptedEffect::new(key, block.clone()));
    }

    pub fn check_have_locas(&self, locs: &Localization) {
        for event in self.events.values() {
            event.check_have_locas(locs);
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
                    error(
                        key,
                        ErrorKey::Validation,
                        &format!("`{}` should be used without `=`", key),
                    );
                }
                DefinitionItem::Assignment(key, _) => error(
                    key,
                    ErrorKey::Validation,
                    "unknown setting in event files, expected only `namespace`",
                ),
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
                    error(
                        b,
                        ErrorKey::EventNamespace,
                        "expected namespace to have a simple string value",
                    );
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
}

impl Event {
    pub fn new(key: Token, block: Block) -> Self {
        Self::validate(&block);
        Self { key, block }
    }

    pub fn validate(block: &Block) {
        let mut vd = Validator::new(block);

        vd.opt_field_bool("hidden");
        vd.opt_field_bool("major");
        vd.opt_field_block("major_trigger"); // trigger
        vd.opt_field_choice(
            "type",
            &[
                "letter_event",
                "character_event",
                "court_event",
                "duel_event",
                "fullscreen_event",
                "empty",
            ],
        );
        let evtype = block
            .get_field_value("type")
            .map_or("missing", |t| t.as_str());
        vd.opt_field_value("scope");
        vd.opt_field_block("immediate"); // effect
        vd.opt_field_block("trigger"); // trigger
        vd.opt_field_block("on_trigger_fail"); // effect
        vd.opt_field_block("weight_multiplier"); // modifier
        vd.opt_field("title"); // desc
        vd.opt_field("desc"); // desc
        if evtype == "letter_event" {
            vd.opt_field("opening"); // desc
            vd.req_field_check("sender", validate_portrait);
        } else {
            vd.advice_field("opening", "only needed for letter_event");
            vd.advice_field("sender", "only needed for letter_event");
        }
        if evtype == "court_event" {
            vd.advice_field("left_portrait", "not needed for court_event");
            vd.advice_field("right_portrait", "not needed for court_event");
        } else {
            vd.opt_field_check("left_portrait", validate_portrait);
            vd.opt_field_check("right_portrait", validate_portrait);
        }
        vd.opt_field_check("lower_left_portrait", validate_portrait);
        vd.opt_field_check("lower_center_portrait", validate_portrait);
        vd.opt_field_check("lower_right_portrait", validate_portrait);
        // TODO: check that artifacts are not in the same position as a character
        vd.opt_field_validated_blocks("artifact", validate_artifact);
        vd.opt_field_validated_block("court_scene", validate_court_scene);
        // TODO: check defined event themes
        vd.opt_field_value("theme");
        // TODO: warn if more than one of each is defined with no trigger
        if evtype == "court_event" {
            vd.advice_field("override_background", "not needed for court_event");
        } else {
            vd.opt_field_validated_blocks("override_background", validate_theme_background);
        }
        vd.opt_field_validated_blocks("override_icon", validate_theme_icon);
        vd.opt_field_validated_blocks("override_sound", validate_theme_sound);
        // Note: override_environment seems to be unused, and themes defined in
        // common/event_themes don't have environments. So I left it out even though
        // it's in the docs.

        // TODO: validate options
        if block.get_field_bool("hidden").unwrap_or(false) {
            vd.opt_field_blocks("option");
        } else {
            vd.req_field_blocks("option");
        }
        vd.opt_field_block("after"); // effect
        vd.opt_field_validated_block("cooldown", validate_cooldown);
        vd.opt_field_value("soundeffect");
        vd.opt_field_bool("orphan");
        // TODO: validate widget
        vd.opt_field("widget");
        vd.opt_field_block("widgets");
        vd.warn_remaining();
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        if let Some(title) = self.block.get_field("title") {
            verify_desc_locas(title, locas, "event title");
        }
        if let Some(desc) = self.block.get_field("desc") {
            verify_desc_locas(desc, locas, "event description");
        }
        if let Some(opening) = self.block.get_field("opening") {
            verify_desc_locas(opening, locas, "letter event opening");
        }
        for option in self.block.get_field_blocks("option") {
            // TODO: descend into the effect blocks and collect custom_tooltip everywhere
            if let Some(name) = option.get_field("name") {
                match name {
                    BlockOrValue::Token(t) => {
                        locas.verify_have_key(t.as_str(), t, "event option name");
                    }
                    BlockOrValue::Block(b) => {
                        if let Some(text) = b.get_field("text") {
                            verify_desc_locas(text, locas, "event option name");
                        } else {
                            warn(b, ErrorKey::Validation, "event option name with no text");
                        }
                    }
                }
            }
            // TODO: see if you can have multiple custom_tooltip in one block (and they all work)
            if let Some(tooltip) = option.get_field("custom_tooltip") {
                match tooltip {
                    BlockOrValue::Token(t) => {
                        locas.verify_have_key(t.as_str(), t, "event option tooltip");
                    }
                    BlockOrValue::Block(b) => {
                        if let Some(text) = b.get_field("text") {
                            verify_desc_locas(text, locas, "event option tooltip");
                        } else {
                            warn(b, ErrorKey::Validation, "event option tooltip with no text");
                        }
                    }
                }
            }
        }
    }
}

fn validate_court_scene(block: &Block) {
    let mut vd = Validator::new(block);

    vd.req_field_value("button_position_character");
    vd.opt_field_bool("court_event_force_open");
    vd.opt_field_bool("show_timeout_info");
    vd.opt_field_bool("should_pause_time");
    vd.opt_field_value("court_owner");
    vd.opt_field("scripted_animation");
    // TODO: validate roles
    vd.opt_field_blocks("roles");
    vd.warn_remaining();
}

fn validate_artifact(block: &Block) {
    let mut vd = Validator::new(block);

    vd.req_field_value("target");
    vd.req_field_choice(
        "position",
        &[
            "lower_left_portrait",
            "lower_center_portrait",
            "lower_right_portrait",
        ],
    );
    vd.opt_field_block("trigger");
    vd.warn_remaining();
}

fn validate_triggered_animation(block: &Block) {
    let mut vd = Validator::new(block);

    vd.req_field_block("trigger");
    vd.req_field_value("animation");
    vd.warn_remaining();
}

fn validate_triggered_outfit(block: &Block) {
    let mut vd = Validator::new(block);

    // trigger is apparently optional
    vd.opt_field_block("trigger");
    // TODO: check that at least one of these is set?
    vd.opt_field_list("outfit_tags");
    vd.opt_field_bool("remove_default_outfit");
    vd.opt_field_bool("hide_info");
    vd.warn_remaining();
}

fn validate_portrait(v: &BlockOrValue) {
    match v {
        BlockOrValue::Token(_) => (),
        BlockOrValue::Block(b) => {
            let mut vd = Validator::new(b);

            vd.req_field_value("character");
            vd.opt_field_block("trigger"); // trigger
            vd.opt_field_value("animation");
            vd.opt_field("scripted_animation");
            vd.opt_field_validated_blocks("triggered_animation", validate_triggered_animation);
            vd.opt_field_list("outfit_tags");
            vd.opt_field_bool("remove_default_outfit");
            vd.opt_field_bool("hide_info");
            vd.opt_field_validated_blocks("triggered_outfit", validate_triggered_outfit);
            // TODO: is this only useful when animation is prisondungeon ?
            vd.opt_field_bool("override_imprisonment_visuals");
            vd.warn_remaining();
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptedTrigger {
    key: Token,
    block: Block,
}

impl ScriptedTrigger {
    fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptedEffect {
    key: Token,
    block: Block,
}

impl ScriptedEffect {
    fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }
}
