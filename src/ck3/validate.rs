//! Validation functions that are useful for more than one data module in ck3.

use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, fatal, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target, validate_target_ok_this, validate_trigger};
use crate::validate::{validate_color, validate_modifiers_with_base, validate_scope_chain};
use crate::validator::Validator;

pub fn validate_theme_background(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BV::Value(token) => {
            data.verify_exists(Item::EventBackground, token);
            let block = Block::new(token.loc);
            data.validate_call(Item::EventBackground, token, &block, sc);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);

            vd.field_validated_block("trigger", |b, data| {
                validate_trigger(b, data, sc, Tooltipped::No);
            });
            if vd.field_value("event_background").is_some() {
                let msg = "`event_background` now causes a crash. It has been replaced by `reference` since 1.9";
                fatal(ErrorKey::Crash)
                    .msg(msg)
                    .loc(block.get_key("event_background").unwrap())
                    .push();
            }
            vd.req_field("reference");
            if let Some(token) = vd.field_value("reference") {
                data.verify_exists(Item::EventBackground, token);
                data.validate_call(Item::EventBackground, token, block, sc);
            }
        }
    }
}

pub fn validate_theme_header_background(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::File, token),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);

            vd.field_validated_block("trigger", |b, data| {
                validate_trigger(b, data, sc, Tooltipped::No);
            });
            vd.req_field("reference");
            if let Some(token) = vd.field_value("reference") {
                data.verify_exists(Item::File, token);
            }
        }
    }
}

pub fn validate_theme_icon(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    vd.field_item("reference", Item::File);
}

pub fn validate_theme_sound(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    vd.field_item("reference", Item::Sound);
}

pub fn validate_theme_transition(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    if let Some(token) = vd.field_value("reference") {
        data.verify_exists(Item::EventTransition, token);
        data.validate_call(Item::EventTransition, token, block, sc);
    }
}

pub fn validate_theme_effect_2d(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_trigger(b, data, sc, Tooltipped::No);
    });
    if let Some(token) = vd.field_value("reference") {
        data.verify_exists(Item::EventEffect2d, token);
    }
}

/// Camera colors must be hsv, and value can be > 1
pub fn validate_camera_color(block: &Block, data: &Everything) {
    let mut count = 0;
    // Get the color tag, as in color = hsv { 0.5 1.0 1.0 }
    let tag = block.tag.as_deref().map_or("rgb", Token::as_str);
    if tag != "hsv" {
        let msg = "camera colors should be in hsv";
        warn(ErrorKey::Colors).msg(msg).loc(block).push();
        validate_color(block, data);
        return;
    }

    for item in block.iter_items() {
        if let Some(t) = item.get_value() {
            t.check_number();
            if let Some(f) = t.get_number() {
                if count <= 1 && !(0.0..=1.0).contains(&f) {
                    let msg = "h and s values should be between 0.0 and 1.0";
                    err(ErrorKey::Colors).msg(msg).loc(t).push();
                }
            } else {
                let msg = "expected hsv value";
                err(ErrorKey::Colors).msg(msg).loc(t).push();
            }
            count += 1;
        }
    }
    if count != 3 {
        let msg = "expected 3 color values";
        err(ErrorKey::Colors).msg(msg).loc(block).push();
    }
}

pub fn validate_cost(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value("gold", sc);
    vd.field_script_value("influence", sc);
    vd.field_script_value("prestige", sc);
    vd.field_script_value("piety", sc);
    vd.field_bool("round");
}

pub fn validate_cost_with_renown(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value("gold", sc);
    vd.field_script_value("influence", sc);
    vd.field_script_value("prestige", sc);
    vd.field_script_value("piety", sc);
    vd.field_script_value("renown", sc);
    vd.field_bool("round");
}

pub fn validate_traits(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("virtues", validate_virtues_sins);
    vd.field_validated_block("sins", validate_virtues_sins);
}

pub fn validate_virtues_sins(block: &Block, data: &Everything) {
    // Can be single tokens ("wrathful") or assignments ("wrathful = 3")
    // or even wrathful = { scale = 2 weight = 2 } whatever that means
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.verify_exists(Item::Trait, token);
    }
    vd.unknown_value_fields(|key, value| {
        data.verify_exists(Item::Trait, key);
        value.expect_number();
    });
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::Trait, key);
        let mut vd = Validator::new(block, data);
        vd.field_numeric("scale");
        vd.field_numeric("weight");
    });
}

pub fn validate_compare_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    // `value` and `factor` are evaluated in the scope created by `target`
    sc.open_builder();
    let mut valid_target = false;
    vd.field_validated_value("target", |_, mut vd| {
        valid_target = validate_scope_chain(vd.value(), data, sc, false);
        vd.accept();
    });
    sc.finalize_builder();
    if valid_target {
        vd.field_script_value("value", sc);
        vd.field_script_value("factor", sc);
    } else {
        vd.field("value");
        vd.field("factor");
    }
    sc.close();

    vd.fields_script_value("multiplier", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_script_value("step", sc); // What does this do?
    vd.field_script_value("offset", sc); // What does this do?
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_opinion_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    vd.req_field("opinion_target");
    if let Some(target) = vd.field_value("opinion_target") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    vd.field_script_value("multiplier", sc);
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_script_value("step", sc); // What does this do?
    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_ai_value_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    // TODO: verify that this actually works. It's only used 1 time in vanilla.
    vd.field_validated_block("dread_modified_ai_boldness", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("dreaded_character");
        vd.req_field("value");
        vd.field_target_ok_this("dreaded_character", sc, Scopes::Character);
        vd.field_script_value("value", sc);
    });
    vd.field_script_value("ai_boldness", sc);
    vd.field_script_value("ai_compassion", sc);
    vd.field_script_value("ai_energy", sc);
    vd.field_script_value("ai_greed", sc);
    vd.field_script_value("ai_honor", sc);
    vd.field_script_value("ai_rationality", sc);
    vd.field_script_value("ai_sociability", sc);
    vd.field_script_value("ai_vengefulness", sc);
    vd.field_script_value("ai_zeal", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_compatibility_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    if let Some(target) = vd.field_value("compatibility_target") {
        validate_target_ok_this(target, data, sc, Scopes::Character);
    }
    vd.field_script_value("multiplier", sc);
    //vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_validated_block("trigger", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
}

pub fn validate_activity_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_target("object", sc, Scopes::Activity);
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_scheme_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_target("object", sc, Scopes::Scheme);
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_random_traits_list(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value("count", sc);
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::Trait, key);
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    });
}

pub fn validate_random_culture(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.unknown_block_fields(|key, block| {
        validate_target(key, data, sc, Scopes::Culture);
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    });
}

pub fn validate_random_faith(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.unknown_block_fields(|key, block| {
        validate_target(key, data, sc, Scopes::Faith);
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    });
}

pub fn validate_maa_stats(vd: &mut Validator) {
    vd.field_numeric("pursuit");
    vd.field_numeric("screen");
    vd.field_numeric("damage");
    vd.field_numeric("toughness");
    vd.field_numeric("siege_value");
}

pub fn validate_portrait_modifier_overrides(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, value| {
        data.verify_exists(Item::PortraitModifierGroup, key);
        if !data.item_has_property(Item::PortraitModifierGroup, key.as_str(), value.as_str()) {
            let msg = format!("portrait modifier group {key} does not have the modifier {value}");
            err(ErrorKey::MissingItem).msg(msg).loc(value).push();
        }
    });
}
