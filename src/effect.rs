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
use crate::tables::effects::{scope_effect, Effect};
use crate::trigger::{validate_normal_trigger, validate_target, validate_trigger_key_bv};
use crate::validate::{
    validate_cooldown, validate_days_weeks_months_years, validate_inside_iterator,
    validate_iterator_fields, validate_modifiers, validate_optional_cooldown,
    validate_optional_cooldown_int, validate_prefix_reference, validate_scripted_modifier_call,
    ListType,
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

pub fn validate_effect<'a>(
    caller: &str,
    list_type: ListType,
    block: &Block,
    data: &'a Everything,
    sc: &mut ScopeContext,
    mut vd: Validator<'a>,
    mut tooltipped: bool,
) {
    // `limit` is accepted in `else` blocks even though it's untidy
    if caller == "if"
        || caller == "else_if"
        || caller == "else"
        || caller == "while"
        || list_type != ListType::None
    {
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

    'outer: for (key, bv) in vd.unknown_keys() {
        if let Some(effect) = data.get_effect(key) {
            match bv {
                BlockOrValue::Value(token) => {
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
                        if token.as_str().parse::<i32>().is_err() {
                            warn(token, ErrorKey::Validation, "expected integer");
                        }
                    }
                }
                Effect::ScriptValue | Effect::NonNegativeValue => {
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
                        if let Some(token) = vd.field_value(&ikey.to_uppercase()) {
                            data.verify_exists(itype, token);
                        }
                        if let Some(token) = vd.field_value(tkey) {
                            validate_target(token, data, sc, outscopes);
                        }
                        if let Some(token) = vd.field_value(&tkey.to_uppercase()) {
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
                Effect::Choice(choices) => {
                    if let Some(token) = bv.expect_value() {
                        if !choices.contains(&token.as_str()) {
                            let msg = format!("expected one of {}", choices.join(", "));
                            error(token, ErrorKey::Validation, &msg);
                        }
                    }
                }
                Effect::Desc => validate_desc(bv, data, sc),
                Effect::Timespan => {
                    if let Some(block) = bv.expect_block() {
                        validate_days_weeks_months_years(block, data, sc);
                    }
                }
                Effect::AddModifier => match bv {
                    BlockOrValue::Value(token) => data.verify_exists(Item::Modifier, token),
                    BlockOrValue::Block(block) => {
                        let mut vd = Validator::new(block, data);
                        vd.req_field("modifier");
                        vd.field_item("modifier", Item::Modifier);
                        vd.field_validated_sc("desc", sc, validate_desc);
                        validate_optional_cooldown(&mut vd, sc);
                    }
                },
                Effect::SpecialBlock => {
                    if let Some(block) = bv.expect_block() {
                        validate_effect_special(
                            &key.as_str().to_lowercase(),
                            block,
                            data,
                            sc,
                            tooltipped,
                        );
                    }
                }
                Effect::SpecialBv => validate_effect_special_bv(
                    &key.as_str().to_lowercase(),
                    bv,
                    data,
                    sc,
                    tooltipped,
                ),
                Effect::ControlOrLabel => match bv {
                    BlockOrValue::Value(t) => data.verify_exists(Item::Localization, t),
                    BlockOrValue::Block(b) => validate_effect_control(
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
        for (i, mut part) in part_vec.iter().enumerate() {
            let first = i == 0;
            let stored_part;

            if let Some((new_part, arg)) = part.split_once('(') {
                if let Some((arg, _)) = arg.split_once(')') {
                    let arg = arg.trim();
                    if new_part.is("vassal_contract_obligation_level_score") {
                        validate_target(&arg, data, sc, Scopes::VassalContract);
                    } else if new_part.is("squared_distance") {
                        validate_target(&arg, data, sc, Scopes::Province);
                    } else {
                        warn(arg, ErrorKey::Validation, "unexpected argument");
                    }
                    stored_part = new_part;
                    part = &stored_part;
                }
            }

            if let Some((prefix, mut arg)) = part.split_once(':') {
                if prefix.is("event_id") {
                    arg = key.split_once(':').unwrap().1;
                }
                if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                    if inscopes == Scopes::None && !first {
                        let msg = format!("`{prefix}:` makes no sense except as first part");
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    sc.expect(inscopes, &prefix);
                    validate_prefix_reference(&prefix, &arg, data);
                    sc.replace(outscope, part.clone());
                    if prefix.is("event_id") {
                        break; // force last part
                    }
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
                let msg = format!("unknown token `{part}`");
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
}

fn validate_effect_control(
    caller: &str,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: bool,
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
        } else {
            vd.field_item("text", Item::TriggerLocalization);
        }
        if let Some(token) = vd.field_value("subject") {
            validate_target(token, data, sc, Scopes::non_primitive());
        }
        tooltipped = false;
    } else {
        vd.ban_field("text", || "`custom_description` or `custom_tooltip`");
        vd.ban_field("subject", || "`custom_description` or `custom_tooltip`");
    }

    if caller == "custom_description" || caller == "custom_description_no_bullet" {
        if let Some(token) = vd.field_value("object") {
            validate_target(token, data, sc, Scopes::non_primitive());
        }
        vd.field_script_value("value", sc);
    } else {
        vd.ban_field("object", || "`custom_description`");
        vd.ban_field("value", || "`custom_description`");
    }

    if caller == "hidden_effect" || caller == "hidden_effect_new_object" {
        tooltipped = false;
    } else if caller == "show_as_tooltip" {
        tooltipped = true;
    }

    if caller == "random" {
        vd.req_field("chance");
        vd.field_script_value("chance", sc);
    } else {
        vd.ban_field("chance", || "`random`");
    }

    if caller == "send_interface_message" || caller == "send_interface_toast" {
        vd.field_value("type");
        vd.field_validated_sc("title", sc, validate_desc);
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_validated_sc("tooltip", sc, validate_desc);
        let icon_scopes =
            Scopes::Character | Scopes::LandedTitle | Scopes::Artifact | Scopes::Faith;
        if let Some(token) = vd.field_value("left_icon") {
            validate_target(token, data, sc, icon_scopes);
        }
        if let Some(token) = vd.field_value("right_icon") {
            validate_target(token, data, sc, icon_scopes);
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
        vd.ban_field("count", || "`while`");
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
        if let Some(b) = vd.field_block("trigger") {
            validate_normal_trigger(b, data, sc, false);
        }
        vd.field_bool("show_chance");
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_script_value("min", sc); // used in vanilla
                                          // TODO: check if "max" also works
    } else {
        if caller != "option" {
            vd.ban_field("trigger", || "`random_list` or `duel`");
        }
        vd.ban_field("show_chance", || "`random_list` or `duel`");
    }

    validate_effect(caller, ListType::None, block, data, sc, vd, tooltipped);
}

fn validate_effect_special_bv(
    caller: &str,
    bv: &BlockOrValue,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: bool,
) {
    if caller.starts_with("set_relation_") {
        match bv {
            BlockOrValue::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("target");
                vd.req_field("reason");
                vd.field_target("target", sc, Scopes::Character);
                vd.field_item("reason", Item::Localization);
                vd.field_item("copy_reason", Item::Relation);
                vd.field_target("province", sc, Scopes::Province);
                vd.field_target("involved_character", sc, Scopes::Character);
            }
        }
    } else if caller == "activate_struggle_catalyst" {
        match bv {
            BlockOrValue::Value(token) => data.verify_exists(Item::Catalyst, token),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("catalyst");
                vd.req_field("character");
                vd.field_item("catalyst", Item::Catalyst);
                vd.field_target("character", sc, Scopes::Character);
            }
        }
    } else if caller == "add_character_flag" {
        match bv {
            BlockOrValue::Value(_token) => (),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("flag");
                vd.field_values("flag");
                validate_optional_cooldown(&mut vd, sc);
            }
        }
    } else if caller == "begin_create_holding" {
        match bv {
            BlockOrValue::Value(token) => data.verify_exists(Item::Holding, token),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("type");
                vd.field_item("type", Item::Holding);
                vd.field_validated_block("refund_cost", |b, data| {
                    let mut vd = Validator::new(b, data);
                    vd.field_script_value("gold", sc);
                    vd.field_script_value("prestige", sc);
                    vd.field_script_value("piety", sc);
                });
            }
        }
    } else if caller == "change_first_name" {
        match bv {
            BlockOrValue::Value(token) => {
                if !data.item_exists(Item::Localization, token.as_str()) {
                    validate_target(token, data, sc, Scopes::Flag);
                }
            }
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("template_character");
                vd.field_target("template_character", sc, Scopes::Character);
            }
        }
    } else if caller == "close_view" {
        match bv {
            BlockOrValue::Value(_token) => (), // TODO
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("view");
                vd.field_value("view"); // TODO
                vd.field_target("player", sc, Scopes::Character);
            }
        }
    } else if caller == "create_alliance" {
        match bv {
            BlockOrValue::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("target");
                vd.field_target("target", sc, Scopes::Character);
                vd.field_target("allied_through_owner", sc, Scopes::Character);
                vd.field_target("allied_through_target", sc, Scopes::Character);
            }
        }
    } else if caller == "create_inspiration" {
        match bv {
            BlockOrValue::Value(token) => data.verify_exists(Item::Inspiration, token),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("type");
                vd.req_field("gold");
                vd.field_item("type", Item::Inspiration);
                vd.field_script_value("gold", sc);
            }
        }
    } else if caller == "create_story" {
        match bv {
            BlockOrValue::Value(token) => data.verify_exists(Item::Story, token),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("type");
                vd.field_item("type", Item::Story);
                vd.field_value("save_scope_as");
                vd.field_value("save_temporary_scope_as");
            }
        }
    } else if caller == "death" {
        match bv {
            BlockOrValue::Value(token) => {
                if !token.is("natural") {
                    let msg = "expected `death = natural`";
                    warn(token, ErrorKey::Validation, msg);
                }
            }
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("death_reason");
                vd.field_item("death_reason", Item::DeathReason);
                vd.field_target("killer", sc, Scopes::Character);
                vd.field_target("artifact", sc, Scopes::Artifact);
            }
        }
    } else if caller == "open_view" || caller == "open_view_data" {
        match bv {
            BlockOrValue::Value(_token) => (), // TODO
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("view");
                vd.field_value("view"); // TODO
                vd.field_value("view_message"); // TODO
                vd.field_target("player", sc, Scopes::Character);
            }
        }
    } else if caller == "remove_courtier_or_guest" {
        match bv {
            BlockOrValue::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("character");
                vd.field_target("character", sc, Scopes::Character);
                vd.field_target("new_location", sc, Scopes::Province);
            }
        }
    } else if caller == "set_coa" {
        if let Some(token) = bv.expect_value() {
            if !data.item_exists(Item::Coa, token.as_str()) {
                let options = Scopes::LandedTitle | Scopes::Dynasty | Scopes::DynastyHouse;
                validate_target(token, data, sc, sc.scopes() & options);
            }
        }
    } else if caller == "set_global_variable"
        || caller == "set_local_variable"
        || caller == "set_variable"
    {
        match bv {
            BlockOrValue::Value(_token) => (),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("name");
                vd.field_value("name");
                if let Some(bv) = vd.field("value") {
                    match bv {
                        BlockOrValue::Value(token) => {
                            validate_target(token, data, sc, Scopes::all_but_none());
                        }
                        BlockOrValue::Block(_) => ScriptValue::validate_bv(bv, data, sc),
                    }
                }
                validate_optional_cooldown(&mut vd, sc);
            }
        }
    } else if caller == "set_location" {
        match bv {
            BlockOrValue::Value(token) => validate_target(token, data, sc, Scopes::Province),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("location");
                vd.field_target("location", sc, Scopes::Province);
                vd.field_bool("stick_to_location");
            }
        }
    } else if caller == "set_owner" {
        match bv {
            BlockOrValue::Value(token) => validate_target(token, data, sc, Scopes::Character),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("target");
                vd.field_target("target", sc, Scopes::Character);
                vd.field_validated_blocks_sc("history", sc, validate_artifact_history);
                vd.field_bool("generate_history");
            }
        }
    } else if caller == "trigger_event" {
        match bv {
            BlockOrValue::Value(token) => {
                data.verify_exists(Item::Event, token);
                data.events.check_scope(token, sc);
            }
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_item("id", Item::Event);
                vd.field_item("on_action", Item::OnAction);
                vd.field_target("saved_event_id", sc, Scopes::Flag);
                vd.field_date("trigger_on_next_date");
                vd.field_bool("delayed");
                validate_optional_cooldown(&mut vd, sc);
                if let Some(token) = block.get_field_value("id") {
                    data.events.check_scope(token, sc);
                }
            }
        }
    }
}

fn validate_effect_special(
    caller: &str,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let mut vd = Validator::new(block, data);
    if caller == "add_activity_log_entry" {
        vd.req_field("key");
        vd.req_field("character");
        vd.field_item("key", Item::Localization);
        vd.field_script_value("score", sc);
        vd.field_validated_block("tags", |b, data| {
            let mut vd = Validator::new(b, data);
            vd.values(); // TODO
        });
        vd.field_bool("show_in_conclusion");
        vd.field_target("character", sc, Scopes::Character);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_target("location", sc, Scopes::Province);
        vd.field_target("artifact", sc, Scopes::Artifact);
        // effects can be put directly in this block
        validate_effect(caller, ListType::None, block, data, sc, vd, tooltipped);
    } else if caller == "add_artifact_history" {
        vd.req_field("type");
        vd.req_field("recipient");
        vd.field_item("type", Item::ArtifactHistory);
        vd.field_date("date");
        vd.field_target("actor", sc, Scopes::Character);
        vd.field_target("recipient", sc, Scopes::Character);
        vd.field_target("location", sc, Scopes::Province);
    } else if caller == "add_artifact_title_history" {
        vd.req_field("target");
        vd.req_field("date");
        vd.field_target("target", sc, Scopes::LandedTitle);
        vd.field_date("date");
    } else if caller == "add_from_contribution_attackers"
        || caller == "add_from_contribution_defenders"
    {
        vd.field_script_value("prestige", sc);
        vd.field_script_value("gold", sc);
        vd.field_script_value("piety", sc);
    } else if caller == "add_hook" || caller == "add_hook_no_toast" {
        vd.req_field("type");
        vd.req_field("target");
        vd.field_item("type", Item::Hook);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_item("secret", Item::Secret);
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "add_opinion" || caller == "reverse_add_opinion" {
        vd.req_field("modifier");
        vd.req_field("target");
        vd.field_item("modifier", Item::Modifier);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_script_value("opinion", sc); // undocumented
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "add_relation_flag" || caller == "remove_relation_flag" {
        vd.req_field("relation");
        vd.req_field("flag");
        vd.req_field("target");
        vd.field_item("relation", Item::Relation);
        // TODO: check that the flag belongs to the relation
        vd.field_value("flag");
        vd.field_target("target", sc, Scopes::Character);
    } else if caller == "add_scheme_cooldown" {
        vd.req_field("target");
        vd.req_field("type");
        vd.field_target("target", sc, Scopes::Character);
        vd.field_item("type", Item::Scheme);
        validate_optional_cooldown_int(&mut vd);
    } else if caller == "add_scheme_modifier" {
        vd.req_field("type");
        vd.field_item("type", Item::Scheme);
        vd.field_integer("days");
    } else if caller == "add_to_global_variable_list"
        || caller == "add_to_local_variable_list"
        || caller == "add_to_variable_list"
        || caller == "remove_list_global_variable"
        || caller == "remove_list_local_variable"
        || caller == "remove_list_variable"
    {
        vd.req_field("name");
        vd.req_field("target");
        vd.field_value("name");
        for target in vd.field_values("target") {
            validate_target(target, data, sc, Scopes::all_but_none());
        }
    } else if caller == "add_to_guest_subset" {
        vd.req_field("name");
        vd.req_field("target");
        vd.field_item("name", Item::GuestSubset);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_item("phase", Item::ActivityPhase);
    } else if caller == "add_trait_xp" {
        vd.req_field("trait");
        vd.req_field("value");
        vd.field_item("trait", Item::Trait);
        vd.field_item("track", Item::TraitTrack);
        vd.field_script_value("value", sc);
    } else if caller == "add_truce_both_ways" || caller == "add_truce_one_way" {
        vd.req_field("character");
        vd.field_target("character", sc, Scopes::Character);
        vd.field_bool("override");
        vd.field_choice("result", &["white_peace", "victory", "defeat"]);
        vd.field_item("casus_belli", Item::CasusBelli);
        vd.field_validated_sc("name", sc, validate_desc);
        vd.field_target("war", sc, Scopes::War);
        validate_optional_cooldown(&mut vd, sc);
        if block.has_key("war") && block.has_key("casus_belli") {
            let msg = "cannot use both `war` and `casus_belli`";
            error(block, ErrorKey::Validation, msg);
        }
    } else if caller == "assign_council_task" {
        vd.req_field("council_task");
        vd.req_field("target");
        vd.field_target("council_task", sc, Scopes::Province | Scopes::Character);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_bool("fire_on_actions");
    } else if caller == "assign_councillor_type" {
        vd.req_field("type");
        vd.req_field("target");
        vd.field_item("type", Item::CouncilPosition);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_bool("fire_on_actions");
        vd.field_bool("remove_existing_councillor");
    } else if caller == "battle_event" {
        vd.req_field("left_portrait");
        vd.req_field("right_portrait");
        vd.req_field("key");
        if let Some(token) = vd.field_value("key") {
            let loca = format!("{token}_friendly");
            data.verify_exists_implied(Item::Localization, &loca, token);
            let loca = format!("{token}_enemy");
            data.verify_exists_implied(Item::Localization, &loca, token);
        }
        vd.field_target("left_portrait", sc, Scopes::Character);
        vd.field_target("right_portrait", sc, Scopes::Character);
        vd.field_value("type"); // TODO, undocumented
        vd.field_bool("target_right"); // undocumented
    } else if caller == "change_cultural_acceptance" {
        vd.req_field("target");
        vd.req_field("value");
        vd.field_target("target", sc, Scopes::Culture);
        vd.field_script_value("value", sc);
        vd.field_validated_sc("desc", sc, validate_desc);
    } else if caller == "change_global_variable"
        || caller == "change_local_variable"
        || caller == "change_variable"
    {
        vd.req_field("name");
        vd.field_value("name");
        vd.field_script_value("add", sc);
        vd.field_script_value("subtract", sc);
        vd.field_script_value("multiply", sc);
        vd.field_script_value("divide", sc);
        vd.field_script_value("modulo", sc);
        vd.field_script_value("min", sc);
        vd.field_script_value("max", sc);
    } else if caller == "change_liege" {
        vd.req_field("liege");
        vd.req_field("change");
        vd.field_target("liege", sc, Scopes::Character);
        vd.field_target("change", sc, Scopes::TitleAndVassalChange);
    } else if caller == "change_title_holder" || caller == "change_title_holder_include_vassals" {
        vd.req_field("holder");
        vd.req_field("change");
        vd.field_target("holder", sc, Scopes::Character);
        vd.field_target("change", sc, Scopes::TitleAndVassalChange);
        vd.field_bool("take_baronies");
        vd.field_target("government_base", sc, Scopes::Character);
    } else if caller == "change_trait_rank" || caller == "set_trait_rank" {
        vd.req_field("trait");
        vd.req_field("rank");
        // TODO: check that it's a rankable trait
        vd.field_item("trait", Item::Trait);
        vd.field_script_value("rank", sc);
        if caller == "change_trait_rank" {
            vd.field_script_value("max", sc);
        }
    } else if caller == "clamp_global_variable"
        || caller == "clamp_local_variable"
        || caller == "clamp_variable"
    {
        vd.req_field("name");
        vd.field_value("name");
        vd.field_script_value("min", sc);
        vd.field_script_value("max", sc);
    } else if caller == "copy_localized_text" {
        vd.req_field("key");
        vd.req_field("target");
        vd.field_value("key");
        vd.field_target("target", sc, Scopes::Character);
    } else if caller == "create_accolade" {
        vd.req_field("knight");
        vd.req_field("primary");
        vd.req_field("secondary");
        vd.field_target("knight", sc, Scopes::Character);
        vd.field_item("primary", Item::AccoladeType);
        vd.field_item("secondary", Item::AccoladeType);
        vd.field_item("name", Item::Localization);
    } else if caller == "create_artifact" || caller == "reforge_artifact" {
        validate_artifact(caller, block, data, vd, sc, tooltipped);
    } else if caller == "create_character" {
        vd.field_value("save_scope_as"); // docs say event_target instead of scope
        vd.field_value("save_temporary_scope_as"); // docs say event_target instead of scope
        vd.field_validated_sc("name", sc, validate_desc);
        vd.field_script_value("age", sc);
        if let Some(token) = vd.field_value("gender") {
            if !token.is("male") && !token.is("female") {
                validate_target(token, data, sc, Scopes::Character);
            }
        }
        vd.field_script_value("gender_female_chance", sc);
        vd.field_target("opposite_gender", sc, Scopes::Character);
        vd.field_values_items("trait", Item::Trait);
        vd.field_blocks("random_traits_list"); // TODO
        vd.field_bool("random_traits");
        vd.field_script_value("health", sc);
        vd.field_script_value("fertility", sc);
        vd.field_target("mother", sc, Scopes::Character);
        vd.field_target("father", sc, Scopes::Character);
        vd.field_target("real_father", sc, Scopes::Character);
        vd.field_target("employer", sc, Scopes::Character);
        vd.field_target("location", sc, Scopes::Province);
        vd.field_item("template", Item::CharacterTemplate); // undocumented
        vd.field_target("template_character", sc, Scopes::Character);
        vd.field_item_or_target("faith", sc, Item::Faith, Scopes::Faith);
        vd.field_block("random_faith"); // TODO
        vd.field_item_or_target(
            "random_faith_in_religion",
            sc,
            Item::Religion,
            Scopes::Faith,
        );
        vd.field_item_or_target("culture", sc, Item::Culture, Scopes::Culture);
        vd.field_block("random_culture"); // TODO
                                          // TODO: figure out what a culture group is, and whether this key still works at all
        vd.field_value("random_culture_in_group");
        vd.field_item_or_target("dynasty_house", sc, Item::House, Scopes::DynastyHouse);
        if let Some(token) = vd.field_value("dynasty") {
            if !token.is("generate") && !token.is("inherit") && !token.is("none") {
                validate_target(token, data, sc, Scopes::Dynasty);
            }
        }
        vd.field_script_value("diplomacy", sc);
        vd.field_script_value("intrigue", sc);
        vd.field_script_value("martial", sc);
        vd.field_script_value("learning", sc);
        vd.field_script_value("prowess", sc);
        vd.field_script_value("stewardship", sc);
        if let Some(b) = vd.field_block("after_creation") {
            sc.open_scope(
                Scopes::Character,
                block.get_key("after_creation").unwrap().clone(),
            );
            validate_normal_effect(b, data, sc, tooltipped);
            sc.close();
        }
    } else if caller == "create_character_memory" {
        vd.req_field("type");
        vd.field_item("type", Item::MemoryType);
        vd.field_validated_block("participants", |b, data| {
            for (_key, token) in b.iter_assignments_warn() {
                validate_target(token, data, sc, Scopes::Character);
            }
        });
        vd.field_validated_block_sc("duration", sc, validate_cooldown);
    } else if caller == "create_dynamic_title" {
        vd.req_field("tier");
        vd.req_field("name");
        vd.field_choice("tier", &["duchy", "kingdom", "empire"]);
        vd.field_validated_sc("name", sc, validate_desc);
        vd.field_validated_sc("adjective", sc, validate_desc);
    } else if caller == "create_holy_order" {
        vd.req_field("leader");
        vd.req_field("capital");
        vd.field_target("leader", sc, Scopes::Character);
        vd.field_target("capital", sc, Scopes::LandedTitle);
        vd.field_value("save_scope_as");
        vd.field_value("save_temporary_scope_as");
    } else if caller == "create_title_and_vassal_change" {
        vd.req_field("type");
        vd.field_choice(
            "type",
            &[
                "conquest",
                "independency",
                "conquest_claim",
                "granted",
                "revoked",
                "conquest_holy_war",
                "swear_fealty",
                "created",
                "usurped",
                "returned",
                "leased_out",
                "conquest_populist",
                "faction_demand",
            ],
        );
        vd.field_value("save_scope_as");
        vd.field_bool("add_claim_on_loss");
    } else if caller == "delay_travel_plan" {
        vd.field_bool("add");
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "divide_war_chest" {
        vd.field_bool("defenders");
        vd.field_script_value("fraction", sc);
        vd.field_bool("gold");
        vd.field_bool("piety");
        vd.field_bool("prestige");
    } else if caller == "duel" {
        vd.field_item("skill", Item::Skill);
        vd.field_list_items("skills", Item::Skill);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_script_value("value", sc);
        vd.field_item("localization", Item::EffectLocalization);
        validate_random_list("duel", block, data, vd, sc, tooltipped);
    } else if caller == "faction_start_war" {
        vd.field_target("title", sc, Scopes::LandedTitle);
    } else if caller == "force_add_to_scheme" {
        vd.field_item("scheme", Item::Scheme);
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "force_vote_as" {
        vd.field_target("target", sc, Scopes::Character);
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "imprison" {
        vd.field_target("target", sc, Scopes::Character);
        vd.field_item("type", Item::PrisonType);
        // The docs also have a "reason" key, but no indication what it is
    } else if caller == "join_faction_forced" {
        vd.field_target("faction", sc, Scopes::Faction);
        vd.field_target("forced_by", sc, Scopes::Character);
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "make_pregnant" || caller == "make_pregnant_no_checks" {
        vd.field_target("father", sc, Scopes::Character);
        vd.field_integer("number_of_children");
        vd.field_bool("known_bastard");
    } else if caller == "move_budget_gold" {
        vd.field_script_value("gold", sc);
        let choices = &[
            "budget_war_chest",
            "budget_reserved",
            "budget_short_term",
            "budget_long_term",
        ];
        vd.field_choice("from", choices);
        vd.field_choice("to", choices);
    } else if caller == "open_interaction_window" || caller == "run_interaction" {
        vd.req_field("interaction");
        vd.req_field("actor");
        vd.req_field("recipient");
        vd.field_value("interaction"); // TODO
        vd.field_bool("redirect");
        vd.field_target("actor", sc, Scopes::Character);
        vd.field_target("recipient", sc, Scopes::Character);
        vd.field_target("secondary_actor", sc, Scopes::Character);
        vd.field_target("secondary_recipient", sc, Scopes::Character);
        if caller == "open_interaction_window" {
            vd.field_target("target_title", sc, Scopes::LandedTitle);
        }
        if caller == "run_interaction" {
            vd.field_choice("execute_threshold", &["accept", "maybe", "decline"]);
            vd.field_choice("send_threshold", &["accept", "maybe", "decline"]);
        }
    } else if caller == "pay_long_term_income"
        || caller == "pay_reserved_income"
        || caller == "pay_short_term_income"
        || caller == "pay_war_chest_income"
    {
        vd.req_field("target");
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "random_list" {
        validate_random_list("random_list", block, data, vd, sc, tooltipped);
    } else if caller == "remove_opinion" {
        vd.req_field("target");
        vd.req_field("modifier");
        vd.field_target("target", sc, Scopes::Character);
        vd.field_item("modifier", Item::Modifier);
        vd.field_bool("single");
    } else if caller == "replace_court_position" {
        vd.req_field("recipient");
        vd.req_field("court_position");
        vd.field_target("recipient", sc, Scopes::Character);
        vd.field_target("holder", sc, Scopes::Character);
        vd.field_item("court_position", Item::CourtPosition);
    } else if caller == "round_global_variable"
        || caller == "round_local_variable"
        || caller == "round_variable"
    {
        vd.req_field("name");
        vd.req_field("nearest");
        vd.field_value("name");
        vd.field_script_value("nearest", sc);
    } else if caller == "save_opinion_value_as" || caller == "save_temporary_opiion_value_as" {
        vd.req_field("name");
        vd.req_field("target");
        vd.field_value("name");
        vd.field_target("target", sc, Scopes::Character);
    } else if caller == "save_scope_value_as" || caller == "save_temporary_scope_value_as" {
        vd.req_field("name");
        vd.req_field("value");
        vd.field_value("name");
        vd.field_script_value_or_flag("value", sc);
    } else if caller == "scheme_freeze" {
        vd.req_field("reason");
        vd.field_item("reason", Item::Localization);
        validate_optional_cooldown(&mut vd, sc);
    } else if caller == "set_council_task" {
        vd.req_field("task_type");
        vd.req_field("target");
        vd.field_item("task_type", Item::CouncilTask);
        vd.field_target("target", sc, Scopes::Character | Scopes::Province);
    } else if caller == "set_culture_name" {
        vd.req_field("noun");
        vd.field_validated_sc("noun", sc, validate_desc);
        vd.field_validated_sc("collective_noun", sc, validate_desc);
        vd.field_validated_sc("prefix", sc, validate_desc);
    } else if caller == "set_death_reason" {
        vd.req_field("death_reason");
        vd.field_item("death_reason", Item::DeathReason);
        vd.field_target("killer", sc, Scopes::Character);
        vd.field_target("artifact", sc, Scopes::Artifact);
    } else if caller == "set_great_holy_war_target" || caller == "start_great_holy_war" {
        vd.req_field("target_character");
        vd.req_field("target_title");
        vd.field_target("target_character", sc, Scopes::Character);
        vd.field_target("target_title", sc, Scopes::LandedTitle);
        if caller == "start_great_holy_war" {
            vd.field_script_value("delay", sc);
            vd.field_target("war", sc, Scopes::War);
        }
    } else if caller == "setup_claim_cb"
        || caller == "setup_de_jure_cb"
        || caller == "setup_invasion_cb"
    {
        vd.req_field("attacker");
        vd.req_field("defender");
        vd.req_field("change");
        vd.field_target("attacker", sc, Scopes::Character);
        vd.field_target("defender", sc, Scopes::Character);
        vd.field_target("change", sc, Scopes::TitleAndVassalChange);
        vd.field_bool("victory");
        if caller == "setup_claim_cb" {
            vd.req_field("claimant");
            vd.field_target("claimant", sc, Scopes::Character);
            vd.field_bool("take_occupied");
            vd.field_bool("civil_war");
            vd.field_choice("titles", &["target_titles", "faction_titles"]); // Undocumented
        } else if caller == "setup_de_jure_cb" {
            vd.field_target("title", sc, Scopes::LandedTitle);
        } else {
            vd.field_bool("take_occupied");
        }
    } else if caller == "spawn_army" {
        // TODO: either levies or men_at_arms
        vd.req_field("location");
        vd.field_script_value("levies", sc);
        vd.field_validated_blocks("men_at_arms", |b, data| {
            let mut vd = Validator::new(b, data);
            vd.req_field("type");
            vd.field_item("type", Item::MenAtArms);
            vd.field_script_value("men", sc);
            vd.field_script_value("stacks", sc);
        });
        vd.field_target("location", sc, Scopes::Province);
        vd.field_target("origin", sc, Scopes::Province);
        vd.field_target("war", sc, Scopes::War);
        vd.field_bool("war_keep_on_attacker_victory");
        vd.field_bool("inheritable");
        vd.field_bool("uses_supply");
        vd.field_target("army", sc, Scopes::Army);
        vd.field_value("save_scope_as");
        vd.field_value("save_temporary_scope_as");
        vd.field_validated_sc("name", sc, validate_desc);
    } else if caller == "start_scheme" {
        vd.req_field("type");
        vd.req_field("target");
        vd.field_item("type", Item::Scheme);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_target("artifact", sc, Scopes::Artifact);
    } else if caller == "start_struggle" {
        vd.req_field("struggle_type");
        vd.req_field("start_phase");
        vd.field_item("struggle_type", Item::Struggle);
        vd.field_item("start_phase", Item::StrugglePhase);
    } else if caller == "start_travel_plan" {
        vd.req_field("destination");
        for token in vd.field_values("destination") {
            validate_target(token, data, sc, Scopes::Province);
        }
        vd.field_target("travel_leader", sc, Scopes::Character);
        for token in vd.field_values("companion") {
            validate_target(token, data, sc, Scopes::Character);
        }
        vd.field_bool("players_use_planner");
        vd.field_bool("return_trip");
        vd.field_item("on_arrival_event", Item::Event);
        vd.field_item("on_arrival_on_action", Item::OnAction);
        vd.field_item("on_start_event", Item::Event);
        vd.field_item("on_start_on_action", Item::OnAction);
        vd.field_item("on_travel_planner_cancel_event", Item::Event);
        vd.field_item("on_travel_planner_cancel_on_action", Item::OnAction);
        vd.field_choice(
            "on_arrival_destinations",
            &["all", "first", "last", "all_but_last"],
        );
        // Root for these events is travel plan owner
        if let Some(token) = block.get_field_value("on_arrival_event") {
            data.events.check_scope(token, sc);
        }
        if let Some(token) = block.get_field_value("on_start_event") {
            data.events.check_scope(token, sc);
        }
        if let Some(token) = block.get_field_value("on_travel_planner_cancel_event") {
            data.events.check_scope(token, sc);
        }
    } else if caller == "start_war" {
        vd.field_item("casus_belli", Item::CasusBelli);
        vd.field_item("cb", Item::CasusBelli);
        vd.field_target("target", sc, Scopes::Character);
        vd.field_target("claimant", sc, Scopes::Character);
        for token in vd.field_values("target_title") {
            validate_target(token, data, sc, Scopes::LandedTitle);
        }
    } else if caller == "stress_impact" {
        vd.field_script_value("base", sc);
        for (token, bv) in vd.unknown_keys() {
            data.verify_exists(Item::Trait, token);
            ScriptValue::validate_bv(bv, data, sc);
        }
    } else if caller == "switch" {
        vd.req_field("trigger");
        if let Some(target) = vd.field_value("trigger") {
            // clone to avoid calling vd again while target is still borrowed
            let target = target.clone();
            for (key, bv) in vd.unknown_keys() {
                // Pretend the switch was written as a series of trigger = key lines
                let synthetic_bv = BlockOrValue::Value(key.clone());
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
    } else if caller == "try_create_important_action" {
        vd.req_field("important_action_type");
        vd.field_item("important_action_type", Item::ImportantAction);
        vd.field_value("scope_name");
    } else if caller == "try_create_suggestion" {
        vd.req_field("suggestion_type");
        vd.field_item("suggestion_type", Item::Suggestion);
        vd.field_target("actor", sc, Scopes::Character);
        vd.field_target("recipient", sc, Scopes::Character);
        vd.field_target("secondary_actor", sc, Scopes::Character);
        vd.field_target("secondary_recipient", sc, Scopes::Character);
        vd.field_target("landed_title", sc, Scopes::LandedTitle);
    } else if caller == "vassal_contract_set_obligation_level" {
        vd.req_field("type");
        vd.req_field("level");
        vd.field_item("type", Item::VassalObligation);
        if let Some(token) = vd.field_value("level") {
            if token.as_str().parse::<i32>().is_err()
                && !data.item_exists(Item::VassalObligationLevel, token.as_str())
            {
                validate_target(token, data, sc, Scopes::VassalContractObligationLevel);
            }
        }
    } else {
        vd.no_warn_remaining(); // TODO
    }
}

fn validate_artifact_history(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.req_field("type");
    vd.field_item("type", Item::ArtifactHistory);
    vd.field_date("date");
    vd.field_target("actor", sc, Scopes::Character);
    vd.field_target("recipient", sc, Scopes::Character);
    vd.field_target("location", sc, Scopes::Province);
}

fn validate_artifact(
    caller: &str,
    _block: &Block,
    _data: &Everything,
    mut vd: Validator,
    sc: &mut ScopeContext,
    _tooltipped: bool,
) {
    vd.field_validated_sc("name", sc, validate_desc);
    vd.field_validated_sc("description", sc, validate_desc);
    vd.field_item("rarity", Item::ArtifactRarity);
    vd.field_item("type", Item::ArtifactSlot);
    vd.field_values_items("modifier", Item::Modifier);
    vd.field_script_value("durability", sc);
    vd.field_script_value("max_durability", sc);
    vd.field_bool("decaying");
    vd.field_validated_blocks_sc("history", sc, validate_artifact_history);
    vd.field_item("template", Item::ArtifactTemplate);
    vd.field_item("visuals", Item::ArtifactVisual);
    vd.field_bool("generate_history");
    vd.field_script_value("quality", sc);
    vd.field_script_value("wealth", sc);
    vd.field_target("creator", sc, Scopes::Character);
    vd.field_target(
        "visuals_source",
        sc,
        Scopes::LandedTitle | Scopes::Dynasty | Scopes::DynastyHouse,
    );

    if caller == "create_artifact" {
        vd.field_value("save_scope_as");
        vd.field_target("title_history", sc, Scopes::LandedTitle);
        vd.field_date("title_history_date");
    } else {
        vd.ban_field("save_scope_as", || "`create_artifact`");
        vd.ban_field("title_history", || "`create_artifact`");
        vd.ban_field("title_history_date", || "`create_artifact`");
    }
}

fn validate_random_list(
    caller: &str,
    _block: &Block,
    data: &Everything,
    mut vd: Validator,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    vd.field_integer("pick");
    vd.field_bool("unique"); // don't know what this does
    vd.field_validated_sc("desc", sc, validate_desc);
    for (key, bv) in vd.unknown_keys() {
        if f64::from_str(key.as_str()).is_err() {
            let msg = "expected number";
            error(key, ErrorKey::Validation, msg);
        } else if let Some(block) = bv.expect_block() {
            validate_effect_control(caller, block, data, sc, tooltipped);
        }
    }
}
