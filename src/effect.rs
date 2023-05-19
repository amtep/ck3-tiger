use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, Comparator};
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::desc::validate_desc;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn, warn_info};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, Scopes};
use crate::tables::effects::{scope_effect, ControlEffect, Effect};
use crate::trigger::{
    validate_normal_trigger, validate_target, validate_trigger, validate_trigger_key_bv,
};
use crate::validate::{
    validate_ai_value_modifier, validate_compare_modifier, validate_compatibility_modifier,
    validate_days_weeks_months_years, validate_inside_iterator, validate_iterator_fields,
    validate_opinion_modifier, validate_prefix_reference, ListType,
};

pub fn validate_normal_effect(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let vd = Validator::new(block, data);
    validate_effect("", ListType::None, block, data, sc, vd, tooltipped);
}

const CALLERS_ALLOW_LIMIT: [&str; 4] = ["if", "else_if", "else", "while"];

pub fn validate_effect<'a>(
    caller: &str,
    list_type: ListType,
    block: &Block,
    data: &'a Everything,
    sc: &mut ScopeContext,
    mut vd: Validator<'a>,
    mut tooltipped: bool,
) {
    if CALLERS_ALLOW_LIMIT.contains(&caller) || list_type != ListType::None {
        if let Some(b) = vd.field_block("limit") {
            validate_normal_trigger(b, data, sc, tooltipped);
        }
    } else {
        vd.ban_field("limit", || "if/else_if or lists");
    }

    validate_iterator_fields(caller, list_type, sc, &mut vd, &mut tooltipped);

    if list_type != ListType::None {
        validate_inside_iterator(
            caller,
            &list_type.to_string(),
            block,
            data,
            sc,
            &mut vd,
            tooltipped,
        );
    }

    // custom_description_no_bullet is folded into custom_description by the caller
    if caller == "custom_description" || caller == "custom_tooltip" {
        if caller == "custom_tooltip" {
            vd.field_value_item("text", Item::Localization);
        } else {
            vd.field_value_item("text", Item::TriggerLocalization);
        }
        if let Some(token) = vd.field_value("subject") {
            validate_target(token, data, sc, Scopes::non_primitive());
        }
    } else {
        vd.ban_field("text", || "`custom_description` or `custom_tooltip`");
        vd.ban_field("subject", || "`custom_description` or `custom_tooltip`");
    }

    if caller == "custom_description" {
        if let Some(token) = vd.field_value("object") {
            validate_target(token, data, sc, Scopes::non_primitive());
        }
        vd.field_script_value("value", sc);
    } else {
        vd.ban_field("object", || "`custom_description`");
        vd.ban_field("value", || "`custom_description`");
    }

    if caller == "while" {
        vd.field_script_value("count", sc);
    } else {
        vd.ban_field("count", || "`while`");
    }

    if caller == "random" {
        vd.req_field("chance");
        vd.field_script_value("chance", sc);
    } else {
        vd.ban_field("chance", || "`random`");
    }

    if caller == "random" || caller == "random_list" {
        vd.field_validated_blocks("modifier", |b, data| {
            validate_trigger("modifier", false, b, data, sc, false);
        });
        vd.field_validated_blocks_sc("compare_modifier", sc, validate_compare_modifier);
        vd.field_validated_blocks_sc("opinion_modifier", sc, validate_opinion_modifier);
        vd.field_validated_blocks_sc("ai_value_modifier", sc, validate_ai_value_modifier);
        vd.field_validated_blocks_sc(
            "compatibility_modifier",
            sc,
            validate_compatibility_modifier,
        );
    } else {
        vd.ban_field("modifier", || "`random` or `random_list`");
        vd.ban_field("compare_modifier", || "`random` or `random_list`");
        vd.ban_field("opinion_modifier", || "`random` or `random_list`");
        vd.ban_field("ai_value_modifier", || "`random` or `random_list`");
        vd.ban_field("compatibility", || "`random` or `random_list`");
    }

    if caller == "random_list" {
        if let Some(b) = vd.field_block("trigger") {
            validate_normal_trigger(b, data, sc, false);
        }
        vd.field_bool("show_chance");
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_script_value("min", sc); // used in vanilla
                                          // TODO: check if "max" also works
    } else {
        if caller != "option" {
            vd.ban_field("trigger", || "`random_list`");
        }
        vd.ban_field("show_chance", || "`random_list`");
    }

    'outer: for (key, bv) in vd.unknown_keys() {
        if let Some(effect) = data.get_effect(key) {
            match bv {
                BlockOrValue::Token(token) => {
                    if !effect.macro_parms().is_empty() {
                        error(token, ErrorKey::Macro, "expected macro arguments");
                    } else if !token.is("yes") {
                        warn(token, ErrorKey::Validation, "expected just effect = yes");
                    }
                    effect.validate_call(key, data, sc, tooltipped);
                }
                BlockOrValue::Block(block) => {
                    let parms = effect.macro_parms();
                    if parms.is_empty() {
                        error_info(
                            block,
                            ErrorKey::Macro,
                            "effect does not need macro arguments",
                            "you can just use it as effect = yes",
                        );
                    } else {
                        let mut vec = Vec::new();
                        let mut vd = Validator::new(block, data);
                        for parm in &parms {
                            vd.req_field(parm.as_str());
                            if let Some(token) = vd.field_value(parm.as_str()) {
                                vec.push(token.clone());
                            } else {
                                continue 'outer;
                            }
                        }
                        let args = parms.into_iter().zip(vec.into_iter()).collect();
                        effect.validate_macro_expansion(key, args, data, sc, tooltipped);
                    }
                }
            }
            continue;
        }

        if let Some((inscopes, effect)) = scope_effect(key, data) {
            sc.expect(inscopes, key);
            match effect {
                Effect::Yes => {
                    if let Some(token) = bv.expect_value() {
                        if !token.is("yes") {
                            let msg = format!("expected just `{key} = yes`");
                            warn(token, ErrorKey::Validation, &msg);
                        }
                    }
                }
                Effect::Bool => {
                    if let Some(token) = bv.expect_value() {
                        validate_target(token, data, sc, Scopes::Bool);
                    }
                }
                Effect::Integer => {
                    if let Some(token) = bv.expect_value() {
                        if token.as_str().parse::<i32>().is_err() {
                            warn(token, ErrorKey::Validation, "expected integer");
                        }
                    }
                }
                Effect::Value | Effect::ScriptValue | Effect::NonNegativeValue => {
                    if let Some(token) = bv.get_value() {
                        if let Ok(number) = token.as_str().parse::<i32>() {
                            if effect == Effect::NonNegativeValue && number < 0 {
                                if key.is("add_gold") {
                                    let msg = "add_gold does not take negative numbers";
                                    let info = "try remove_short_term_gold instead";
                                    warn_info(token, ErrorKey::Range, msg, info);
                                } else {
                                    let msg = format!("{key} does not take negative numbers");
                                    warn(token, ErrorKey::Range, &msg);
                                }
                            }
                        }
                    }
                    ScriptValue::validate_bv(bv, data, sc);
                }
                Effect::Scope(outscopes) => {
                    if let Some(token) = bv.expect_value() {
                        validate_target(token, data, sc, outscopes);
                    }
                }
                Effect::Item(itype) => {
                    if let Some(token) = bv.expect_value() {
                        data.verify_exists(itype, token);
                    }
                }
                Effect::Target(key, outscopes) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        vd.req_field(key);
                        if let Some(token) = vd.field_value(key) {
                            validate_target(token, data, sc, outscopes);
                        }
                    }
                }
                Effect::TargetValue(key, outscopes, valuekey) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        vd.req_field(key);
                        vd.req_field(valuekey);
                        if let Some(token) = vd.field_value(key) {
                            validate_target(token, data, sc, outscopes);
                        }
                        if let Some(bv) = vd.field(valuekey) {
                            ScriptValue::validate_bv(bv, data, sc);
                        }
                    }
                }
                Effect::ItemTarget(ikey, itype, tkey, outscopes) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        if let Some(token) = vd.field_value(ikey) {
                            data.verify_exists(itype, token);
                        }
                        if let Some(token) = vd.field_value(tkey) {
                            validate_target(token, data, sc, outscopes);
                        }
                    }
                }
                Effect::ItemValue(key, itype) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        vd.req_field(key);
                        vd.req_field("value");
                        if let Some(token) = vd.field_value(key) {
                            data.verify_exists(itype, token);
                        }
                        if let Some(bv) = vd.field("value") {
                            ScriptValue::validate_bv(bv, data, sc);
                        }
                    }
                }
                Effect::Desc => validate_desc(bv, data, sc),
                Effect::Gender => {
                    if let Some(token) = bv.expect_value() {
                        if !(token.is("male") || token.is("female") || token.is("random")) {
                            let msg = "expected `male`, `female`, or `random`";
                            warn(token, ErrorKey::Validation, msg);
                        }
                    }
                }
                Effect::Timespan => {
                    if let Some(block) = bv.expect_block() {
                        validate_days_weeks_months_years(block, data, sc);
                    }
                }
                Effect::Special(_special) => (), // TODO
                Effect::Control(ControlEffect::CustomTooltip) => match bv {
                    BlockOrValue::Token(t) => data.verify_exists(Item::Localization, t),
                    BlockOrValue::Block(b) => validate_effect_control(
                        ControlEffect::CustomTooltip,
                        b,
                        data,
                        sc,
                        tooltipped,
                    ),
                },
                Effect::Control(control) => {
                    if let Some(block) = bv.expect_block() {
                        validate_effect_control(control, block, data, sc, tooltipped);
                    }
                }
                Effect::Removed(version, explanation) => {
                    let msg = format!("`{key}` was removed in {version}");
                    warn_info(key, ErrorKey::Removed, &msg, explanation);
                }
                Effect::Unchecked => (),
            }
            continue;
        }

        if let Some((it_type, it_name)) = key.split_once('_') {
            if it_type.is("any")
                || it_type.is("ordered")
                || it_type.is("every")
                || it_type.is("random")
            {
                if let Some((inscopes, outscope)) = scope_iterator(&it_name, data) {
                    if it_type.is("any") {
                        let msg = "cannot use `any_` lists in an effect";
                        error(key, ErrorKey::Validation, msg);
                        continue;
                    }
                    sc.expect(inscopes, key);
                    let ltype = ListType::try_from(it_type.as_str()).unwrap();
                    sc.open_scope(outscope, key.clone());
                    if let Some(b) = bv.expect_block() {
                        let vd = Validator::new(b, data);
                        validate_effect(it_name.as_str(), ltype, b, data, sc, vd, tooltipped);
                    }
                    sc.close();
                    continue;
                }
            }
        }

        // Check if it's a target = { target_scope } block.
        // The logic here is similar to logic in triggers and script values,
        // but not quite the same :(
        let part_vec = key.split('.');
        sc.open_builder();
        for (i, part) in part_vec.iter().enumerate() {
            let first = i == 0;

            if let Some((prefix, arg)) = part.split_once(':') {
                if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                    if inscopes == Scopes::None && !first {
                        let msg = format!("`{prefix}:` makes no sense except as first part");
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    sc.expect(inscopes, &prefix);
                    validate_prefix_reference(&prefix, &arg, data);
                    sc.replace(outscope, part.clone());
                } else {
                    let msg = format!("unknown prefix `{prefix}:`");
                    error(part, ErrorKey::Validation, &msg);
                    sc.close();
                    continue 'outer;
                }
            } else if part.lowercase_is("root")
                || part.lowercase_is("prev")
                || part.lowercase_is("this")
            {
                if !first {
                    let msg = format!("`{part}` makes no sense except as first part");
                    warn(part, ErrorKey::Validation, &msg);
                }
                if part.lowercase_is("root") {
                    sc.replace_root();
                } else if part.lowercase_is("prev") {
                    sc.replace_prev(part);
                } else {
                    sc.replace_this();
                }
            } else if let Some((inscopes, outscope)) = scope_to_scope(part) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{part}` makes no sense except as first part");
                    warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, part);
                sc.replace(outscope, part.clone());
            // TODO: warn if trying to use iterator or effect here
            } else {
                // TODO: this check on caller is temporary until we parse the scripted modifiers
                if caller != "random" && caller != "random_list" {
                    let msg = format!("unknown token `{part}`");
                    error(part, ErrorKey::Validation, &msg);
                }
                sc.close();
                continue 'outer;
            }
        }

        if let Some(block) = bv.expect_block() {
            validate_normal_effect(block, data, sc, tooltipped);
        }
        sc.close();
    }
}

fn validate_effect_control(
    control: ControlEffect,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let mut vd = Validator::new(block, data);
    #[allow(clippy::match_same_arms)] // They only match because they need further coding
    match control {
        ControlEffect::CustomDescription => {
            vd.req_field("text");
            validate_effect(
                "custom_description",
                ListType::None,
                block,
                data,
                sc,
                vd,
                false,
            );
        }
        ControlEffect::CustomTooltip => {
            vd.req_field("text");
            validate_effect("custom_tooltip", ListType::None, block, data, sc, vd, false);
        }
        ControlEffect::If => {
            vd.req_field_warn("limit");
            validate_effect("if", ListType::None, block, data, sc, vd, tooltipped);
        }
        ControlEffect::Else => {
            validate_effect("else", ListType::None, block, data, sc, vd, tooltipped);
        }
        ControlEffect::InterfaceMessage => {
            vd.field_value("type");
            if let Some(bv) = vd.field("title") {
                validate_desc(bv, data, sc);
            }
            if let Some(bv) = vd.field("desc") {
                validate_desc(bv, data, sc);
            }
            if let Some(bv) = vd.field("tooltip") {
                validate_desc(bv, data, sc);
            }
            // TODO: Scopes::Faith isn't documented. Verify that it actually works.
            if let Some(token) = vd.field_value("left_icon") {
                validate_target(
                    token,
                    data,
                    sc,
                    Scopes::Character | Scopes::LandedTitle | Scopes::Artifact | Scopes::Faith,
                );
            }
            if let Some(token) = vd.field_value("right_icon") {
                validate_target(
                    token,
                    data,
                    sc,
                    Scopes::Character | Scopes::LandedTitle | Scopes::Artifact | Scopes::Faith,
                );
            }
            if let Some(token) = vd.field_value("goto") {
                let msg = "`goto` was removed from interface messages in 1.9";
                warn(token, ErrorKey::Removed, msg);
            }
            validate_effect(
                "send_interface_message",
                ListType::None,
                block,
                data,
                sc,
                vd,
                true,
            );
        }
        ControlEffect::HiddenEffect => {
            validate_effect("hidden_effect", ListType::None, block, data, sc, vd, false);
        }
        ControlEffect::Random => {
            validate_effect("random", ListType::None, block, data, sc, vd, tooltipped);
        }
        ControlEffect::RandomList => {
            vd.field_integer("pick");
            vd.field_bool("unique"); // don't know what this does
            vd.field_validated_sc("desc", sc, validate_desc);
            for (key, bv) in vd.unknown_keys() {
                if f64::from_str(key.as_str()).is_err() {
                    let msg = "expected numeric value";
                    error(key, ErrorKey::Validation, msg);
                } else if let Some(block) = bv.expect_block() {
                    let vd = Validator::new(block, data);
                    validate_effect(
                        "random_list",
                        ListType::None,
                        block,
                        data,
                        sc,
                        vd,
                        tooltipped,
                    );
                }
            }
        }
        ControlEffect::ShowAsTooltip => {
            validate_effect("show_as_tooltip", ListType::None, block, data, sc, vd, true);
        }
        ControlEffect::Switch => {
            vd.req_field("trigger");
            if let Some(target) = vd.field_value("trigger") {
                // clone to avoid calling vd again while target is still borrowed
                let target = target.clone();
                for (key, bv) in vd.unknown_keys() {
                    // Pretend the switch was written as a series of trigger = key lines
                    let synthetic_bv = BlockOrValue::Token(key.clone());
                    validate_trigger_key_bv(
                        &target,
                        Comparator::Eq,
                        &synthetic_bv,
                        data,
                        sc,
                        tooltipped,
                    );

                    if let Some(block) = bv.expect_block() {
                        let vd = Validator::new(block, data);
                        validate_effect("", ListType::None, block, data, sc, vd, tooltipped);
                    }
                }
            }
        }
        ControlEffect::While => {
            if !(block.get_key("limit").is_some() || block.get_key("count").is_some()) {
                warn(
                    block,
                    ErrorKey::Validation,
                    "`while` needs one of `limit` or `count`",
                );
            }
            validate_effect("while", ListType::None, block, data, sc, vd, tooltipped);
        }
    }
}
