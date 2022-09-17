use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::desc::validate_desc;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, Scopes};
use crate::tables::effects::{scope_effect, ControlEffect, Effect};
use crate::trigger::{validate_normal_trigger, validate_target};
use crate::validate::{
    validate_inside_iterator, validate_iterator_fields, validate_prefix_reference, ListType,
};

pub fn validate_normal_effect<'a>(
    block: &Block,
    data: &'a Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let vd = Validator::new(block, data);
    validate_effect("", ListType::None, block, data, sc, vd, tooltipped);
}

pub fn validate_effect<'a>(
    caller: &str,
    list_type: ListType,
    block: &Block,
    data: &'a Everything,
    sc: &mut ScopeContext,
    mut vd: Validator<'a>,
    mut tooltipped: bool,
) {
    if let Some(b) = vd.field_block("limit") {
        if caller == "if"
            || caller == "else_if"
            || caller == "else"
            || caller == "while"
            || list_type != ListType::None
        {
            validate_normal_trigger(b, data, sc, tooltipped);
        } else {
            warn(
                block.get_key("limit").unwrap(),
                ErrorKey::Validation,
                "`limit` can only be used in if/else_if or lists",
            );
        }
    }

    if validate_iterator_fields(list_type, block, data, sc, &mut vd) {
        tooltipped = false;
    }

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

    if let Some(token) = vd.field_value("text") {
        if caller == "custom_description" {
            // TODO: verify effect localization
        } else if caller == "custom_tooltip" {
            data.verify_exists(Item::Localization, token);
        } else {
            warn(
                block.get_key("text").unwrap(),
                ErrorKey::Validation,
                "`text` can only be used in `custom_description` or `custom_tooltip`",
            );
        }
    }

    if let Some(token) = vd.field_value("subject") {
        if caller == "custom_description" || caller == "custom_tooltip" {
            validate_target(token, data, sc, Scopes::non_primitive());
        } else {
            warn(
                block.get_key("subject").unwrap(),
                ErrorKey::Validation,
                "`subject` can only be used in `custom_description` or `custom_tooltip`",
            );
        }
    }

    if let Some(token) = vd.field_value("object") {
        if caller == "custom_description" {
            validate_target(token, data, sc, Scopes::non_primitive());
        } else {
            warn(
                block.get_key("object").unwrap(),
                ErrorKey::Validation,
                "`object` can only be used in `custom_description`",
            );
        }
    }

    if let Some(bv) = vd.field("value") {
        if caller == "custom_description" {
            ScriptValue::validate_bv(bv, data, sc);
        } else {
            warn(
                block.get_key("value").unwrap(),
                ErrorKey::Validation,
                "`value` can only be used in `custom_description`",
            );
        }
    }

    if let Some(bv) = vd.field("count") {
        if caller == "while" {
            ScriptValue::validate_bv(bv, data, sc);
        } else {
            warn(
                block.get_key("count").unwrap(),
                ErrorKey::Validation,
                "`count` can only be used in `while`",
            );
        }
    }

    if let Some(bv) = vd.field("chance") {
        if caller == "random" {
            ScriptValue::validate_bv(bv, data, sc);
        } else {
            warn(
                block.get_key("chance").unwrap(),
                ErrorKey::Validation,
                "`chance` can only be used in `random`",
            );
        }
    }

    vd.field_validated_blocks("modifier", |_b, _data| {
        if caller == "random" {
            // TODO
        } else {
            warn(
                block.get_key("modifier").unwrap(),
                ErrorKey::Validation,
                "`modifier` can only be used in `random`",
            );
        }
    });

    'outer: for (key, bv) in vd.unknown_keys() {
        if let Some((inscopes, effect)) = scope_effect(key, data) {
            sc.expect(inscopes, key);
            match effect {
                Effect::Yes => {
                    if let Some(token) = bv.expect_value() {
                        if !token.is("yes") {
                            let msg = format!("expected just `{} = yes`", key);
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
                                let msg = format!("{} does not take negative numbers", key);
                                warn(token, ErrorKey::Validation, &msg);
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
                        vd.warn_remaining();
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
                        vd.warn_remaining();
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
                        vd.warn_remaining();
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
                        vd.warn_remaining();
                    }
                }
                Effect::Desc => validate_desc(bv, data, sc),
                Effect::Gender => {
                    if let Some(token) = bv.expect_value() {
                        if !(token.is("male") || token.is("female") || token.is("random")) {
                            warn(
                                token,
                                ErrorKey::Validation,
                                "expected `male`, `female`, or `random`",
                            );
                        }
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
                    let ltype = match it_type.as_str() {
                        "every" => ListType::Every,
                        "ordered" => ListType::Ordered,
                        "random" => ListType::Random,
                        _ => unreachable!(),
                    };
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

        if let Some(effect) = data.get_effect(key) {
            match bv {
                BlockOrValue::Token(token) => {
                    if !effect.macro_parms().is_empty() {
                        error(token, ErrorKey::Macro, "expected macro arguments");
                    } else if !token.is("yes") {
                        warn(token, ErrorKey::Validation, "expected just effect = yes");
                    }
                    effect.validate_call(&key.loc, data, sc, tooltipped);
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
                        effect.validate_macro_expansion(args, data, sc, tooltipped);
                        vd.warn_remaining();
                    }
                }
            }
            continue;
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
                        let msg = format!("`{}:` makes no sense except as first part", prefix);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    sc.expect(inscopes, &prefix);
                    validate_prefix_reference(&prefix, &arg, data);
                    sc.replace(outscope, part.clone());
                } else {
                    let msg = format!("unknown prefix `{}:`", prefix);
                    error(part, ErrorKey::Validation, &msg);
                    sc.close();
                    continue 'outer;
                }
            } else if part.is("root")
                || part.is("prev")
                || part.is("this")
                || part.is("ROOT")
                || part.is("PREV")
                || part.is("THIS")
            {
                if !first {
                    let msg = format!("`{}` makes no sense except as first part", part);
                    warn(part, ErrorKey::Validation, &msg);
                }
                if part.is("root") || part.is("ROOT") {
                    sc.replace_root();
                } else if part.is("prev") || part.is("PREV") {
                    sc.replace_prev(part);
                } else {
                    sc.replace_this();
                }
            } else if let Some((inscopes, outscope)) = scope_to_scope(part.as_str()) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{}` makes no sense except as first part", part);
                    warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, part);
                sc.replace(outscope, part.clone());
            // TODO: warn if trying to use iterator or effect here
            } else {
                let msg = format!("unknown token `{}`", part);
                error(part, ErrorKey::Validation, &msg);
                sc.close();
                continue 'outer;
            }
        }

        if let Some(block) = bv.expect_block() {
            validate_normal_effect(block, data, sc, tooltipped);
        }
        sc.close();
    }

    vd.warn_remaining();
}

fn validate_effect_control(
    control: ControlEffect,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let mut vd = Validator::new(block, data);
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
                validate_target(
                    token,
                    data,
                    sc,
                    Scopes::Character | Scopes::LandedTitle | Scopes::Province,
                );
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
            // TODO: need to parse modifiers first
            // validate_effect("random", ListType::None, block, data, sc, vd, false);
        }
        ControlEffect::RandomList => (), // TODO
        ControlEffect::ShowAsTooltip => {
            validate_effect("show_as_tooltip", ListType::None, block, data, sc, vd, true);
        }
        ControlEffect::Switch => (), // TODO
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
