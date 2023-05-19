use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, Comparator, Date};
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, warn, warn_info};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, Scopes};
use crate::tables::triggers::{scope_trigger, trigger_comparevalue, Trigger};
use crate::token::Token;
use crate::util::stringify_choices;
use crate::validate::validate_prefix_reference;

pub fn validate_normal_trigger(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    validate_trigger("", false, block, data, sc, tooltipped);
}

pub fn validate_trigger(
    caller: &str,
    in_list: bool,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: bool,
) {
    let mut seen_if = false;

    if caller == "custom_description"
        || caller == "custom_tooltip"
        || block.get_field_value("custom_tooltip").is_some()
    {
        tooltipped = false;
    }

    for (key, cmp, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("limit") {
                // limit blocks are accepted in trigger_else even though it doesn't make sense
                if caller == "trigger_if" || caller == "trigger_else_if" || caller == "trigger_else"
                {
                    if let Some(block) = bv.expect_block() {
                        validate_normal_trigger(block, data, sc, tooltipped);
                    }
                } else {
                    warn(key, ErrorKey::Validation, "can only use `limit` in `trigger_if` or `trigger_else_if` or `trigger_else`");
                }
                continue;
            }

            if key.is("trigger_if") {
                seen_if = true;
            } else if key.is("trigger_else_if") || key.is("trigger_else") {
                if !seen_if {
                    let msg = "must follow `trigger_if` or `trigger_else_if`";
                    error(key, ErrorKey::Validation, msg);
                }
                if key.is("trigger_else") {
                    seen_if = false;
                }
            } else {
                seen_if = false;
            }

            if key.is("percent") {
                if !in_list {
                    let msg = "can only use `percent =` in an `any_` list";
                    warn(key, ErrorKey::Validation, msg);
                    continue;
                }
                if let Some(token) = bv.get_value() {
                    if let Ok(num) = token.as_str().parse::<f64>() {
                        if num > 1.0 {
                            let msg = "'percent' here needs to be between 0 and 1";
                            warn(token, ErrorKey::Range, msg);
                        }
                    }
                }
                ScriptValue::validate_bv(bv, data, sc);
                continue;
            }

            if key.is("count") {
                if !in_list {
                    let msg = "can only use `count =` in an `any_` list";
                    warn(key, ErrorKey::Validation, msg);
                    continue;
                }
                if let Some(token) = bv.get_value() {
                    if token.is("all") {
                        continue;
                    }
                }
                ScriptValue::validate_bv(bv, data, sc);
                continue;
            }

            if key.is("list") || key.is("variable") {
                if caller != "any_in_list"
                    && caller != "any_in_global_list"
                    && caller != "any_in_local_list"
                {
                    let msg = format!("can only use `{key} =` in `any_in_list`, `any_in_global_list`, or `any_in_local_list`");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                bv.expect_value(); // TODO
                continue;
            }

            if key.is("type") {
                if caller == "any_relation" {
                    if let Some(token) = bv.expect_value() {
                        data.verify_exists(Item::Relation, token);
                    }
                } else if caller == "any_court_position_holder" {
                    if let Some(token) = bv.expect_value() {
                        data.verify_exists(Item::CourtPosition, token);
                    }
                } else {
                    let msg = format!("can only use `{key} =` in `any_relation` list");
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if key.is("province") {
                if caller != "any_pool_character" {
                    let msg = format!("can only use `{key} =` in `any_pool_character` list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(token) = bv.expect_value() {
                    validate_target(token, data, sc, Scopes::Province);
                }
                continue;
            }

            if key.is("even_if_dead") || key.is("only_if_dead") {
                if !in_list || !sc.can_be(Scopes::Character) {
                    let msg = format!("can only use `{key} =` in a character list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                // TODO Might be literal yes/no expected, might be any bool value
                bv.expect_value();
                continue;
            }

            if key.is("involvement") {
                if caller != "any_character_struggle" {
                    let msg = format!("can only use `{key} =` in `any_character_struggle` list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(token) = bv.expect_value() {
                    if !(token.is("involved") || token.is("interloper")) {
                        let msg = "expected one of `involved` or `interloper`";
                        error(token, ErrorKey::Validation, msg);
                    }
                }
                continue;
            }

            if key.is("region") {
                if caller != "any_county_in_region" {
                    let msg = format!("can only use `{key} =` in `any_county_in_region` list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(Item::Region, token);
                }
                continue;
            }

            if key.is("filter") || key.is("continue") {
                if caller != "any_in_de_jure_hierarchy" && caller != "any_in_de_facto_hierarchy" {
                    let msg = format!("can only use `{key} =` in `..._hierarchy` list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(block) = bv.expect_block() {
                    validate_normal_trigger(block, data, sc, false);
                }
                continue;
            }

            if key.is("pressed") || key.is("explicit") {
                if caller != "any_claim" {
                    let msg = format!("can only use `{key} =` in `any_claim` list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                bv.expect_value(); // TODO: check yes/no/all
                continue;
            }

            if key.is("name") {
                if caller != "any_guest_subset" && caller != "any_guest_subset_current_phase" {
                    let msg = format!("can only use `{key} =` in `any_guest_subset` list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(Item::GuestSubset, token);
                }
                continue;
            }

            if key.is("phase") {
                if caller != "any_guest_subset" {
                    let msg = format!("can only use `{key} =` in `any_guest_subset` list");
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(Item::ActivityPhase, token);
                }
                continue;
            }

            if key.is("text") {
                if caller == "custom_description" {
                    if let Some(token) = bv.expect_value() {
                        data.verify_exists(Item::TriggerLocalization, token);
                    }
                } else if caller == "custom_tooltip" {
                    if let Some(token) = bv.expect_value() {
                        data.localization.verify_exists(token);
                    }
                } else {
                    let msg = format!(
                        "can only use `{key} =` in `custom_description` or `custom_tooltip`",
                    );
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if key.is("subject") {
                if caller == "custom_description" || caller == "custom_tooltip" {
                    if let Some(token) = bv.expect_value() {
                        validate_target(token, data, sc, Scopes::non_primitive());
                    }
                } else {
                    let msg = format!(
                        "can only use `{key} =` in `custom_description` or `custom_tooltip`",
                    );
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }
            if key.is("object") {
                if caller == "custom_description" {
                    if let Some(token) = bv.expect_value() {
                        validate_target(token, data, sc, Scopes::non_primitive());
                    }
                } else {
                    let msg = format!("can only use `{key} =` in `custom_description`");
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }
            if key.is("value") {
                if caller == "custom_description" {
                    ScriptValue::validate_bv(bv, data, sc);
                } else {
                    let msg = format!("can only use `{key} =` in `custom_description`");
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if key.is("add") || key.is("factor") {
                if caller == "modifier" {
                    ScriptValue::validate_bv(bv, data, sc);
                } else {
                    let msg = format!("can only use `{key} =` in `modifier`");
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            // TODO: undocumented; found in vanilla. Verify that it works.
            if key.is("trigger") {
                if caller == "modifier" {
                    if let Some(block) = bv.expect_block() {
                        validate_normal_trigger(block, data, sc, false);
                    }
                } else {
                    let msg = format!("can only use `{key} =` in `modifier`");
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if key.is("amount") {
                if caller == "calc_true_if" {
                    if let Some(token) = bv.expect_value() {
                        if token.as_str().parse::<i32>().is_err() {
                            warn(token, ErrorKey::Validation, "expected a number");
                        }
                    }
                } else {
                    let msg = format!("can only use `{key} =` in `calc_true_if`");
                    warn(key, ErrorKey::Validation, &msg);
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
                        if !it_type.is("any") {
                            let msg = format!("cannot use `{it_type}_` list in a trigger");
                            error(key, ErrorKey::Validation, &msg);
                            continue;
                        }
                        sc.expect(inscopes, key);
                        if let Some(b) = bv.get_block() {
                            sc.open_scope(outscope, key.clone());
                            validate_trigger(key.as_str(), true, b, data, sc, tooltipped);
                            sc.close();
                        } else {
                            error(bv, ErrorKey::Validation, "expected block, found value");
                        }
                        continue;
                    }
                }
            }

            validate_trigger_key_bv(key, *cmp, bv, data, sc, tooltipped);
        } else {
            match bv {
                BlockOrValue::Token(t) => warn_info(
                    t,
                    ErrorKey::Validation,
                    "unexpected token",
                    "did you forget an = ?",
                ),
                BlockOrValue::Block(b) => warn_info(
                    b,
                    ErrorKey::Validation,
                    "unexpected block",
                    "did you forget an = ?",
                ),
            }
        }
    }
}

pub fn validate_trigger_key_bv(
    key: &Token,
    cmp: Comparator,
    bv: &BlockOrValue,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    // Scripted trigger?
    if let Some(trigger) = data.get_trigger(key) {
        match bv {
            BlockOrValue::Token(token) => {
                if !(token.is("yes") || token.is("no")) {
                    warn(token, ErrorKey::Validation, "expected yes or no");
                }
                if !trigger.macro_parms().is_empty() {
                    error(token, ErrorKey::Macro, "expected macro arguments");
                }
                trigger.validate_call(key, data, sc, tooltipped);
            }
            BlockOrValue::Block(block) => {
                let parms = trigger.macro_parms();
                if parms.is_empty() {
                    error(
                        block,
                        ErrorKey::Macro,
                        "trigger does not need macro arguments",
                    );
                } else {
                    let mut vec = Vec::new();
                    let mut vd = Validator::new(block, data);
                    for parm in &parms {
                        vd.req_field(parm);
                        if let Some(token) = vd.field_value(parm.as_str()) {
                            vec.push(token.clone());
                        } else {
                            return;
                        }
                    }
                    let args = parms.into_iter().zip(vec.into_iter()).collect();
                    trigger.validate_macro_expansion(key, args, data, sc, tooltipped);
                }
            }
        }
        return;
    }

    // `10 < scriptvalue` is a valid trigger
    if key.as_str().parse::<f64>().is_ok() {
        ScriptValue::validate_bv(bv, data, sc);
        return;
    }

    let part_vec = key.split('.');
    sc.open_builder();
    let mut warn_against_eq = None;
    let mut found_trigger = None;
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];

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
                return;
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
        } else if data.scriptvalues.exists(part.as_str()) {
            if !last {
                let msg = "script value should be the last part";
                warn(part, ErrorKey::Validation, msg);
                sc.close();
                return;
            }
            data.scriptvalues.validate_call(part, data, sc);
            sc.replace(Scopes::Value, part.clone());
        } else if let Some((inscopes, outscope)) = scope_to_scope(part) {
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as first part");
                warn(part, ErrorKey::Validation, &msg);
            }
            sc.expect(inscopes, part);
            sc.replace(outscope, part.clone());
        } else if let Some((inscopes, trigger)) = scope_trigger(part, data) {
            if WARN_AGAINST_EQ.contains(&part.as_str()) {
                warn_against_eq = Some(part);
            }
            if !last {
                let msg = format!("`{part}` should be the last part");
                warn(part, ErrorKey::Validation, &msg);
                sc.close();
                return;
            }
            found_trigger = Some((trigger, part.clone()));
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as only part");
                warn(part, ErrorKey::Validation, &msg);
            }
            if part.is("current_year") && sc.scopes() == Scopes::None {
                warn_info(
                    part,
                    ErrorKey::Bugs,
                    "current_year does not work in empty scope",
                    "try using current_date, or dummy_male.current_year",
                );
            } else {
                sc.expect(inscopes, part);
            }
        } else {
            // TODO: warn if trying to use iterator here
            let msg = format!("unknown token `{part}`");
            error(part, ErrorKey::Validation, &msg);
            sc.close();
            return;
        }
    }

    if let Some(token) = warn_against_eq {
        if matches!(cmp, Comparator::Eq | Comparator::QEq) {
            let msg = format!("`{token} {cmp}` means exactly equal to that amount, which is usually not what you want");
            warn(token, ErrorKey::Logic, &msg);
        }
    }

    if let Some((trigger, name)) = found_trigger {
        sc.close();
        match_trigger_bv(&trigger, &name, cmp, bv, data, sc, tooltipped);
        return;
    }

    if !matches!(cmp, Comparator::Eq | Comparator::QEq) {
        if sc.can_be(Scopes::Value) {
            sc.close();
            ScriptValue::validate_bv(bv, data, sc);
        } else {
            let msg = format!("unexpected comparator {cmp}");
            warn(key, ErrorKey::Validation, &msg);
            sc.close();
        }
        return;
    }

    match bv {
        BlockOrValue::Token(t) => {
            let scopes = sc.scopes();
            sc.close();
            validate_target(t, data, sc, scopes);
        }
        BlockOrValue::Block(b) => {
            validate_normal_trigger(b, data, sc, tooltipped);
            sc.close();
        }
    }
}

fn match_trigger_fields(
    fields: &[(&str, Trigger)],
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let mut vd = Validator::new(block, data);
    for (field, _) in fields {
        if let Some(opt) = field.strip_prefix('?') {
            vd.field_any_cmp(opt);
        } else if let Some(mlt) = field.strip_prefix('*') {
            vd.fields_any_cmp(mlt);
        } else {
            vd.req_field(field);
            vd.field_any_cmp(field);
        }
    }

    for (key, cmp, bv) in block.iter_items() {
        if let Some(key) = key {
            for (field, trigger) in fields {
                let fieldname = if let Some(opt) = field.strip_prefix('?') {
                    opt
                } else if let Some(mlt) = field.strip_prefix('*') {
                    mlt
                } else {
                    field
                };
                if key.is(fieldname) {
                    match_trigger_bv(trigger, key, *cmp, bv, data, sc, tooltipped);
                }
            }
        }
    }
}

fn match_trigger_bv(
    trigger: &Trigger,
    name: &Token,
    cmp: Comparator,
    bv: &BlockOrValue,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let mut must_be_eq = true;

    match trigger {
        Trigger::Boolean => {
            if let Some(token) = bv.expect_value() {
                validate_target(token, data, sc, Scopes::Bool);
            }
        }
        Trigger::CompareValue => {
            must_be_eq = false;
            ScriptValue::validate_bv(bv, data, sc);
        }
        Trigger::SetValue => {
            ScriptValue::validate_bv(bv, data, sc);
        }
        Trigger::CompareDate => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                if Date::from_str(token.as_str()).is_err() {
                    let msg = format!("{name} expects a date value");
                    warn(token, ErrorKey::Validation, &msg);
                }
            }
        }
        Trigger::Scope(s) => {
            if let Some(token) = bv.get_value() {
                validate_target(token, data, sc, *s);
            } else if s.contains(Scopes::Value) {
                ScriptValue::validate_bv(bv, data, sc);
            } else {
                bv.expect_value();
            }
        }
        Trigger::Item(i) => {
            if let Some(token) = bv.expect_value() {
                data.verify_exists(*i, token);
            }
        }
        Trigger::ScopeOrItem(s, i) => {
            if let Some(token) = bv.expect_value() {
                if !data.item_exists(*i, token.as_str()) {
                    validate_target(token, data, sc, *s);
                }
            }
        }
        Trigger::Choice(choices) => {
            if let Some(token) = bv.expect_value() {
                if !choices.iter().any(|c| token.is(c)) {
                    let msg = format!("unknown value {token} for {name}");
                    let info = format!("valid values are: {}", stringify_choices(choices));
                    warn_info(token, ErrorKey::Validation, &msg, &info);
                }
            }
        }
        Trigger::Block(fields) => {
            if let Some(block) = bv.expect_block() {
                match_trigger_fields(fields, block, data, sc, tooltipped);
            }
        }
        Trigger::ScopeOrBlock(s, fields) => match bv {
            BlockOrValue::Token(token) => validate_target(token, data, sc, *s),
            BlockOrValue::Block(block) => match_trigger_fields(fields, block, data, sc, tooltipped),
        },
        Trigger::ItemOrBlock(i, fields) => match bv {
            BlockOrValue::Token(token) => data.verify_exists(*i, token),
            BlockOrValue::Block(block) => match_trigger_fields(fields, block, data, sc, tooltipped),
        },
        Trigger::ScopeList(s) => {
            if let Some(block) = bv.expect_block() {
                let mut vd = Validator::new(block, data);
                for token in vd.values() {
                    validate_target(token, data, sc, *s);
                }
            }
        }
        Trigger::ScopeCompare(s) => {
            if let Some(block) = bv.expect_block() {
                if block.iter_items().count() != 1 {
                    let msg = "unexpected number of items in block";
                    warn(block, ErrorKey::Validation, msg);
                }
                for (key, _cmp, bv) in block.iter_items() {
                    if let Some(key) = key {
                        validate_target(key, data, sc, *s);
                        if let Some(token) = bv.expect_value() {
                            validate_target(token, data, sc, *s);
                        }
                    } else {
                        let msg = "unexpected item in block";
                        warn(bv, ErrorKey::Validation, msg);
                    }
                }
            }
        }
        Trigger::CompareToScope(s) => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                validate_target(token, data, sc, *s);
            }
        }
        Trigger::Control => {
            if let Some(block) = bv.expect_block() {
                validate_trigger(name.as_str(), false, block, data, sc, tooltipped);
            }
        }
        Trigger::Special => {
            // exists, switch, time_of_year, custom_tooltip and weighted_calc_true_if
            if name.is("exists") {
                if let Some(token) = bv.expect_value() {
                    if token.is("yes") || token.is("no") {
                        if sc.must_be(Scopes::None) {
                            let msg = "`exists = {token}` does nothing in None scope";
                            warn(token, ErrorKey::Scopes, msg);
                        }
                    } else if token.as_str().starts_with("flag:") {
                        // exists = flag:$REASON$ is used in vanilla just to shut up their error.log,
                        // so accept it silently even though it's a no-op.
                    } else {
                        validate_target(token, data, sc, Scopes::non_primitive());

                        if tooltipped {
                            if let Some(firstpart) = token.as_str().strip_suffix(".holder") {
                                let msg = format!("could rewrite this as `{firstpart} = {{ is_title_created = yes }}`");
                                let info = "it gives a nicer tooltip";
                                advice_info(name, ErrorKey::Tooltip, &msg, info);
                            }
                        }
                    }
                }
            } else if name.is("custom_tooltip") {
                match bv {
                    BlockOrValue::Token(t) => data.verify_exists(Item::Localization, t),
                    BlockOrValue::Block(b) => {
                        validate_trigger(name.as_str(), false, b, data, sc, false);
                    }
                }
            }
        }
        Trigger::UncheckedValue => {
            bv.expect_value();
        }
    }

    if must_be_eq && !matches!(cmp, Comparator::Eq | Comparator::QEq) {
        let msg = format!("unexpected comparator {cmp}");
        warn(name, ErrorKey::Validation, &msg);
    }
}

pub fn validate_target(token: &Token, data: &Everything, sc: &mut ScopeContext, outscopes: Scopes) {
    if token.as_str().parse::<f64>().is_ok() {
        if !outscopes.intersects(Scopes::Value | Scopes::None) {
            let msg = format!("expected {outscopes}");
            warn(token, ErrorKey::Scopes, &msg);
        }
        return;
    }
    let part_vec = token.split('.');
    sc.open_builder();
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];

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
                return;
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
        } else if let Some(inscopes) = trigger_comparevalue(part, data) {
            if !last {
                let msg = format!("`{part}` only makes sense as the last part");
                warn(part, ErrorKey::Scopes, &msg);
                sc.close();
                return;
            }
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as first part");
                warn(part, ErrorKey::Validation, &msg);
            }
            if part.is("current_year") && sc.scopes() == Scopes::None {
                warn_info(
                    part,
                    ErrorKey::Bugs,
                    "current_year does not work in empty scope",
                    "try using current_date, or dummy_male.current_year",
                );
            } else {
                sc.expect(inscopes, part);
            }
            sc.replace(Scopes::Value, part.clone());
        } else if data.scriptvalues.exists(part.as_str()) {
            // TODO: validate inscope of the script value against sc
            if !last {
                let msg = format!("`{part}` only makes sense as the last part");
                warn(part, ErrorKey::Scopes, &msg);
                sc.close();
                return;
            }
            sc.replace(Scopes::Value, part.clone());
        // TODO: warn if trying to use iterator here
        } else {
            let msg = format!("unknown token `{part}`");
            error(part, ErrorKey::Validation, &msg);
            sc.close();
            return;
        }
    }
    let final_scopes = sc.scopes();
    if !outscopes.intersects(final_scopes | Scopes::None) {
        let part = &part_vec[part_vec.len() - 1];
        let msg = format!("`{part}` produces {final_scopes} but expected {outscopes}");
        warn(part, ErrorKey::Scopes, &msg);
    }
    sc.close();
}

// LAST UPDATED VERSION 1.9.0.2
const WARN_AGAINST_EQ: &[&str] = &[
    "gold",
    "prestige",
    "piety",
    "dynasty_prestige",
    "title_held_years",
    "years_as_ruler",
    "culture_age",
    "ghw_war_chest_gold",
    "ghw_war_chest_piety",
    "ghw_war_chest_prestige",
    "available_loot",
    "long_term_gold",
    "long_term_gold_maximum",
    "reserved_gold",
    "reserved_gold_maximum",
    "short_term_gold",
    "short_term_gold_maximum",
    "war_chest_gold",
    "war_chest_gold_maximum",
    "yearly_character_balance",
    "yearly_character_expenses",
    "yearly_character_income",
    "inspiration_gold_invested",
    "memory_age_years",
    "monthly_character_income",
    "monthly_character_income_long_term",
    "monthly_character_income_reserved",
    "monthly_character_income_short_term",
    "monthly_character_income_war_chest",
    "monthly_income",
    "num_total_troops",
    "next_destination_arrival_days",
    "years_as_diarch",
    "years_in_diarchy",
    "title_held_years",
];
