use crate::block::validator::Validator;
use crate::block::{Block, Comparator, Eq::*, BV};
use crate::context::ScopeContext;
use crate::data::effect_localization::EffectLocalization;
use crate::desc::validate_desc;
use crate::effect_validation::{validate_effect_block, validate_effect_bv, validate_effect_value};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{advice_info, error, error_info, warn, warn_info, ErrorKey};
use crate::scopes::{scope_iterator, Scopes};
use crate::scriptvalue::validate_scriptvalue;
use crate::tables::effects::{scope_effect, Effect};
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_normal_trigger, validate_target, validate_target_ok_this};
use crate::validate::{
    precheck_iterator_fields, validate_days_weeks_months_years, validate_ifelse_sequence,
    validate_inside_iterator, validate_iterator_fields, validate_modifiers,
    validate_optional_duration, validate_scope_chain, validate_scripted_modifier_call, ListType,
};

pub fn validate_normal_effect(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
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
    mut tooltipped: Tooltipped,
) {
    if caller == "if"
        || caller == "else_if"
        || caller == "else"
        || caller == "while"
        || list_type != ListType::None
    {
        vd.field_validated_key_block("limit", |key, block, data| {
            if caller == "else" {
                let msg = "`else` with a `limit` does work, but may indicate a mistake";
                let info = "normally you would use `else_if` instead.";
                advice_info(key, ErrorKey::IfElse, msg, info);
            }
            validate_normal_trigger(block, data, sc, tooltipped);
        });
    } else {
        vd.ban_field("limit", || "if/else_if or lists");
    }

    validate_iterator_fields(caller, list_type, data, sc, &mut vd, &mut tooltipped);

    if list_type != ListType::None {
        validate_inside_iterator(caller, list_type, block, data, sc, &mut vd, tooltipped);
    }

    validate_ifelse_sequence(block, "if", "else_if", "else");

    vd.allow_qeq(true);
    'outer: for (key, cmp, bv) in vd.unknown_fields_cmp() {
        if let Some(effect) = data.get_effect(key) {
            match bv {
                BV::Value(token) => {
                    if !effect.macro_parms().is_empty() {
                        error(token, ErrorKey::Macro, "expected macro arguments");
                    } else if !token.is("yes") {
                        warn(token, ErrorKey::Validation, "expected just effect = yes");
                    }
                    effect.validate_call(key, data, sc, tooltipped);
                }
                BV::Block(block) => {
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
                            vd.req_field(parm);
                            if let Some(token) = vd.field_value(parm) {
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

        if let Some(modifier) = data.scripted_modifiers.get(key.as_str()) {
            if caller != "random" && caller != "random_list" && caller != "duel" {
                let msg = "cannot use scripted modifier here";
                error(key, ErrorKey::Validation, msg);
                continue;
            }
            validate_scripted_modifier_call(key, bv, modifier, data, sc);
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
                Effect::Boolean => {
                    if let Some(token) = bv.expect_value() {
                        validate_target(token, data, sc, Scopes::Bool);
                    }
                }
                Effect::Integer => {
                    if let Some(token) = bv.expect_value() {
                        token.expect_integer();
                    }
                }
                Effect::ScriptValue | Effect::NonNegativeValue => {
                    if let Some(token) = bv.get_value() {
                        if let Some(number) = token.get_number() {
                            if matches!(effect, Effect::NonNegativeValue) && number < 0.0 {
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
                    validate_scriptvalue(bv, data, sc);
                }
                Effect::Scope(outscopes) => {
                    if let Some(token) = bv.expect_value() {
                        validate_target(token, data, sc, outscopes);
                    }
                }
                Effect::ScopeOkThis(outscopes) => {
                    if let Some(token) = bv.expect_value() {
                        validate_target_ok_this(token, data, sc, outscopes);
                    }
                }
                Effect::Item(itype) => {
                    if let Some(token) = bv.expect_value() {
                        data.verify_exists(itype, token);
                    }
                }
                Effect::ScopeOrItem(outscopes, itype) => {
                    if let Some(token) = bv.expect_value() {
                        if !data.item_exists(itype, token.as_str()) {
                            validate_target(token, data, sc, outscopes);
                        }
                    }
                }
                Effect::Target(key, outscopes) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        vd.set_case_sensitive(false);
                        vd.req_field(key);
                        vd.field_target(key, sc, outscopes);
                    }
                }
                Effect::TargetValue(key, outscopes, valuekey) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        vd.set_case_sensitive(false);
                        vd.req_field(key);
                        vd.req_field(valuekey);
                        vd.field_target(key, sc, outscopes);
                        vd.field_script_value(valuekey, sc);
                    }
                }
                Effect::ItemTarget(ikey, itype, tkey, outscopes) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        vd.set_case_sensitive(false);
                        vd.field_item(ikey, itype);
                        vd.field_target(tkey, sc, outscopes);
                    }
                }
                Effect::ItemValue(key, itype) => {
                    if let Some(block) = bv.expect_block() {
                        let mut vd = Validator::new(block, data);
                        vd.set_case_sensitive(false);
                        vd.req_field(key);
                        vd.req_field("value");
                        vd.field_item(key, itype);
                        vd.field_script_value("value", sc);
                    }
                }
                Effect::Choice(choices) => {
                    if let Some(token) = bv.expect_value() {
                        if !choices.contains(&token.as_str()) {
                            let msg = format!("expected one of {}", choices.join(", "));
                            error(token, ErrorKey::Choice, &msg);
                        }
                    }
                }
                Effect::Desc => validate_desc(bv, data, sc),
                Effect::Timespan => {
                    if let Some(block) = bv.expect_block() {
                        validate_days_weeks_months_years(block, data, sc);
                    }
                }
                Effect::AddModifier => {
                    let visible = key.is("add_character_modifier")
                        || key.is("add_house_modifier")
                        || key.is("add_dynasty_modifier")
                        || key.is("add_county_modifier");
                    match bv {
                        BV::Value(token) => {
                            data.verify_exists(Item::Modifier, token);
                            if visible {
                                data.verify_exists(Item::Localization, token);
                            }
                            data.database.validate_property_use(
                                Item::Modifier,
                                token,
                                data,
                                key,
                                "",
                            );
                        }
                        BV::Block(block) => {
                            let mut vd = Validator::new(block, data);
                            vd.set_case_sensitive(false);
                            vd.req_field("modifier");
                            if let Some(token) = vd.field_value("modifier") {
                                data.verify_exists(Item::Modifier, token);
                                if visible && !block.has_key("desc") {
                                    data.verify_exists(Item::Localization, token);
                                }
                                data.database.validate_property_use(
                                    Item::Modifier,
                                    token,
                                    data,
                                    key,
                                    "",
                                );
                            }
                            vd.field_validated_sc("desc", sc, validate_desc);
                            validate_optional_duration(&mut vd, sc);
                        }
                    }
                }
                Effect::VB(v) => {
                    if let Some(block) = bv.expect_block() {
                        validate_effect_block(v, key, block, data, sc, tooltipped);
                    }
                }
                Effect::VV(v) => {
                    if let Some(token) = bv.expect_value() {
                        validate_effect_value(v, key, token, data, sc, tooltipped);
                    }
                }
                Effect::VBv(v) => validate_effect_bv(v, key, bv, data, sc, tooltipped),
                Effect::ControlOrLabel => match bv {
                    BV::Value(t) => data.verify_exists(Item::Localization, t),
                    BV::Block(b) => validate_effect_control(
                        &key.as_str().to_lowercase(),
                        b,
                        data,
                        sc,
                        tooltipped,
                    ),
                },
                Effect::Control => {
                    if let Some(block) = bv.expect_block() {
                        validate_effect_control(
                            &key.as_str().to_lowercase(),
                            block,
                            data,
                            sc,
                            tooltipped,
                        );
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
                    if let Some(b) = bv.expect_block() {
                        precheck_iterator_fields(ltype, b, data, sc);
                    }
                    sc.open_scope(outscope, key.clone());
                    if let Some(b) = bv.get_block() {
                        let vd = Validator::new(b, data);
                        validate_effect(it_name.as_str(), ltype, b, data, sc, vd, tooltipped);
                    }
                    sc.close();
                    continue;
                }
            }
        }

        // Check if it's a target = { target_scope } block.
        sc.open_builder();
        if validate_scope_chain(key, data, sc, matches!(cmp, Comparator::Equals(Question))) {
            sc.finalize_builder();
            if key.starts_with("flag:") {
                let msg = "as of 1.9, flag literals can not be used on the left-hand side";
                error(key, ErrorKey::Scopes, msg);
            }
            if let Some(block) = bv.expect_block() {
                validate_normal_effect(block, data, sc, tooltipped);
            }
        }
        sc.close();
    }
}

pub fn validate_effect_control(
    caller: &str,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);

    if caller == "if" || caller == "else_if" {
        vd.req_field_warn("limit");
    }

    if caller == "custom_description"
        || caller == "custom_description_no_bullet"
        || caller == "custom_tooltip"
        || caller == "custom_label"
    {
        vd.req_field("text");
        if caller == "custom_tooltip" || caller == "custom_label" {
            vd.field_item("text", Item::Localization);
        } else if let Some(token) = vd.field_value("text") {
            data.verify_exists(Item::EffectLocalization, token);
            if let Some((key, block)) = data
                .database
                .get_key_block(Item::EffectLocalization, token.as_str())
            {
                EffectLocalization::validate_use(key, block, data, token, tooltipped);
            }
        }
        vd.field_target_ok_this("subject", sc, Scopes::non_primitive());
        tooltipped = Tooltipped::No;
    } else {
        vd.ban_field("text", || "`custom_description` or `custom_tooltip`");
        vd.ban_field("subject", || "`custom_description` or `custom_tooltip`");
    }

    if caller == "custom_description" || caller == "custom_description_no_bullet" {
        vd.field_target_ok_this("object", sc, Scopes::non_primitive());
        vd.field_script_value("value", sc);
    } else {
        vd.ban_field("object", || "`custom_description`");
        vd.ban_field("value", || "`custom_description`");
    }

    if caller == "hidden_effect" || caller == "hidden_effect_new_object" {
        tooltipped = Tooltipped::No;
    }

    if caller == "random" {
        vd.req_field("chance");
        vd.field_script_value("chance", sc);
    } else {
        vd.ban_field("chance", || "`random`");
    }

    if caller == "send_interface_message" || caller == "send_interface_toast" {
        vd.field_item("type", Item::Message);
        vd.field_validated_sc("title", sc, validate_desc);
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_validated_sc("tooltip", sc, validate_desc);
        let icon_scopes =
            Scopes::Character | Scopes::LandedTitle | Scopes::Artifact | Scopes::Faith;
        if let Some(token) = vd.field_value("left_icon") {
            validate_target_ok_this(token, data, sc, icon_scopes);
        }
        if let Some(token) = vd.field_value("right_icon") {
            validate_target_ok_this(token, data, sc, icon_scopes);
        }
        if let Some(token) = vd.field_value("goto") {
            let msg = "`goto` was removed from interface messages in 1.9";
            warn(token, ErrorKey::Removed, msg);
        }
    }

    if caller == "while" {
        if !(block.has_key("limit") || block.has_key("count")) {
            let msg = "`while` needs one of `limit` or `count`";
            warn(block, ErrorKey::Validation, msg);
        }

        vd.field_script_value("count", sc);
    } else {
        vd.ban_field("count", || "`while` and `any_` lists");
    }

    if caller == "random" || caller == "random_list" || caller == "duel" {
        validate_modifiers(&mut vd, sc);
    } else {
        vd.ban_field("modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("compare_modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("opinion_modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("ai_value_modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("compatibility", || "`random`, `random_list` or `duel`");
    }

    if caller == "random_list" || caller == "duel" {
        vd.field_validated_block("trigger", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_bool("show_chance");
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_script_value("min", sc); // used in vanilla
        vd.field_script_value("max", sc); // used in vanilla
                                          // TODO: check if "max" also works
    } else {
        if caller != "option" {
            vd.ban_field("trigger", || "`random_list` or `duel`");
        }
        vd.ban_field("show_chance", || "`random_list` or `duel`");
    }

    validate_effect(caller, ListType::None, block, data, sc, vd, tooltipped);
}
