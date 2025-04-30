use crate::block::{Block, BV};
use crate::ck3::validate::{
    validate_theme_background, validate_theme_effect_2d, validate_theme_header_background,
    validate_theme_icon, validate_theme_sound, validate_theme_transition,
};
use crate::context::ScopeContext;
use crate::data::events::Event;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, warn, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target, validate_trigger};
use crate::validate::{
    validate_ai_chance, validate_duration, validate_modifiers_with_base, ListType,
};
use crate::validator::Validator;

const EVENT_TYPES: &[&str] = &[
    "letter_event",
    "character_event",
    "court_event",
    "duel_event",
    "fullscreen_event",
    "activity_event",
];

pub fn get_event_scope(key: &Token, block: &Block) -> (Scopes, Token) {
    if let Some(token) = block.get_field_value("scope") {
        (Scopes::from_snake_case(token.as_str()).unwrap_or(Scopes::non_primitive()), token.clone())
    } else {
        (Scopes::Character, key.clone())
    }
}

pub fn validate_event(event: &Event, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(&event.block, data);

    let mut tooltipped_immediate = Tooltipped::Past;
    let mut tooltipped = Tooltipped::Yes;
    if event.key.starts_with("debug.") || event.block.field_value_is("hidden", "yes") {
        // Suppress missing-localization messages
        tooltipped_immediate = Tooltipped::No;
        tooltipped = Tooltipped::No;
    }

    let evtype = event.block.get_field_value("type").map_or("character_event", |t| t.as_str());
    if evtype == "empty" {
        let msg = "`type = empty` has been replaced by `scope = none`";
        let errloc = vd.field_value("type").unwrap();
        err(ErrorKey::Validation).msg(msg).loc(errloc).push();
    } else {
        vd.field_choice("type", EVENT_TYPES);
    }

    // TODO: Should be an `Item::WidgetName` but widget name processing currently doesn't catch
    // subwidgets with names.
    vd.field_value("window");

    if let Some(token) = vd.field_value("scope") {
        if Scopes::from_snake_case(token.as_str()).is_none() {
            warn(ErrorKey::Scopes).msg("unknown scope type").loc(token).push();
        }
    }

    // "dlc or mod this event comes from"
    vd.field_item("content_source", Item::Dlc);

    vd.field_bool("hidden");
    vd.field_bool("major");
    vd.field_validated_block("major_trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });

    vd.field_validated_block("immediate", |b, data| {
        validate_effect(b, data, sc, tooltipped_immediate);
    });
    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    vd.field_validated_block("on_trigger_fail", |b, data| {
        validate_effect(b, data, sc, Tooltipped::No);
    });
    vd.field_validated_block_sc("weight_multiplier", sc, validate_modifiers_with_base);

    vd.field_validated_sc("title", sc, validate_desc);
    vd.field_validated_sc("desc", sc, validate_desc);

    if evtype == "letter_event" {
        vd.field_validated_sc("opening", sc, validate_desc);
        vd.req_field("sender");
        vd.field_validated_sc("sender", sc, validate_portrait);
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
            validate_portrait(bv, data, sc);
        });
        vd.field_validated("right_portrait", |bv, data| {
            validate_portrait(bv, data, sc);
        });
        vd.field_validated("center_portrait", |bv, data| {
            validate_portrait(bv, data, sc);
        });
    }
    vd.field_validated("lower_left_portrait", |bv, data| {
        validate_portrait(bv, data, sc);
    });
    vd.field_validated("lower_center_portrait", |bv, data| {
        validate_portrait(bv, data, sc);
    });
    vd.field_validated("lower_right_portrait", |bv, data| {
        validate_portrait(bv, data, sc);
    });
    // TODO: check that artifacts are not in the same position as a character
    vd.multi_field_validated_block_sc("artifact", sc, validate_artifact);
    vd.field_validated_block_sc("court_scene", sc, validate_court_scene);
    if let Some(token) = vd.field_value("theme") {
        data.verify_exists(Item::EventTheme, token);
        data.validate_call(Item::EventTheme, token, &event.block, sc);
    }
    // TODO: warn if more than one of each is defined with no trigger
    if evtype == "court_event" {
        vd.advice_field("override_background", "not needed for court_event");
    } else {
        vd.multi_field_validated_sc("override_background", sc, validate_theme_background);
    }
    vd.multi_field_validated_sc("override_icon", sc, validate_theme_icon);
    vd.multi_field_validated_sc("override_header_background", sc, validate_theme_header_background);
    vd.multi_field_validated_block_sc("override_sound", sc, validate_theme_sound);
    vd.multi_field_validated_block_sc("override_transition", sc, validate_theme_transition);
    vd.multi_field_validated_sc("override_effect_2d", sc, validate_theme_effect_2d);
    // Note: override_environment seems to be unused, and themes defined in
    // common/event_themes don't have environments. So I left it out even though
    // it's in the docs.

    if !event.block.get_field_bool("hidden").unwrap_or(false) {
        vd.req_field("option");
    }
    let mut has_options = false;
    vd.multi_field_validated_block("option", |block, data| {
        has_options = true;
        validate_event_option(block, data, sc, tooltipped);
    });

    vd.field_validated_key_block("after", |key, block, data| {
        if !has_options {
            let msg = "`after` effect will not run if there are no `option` blocks";
            let info = "you can put it in `immediate` instead";
            err(ErrorKey::Logic).msg(msg).info(info).loc(key).push();
        }
        validate_effect(block, data, sc, tooltipped);
    });
    vd.field_validated_block_sc("cooldown", sc, validate_duration);
    vd.field_value("soundeffect"); // TODO
    vd.field_bool("orphan");
    // TODO: validate widget
    vd.field("widget");
    vd.field_block("widgets");
}

fn validate_event_option(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);
    vd.multi_field_validated("name", |bv, data| match bv {
        BV::Value(t) => {
            data.verify_exists(Item::Localization, t);
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
    vd.multi_field_item("trait", Item::Trait);
    vd.multi_field_item("skill", Item::Skill);

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

    // undocumented
    vd.field_item("clicksound", Item::Sound);

    validate_effect_internal(
        &Lowercase::new_unchecked("option"),
        ListType::None,
        block,
        data,
        sc,
        &mut vd,
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
    vd.multi_field_validated_block("roles", |b, data| {
        for (key, bv) in b.iter_assignments_and_definitions_warn() {
            match bv {
                BV::Block(block) => {
                    validate_target(key, data, sc, Scopes::Character);
                    let mut vd = Validator::new(block, data);
                    vd.req_field_one_of(&["group", "role"]);
                    vd.field_item("group", Item::CourtSceneGroup);
                    vd.field_item("role", Item::CourtSceneRole);
                    vd.field_item("animation", Item::PortraitAnimation);
                    vd.multi_field_validated_block("triggered_animation", |b, data| {
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
    vd.field_validated_value("animation", |_, mut vd| {
        if !vd.maybe_item(Item::PortraitAnimation) && vd.maybe_item(Item::ScriptedAnimation) {
            let msg = format!(
                "portrait animation {vd} not defined in {}",
                Item::PortraitAnimation.path()
            );
            let info = format!("Did you mean `scripted_animation = {vd}`?");
            warn(ErrorKey::MissingItem).strong().msg(msg).info(info).loc(vd).push();
        } else {
            vd.item(Item::PortraitAnimation);
        }
    });
    vd.field_validated_value("scripted_animation", |_, mut vd| {
        if !vd.maybe_item(Item::ScriptedAnimation) && vd.maybe_item(Item::PortraitAnimation) {
            let msg = format!(
                "scripted animation {vd} not defined in {}",
                Item::ScriptedAnimation.path()
            );
            let info = format!("Did you mean `animation = {vd}`?");
            warn(ErrorKey::MissingItem).strong().msg(msg).info(info).loc(vd).push();
        } else {
            vd.item(Item::ScriptedAnimation);
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
    vd.field_item("camera", Item::PortraitCamera);
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
        BV::Value(t) => {
            validate_target(t, data, sc, Scopes::Character);
        }
        BV::Block(b) => {
            let mut vd = Validator::new(b, data);

            vd.req_field("character");
            vd.field_target("character", sc, Scopes::Character);
            vd.field_validated_block("trigger", |b, data| {
                validate_trigger(b, data, sc, Tooltipped::No);
            });
            validate_animations(&mut vd);
            vd.multi_field_validated_block("triggered_animation", |b, data| {
                validate_triggered_animation(b, data, sc);
            });
            vd.field_list("outfit_tags"); // TODO
            vd.field_bool("remove_default_outfit");
            vd.field_bool("hide_info");
            vd.multi_field_validated_block("triggered_outfit", |b, data| {
                validate_triggered_outfit(b, data, sc);
            });
            vd.field_item("camera", Item::PortraitCamera);

            // TODO: is this only useful when animation is prisondungeon ?
            vd.field_bool("override_imprisonment_visuals");
            vd.field_bool("animate_if_dead");
        }
    }
}
