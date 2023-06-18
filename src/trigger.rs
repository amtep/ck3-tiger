use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, Comparator, Date, BV};
use crate::context::ScopeContext;
use crate::data::genes::Gene;
use crate::data::scriptvalues::ScriptValue;
use crate::data::trigger_localization::TriggerLocalization;
use crate::desc::validate_desc;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, warn, warn2, warn_info};
use crate::everything::Everything;
use crate::helpers::stringify_choices;
use crate::item::Item;
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, Scopes};
use crate::tables::triggers::{scope_trigger, trigger_comparevalue, Trigger};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{
    validate_inside_iterator, validate_iterator_fields, validate_prefix_reference, ListType,
};

pub fn validate_normal_trigger(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    validate_trigger("", false, block, data, sc, tooltipped);
}

pub fn validate_trigger(
    caller: &str,
    in_list: bool,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);

    if caller == "custom_description"
        || caller == "custom_tooltip"
        || block.get_field_value("custom_tooltip").is_some()
    {
        tooltipped = Tooltipped::No;
    }

    // limit blocks are accepted in trigger_else even though it doesn't make sense
    if caller == "trigger_if" || caller == "trigger_else_if" || caller == "trigger_else" {
        vd.field_validated_block("limit", |block, data| {
            validate_normal_trigger(block, data, sc, tooltipped);
        });
    } else {
        vd.ban_field("limit", || {
            "`trigger_if`, `trigger_else_if` or `trigger_else`"
        });
    }

    let list_type = if in_list {
        ListType::Any
    } else {
        ListType::None
    };
    validate_iterator_fields(caller, list_type, data, sc, &mut vd, &mut tooltipped);

    if list_type != ListType::None {
        validate_inside_iterator(caller, list_type, block, data, sc, &mut vd, tooltipped);
    }

    // TODO: the custom_description and custom_tooltip logic is duplicated for effects
    if caller == "custom_description" || caller == "custom_tooltip" {
        vd.req_field("text");
        if caller == "custom_tooltip" {
            vd.field_item("text", Item::Localization);
        } else {
            if let Some(token) = vd.field_value("text") {
                data.verify_exists(Item::TriggerLocalization, token);
                if let Some((key, block)) = data
                    .database
                    .get_key_block(Item::TriggerLocalization, token.as_str())
                {
                    TriggerLocalization::validate_use(key, block, data, token, tooltipped);
                }
            }
        }
        vd.field_target("subject", sc, Scopes::non_primitive());
    } else {
        vd.ban_field("text", || "`custom_description` or `custom_tooltip`");
        vd.ban_field("subject", || "`custom_description` or `custom_tooltip`");
    }

    if caller == "custom_description" {
        vd.field_target("object", sc, Scopes::non_primitive());
        vd.field_script_value("value", sc);
    } else {
        vd.ban_field("object", || "`custom_description`");
        vd.ban_field("value", || "`custom_description`");
    }

    if caller == "modifier" {
        vd.fields_script_value("add", sc);
        vd.fields_script_value("factor", sc);
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_validated_block("trigger", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("add", || "`modifier` or script values");
        vd.ban_field("factor", || "`modifier` blocks");
        vd.ban_field("desc", || "`modifier` blocks");
        vd.ban_field("trigger", || "`modifier` blocks");
    }

    if caller == "calc_true_if" {
        vd.req_field("amount");

        // TODO: verify these are integers
        vd.fields_any_cmp("amount");
    } else if !in_list {
        vd.ban_field("amount", || "`calc_true_if`");
    }

    for (key, cmp, bv) in vd.unknown_keys_any_cmp() {
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
                        validate_trigger(it_name.as_str(), true, b, data, sc, tooltipped);
                        sc.close();
                    } else {
                        error(bv, ErrorKey::Validation, "expected block, found value");
                    }
                    continue;
                }
            }
        }

        validate_trigger_key_bv(key, cmp, bv, data, sc, tooltipped);
    }
}

pub fn validate_trigger_key_bv(
    key: &Token,
    cmp: Comparator,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    // Scripted trigger?
    if let Some(trigger) = data.get_trigger(key) {
        match bv {
            BV::Value(token) => {
                if !(token.is("yes") || token.is("no")) {
                    warn(token, ErrorKey::Validation, "expected yes or no");
                }
                if !trigger.macro_parms().is_empty() {
                    error(token, ErrorKey::Macro, "expected macro arguments");
                }
                let tooltipped = if token.is("no") {
                    tooltipped.negated()
                } else {
                    tooltipped
                };
                trigger.validate_call(key, data, sc, tooltipped);
            }
            BV::Block(block) => {
                let parms = trigger.macro_parms();
                if parms.is_empty() {
                    let msg = "trigger does not need macro arguments";
                    error(block, ErrorKey::Macro, msg);
                } else {
                    let mut vec = Vec::new();
                    let mut vd = Validator::new(block, data);
                    for parm in &parms {
                        vd.req_field(parm);
                        if let Some(token) = vd.field_value(parm) {
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
    if key.is_number() {
        ScriptValue::validate_bv(bv, data, sc);
        return;
    }

    let part_vec = key.split('.');
    sc.open_builder();
    let mut found_trigger = None;
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let mut part = &part_vec[i];
        let store_part; // needed for borrow checker

        if let Some((new_part, arg)) = part.split_after('(') {
            if let Some((arg, _)) = arg.split_once(')') {
                let arg = arg.trim();
                if new_part.is("vassal_contract_obligation_level_score(") {
                    validate_target(&arg, data, sc, Scopes::VassalContract);
                } else if new_part.is("squared_distance(") {
                    validate_target(&arg, data, sc, Scopes::Province);
                } else {
                    warn(arg, ErrorKey::Validation, "unexpected argument");
                }
                store_part = new_part;
                part = &store_part;
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
                if prefix.is("scope") {
                    sc.replace_named_scope(arg.as_str(), part);
                } else {
                    sc.replace(outscope, part.clone());
                }
                if prefix.is("event_id") {
                    break; // force last part
                }
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
                sc.replace_prev();
            } else {
                sc.replace_this();
            }
        } else if data.scriptvalues.exists(part.as_str()) {
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

    if let Some((trigger, name)) = found_trigger {
        sc.close();
        match_trigger_bv(&trigger, &name, cmp, bv, data, sc, tooltipped);
        return;
    }

    if !matches!(cmp, Comparator::Eq | Comparator::QEq) {
        if sc.can_be(Scopes::Value) {
            sc.close();
            ScriptValue::validate_bv(bv, data, sc);
        } else if matches!(cmp, Comparator::Ne | Comparator::EEq) {
            let scopes = sc.scopes();
            sc.close();
            if let Some(token) = bv.expect_value() {
                validate_target(token, data, sc, scopes);
            }
        } else {
            let msg = format!("unexpected comparator {cmp}");
            warn(key, ErrorKey::Validation, &msg);
            sc.close();
        }
        return;
    }

    match bv {
        BV::Value(t) => {
            let scopes = sc.scopes();
            sc.close();
            validate_target(t, data, sc, scopes);
        }
        BV::Block(b) => {
            sc.finalize_builder();
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
    tooltipped: Tooltipped,
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
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut must_be_eq = true;
    let mut warn_if_eq = false;

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
        Trigger::CompareValueWarnEq => {
            must_be_eq = false;
            warn_if_eq = true;
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
            BV::Value(token) => validate_target(token, data, sc, *s),
            BV::Block(block) => match_trigger_fields(fields, block, data, sc, tooltipped),
        },
        Trigger::ItemOrBlock(i, fields) => match bv {
            BV::Value(token) => data.verify_exists(*i, token),
            BV::Block(block) => match_trigger_fields(fields, block, data, sc, tooltipped),
        },
        Trigger::CompareValueOrBlock(fields) => match bv {
            BV::Value(t) => {
                validate_target(t, data, sc, Scopes::Value);
                must_be_eq = false;
            }
            BV::Block(b) => {
                match_trigger_fields(fields, b, data, sc, tooltipped);
            }
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
                let mut tooltipped = tooltipped;
                if name.lowercase_is("all_false")
                    || name.lowercase_is("not")
                    || name.lowercase_is("nand")
                    || name.lowercase_is("nor")
                {
                    tooltipped = tooltipped.negated();
                }
                validate_trigger(name.as_str(), false, block, data, sc, tooltipped);
            }
        }
        Trigger::Special => {
            if name.is("exists") {
                if let Some(token) = bv.expect_value() {
                    if token.is("yes") || token.is("no") {
                        if sc.must_be(Scopes::None) {
                            let msg = "`exists = {token}` does nothing in None scope";
                            warn(token, ErrorKey::Scopes, msg);
                        }
                    } else if (token.starts_with("scope:") && !token.as_str().contains('.'))
                        || token.starts_with("flag:")
                    {
                        // exists = scope:name is used to check if that scope name was set
                        // exists = flag:$REASON$ is used in vanilla just to shut up their error.log,
                        // so accept it silently even though it's a no-op.
                    } else {
                        validate_target(token, data, sc, Scopes::non_primitive());

                        if tooltipped.is_tooltipped() {
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
                    BV::Value(t) => data.verify_exists(Item::Localization, t),
                    BV::Block(b) => {
                        validate_trigger(name.as_str(), false, b, data, sc, Tooltipped::No);
                    }
                }
            } else if name.is("has_gene") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.field_item("category", Item::GeneCategory);
                    if let Some(category) = block.get_field_value("category") {
                        if let Some(template) = vd.field_value("template") {
                            Gene::verify_has_template(category.as_str(), template, data);
                        }
                    }
                }
            } else if name.is("save_temporary_opinion_value_as") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("name");
                    vd.req_field("target");
                    vd.field_target("target", sc, Scopes::Character);
                    if let Some(name) = vd.field_value("name") {
                        sc.define_name(name.as_str(), name.clone(), Scopes::Value);
                    }
                }
            } else if name.is("save_temporary_scope_value_as") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("name");
                    vd.req_field("value");
                    vd.field_validated("value", |bv, data| match bv {
                        BV::Value(token) => validate_target(token, data, sc, Scopes::primitive()),
                        BV::Block(_) => ScriptValue::validate_bv(bv, data, sc),
                    });
                    // TODO: figure out the scope type of `value` and use that
                    if let Some(name) = vd.field_value("name") {
                        sc.define_name(name.as_str(), name.clone(), Scopes::primitive());
                    }
                }
            } else if name.is("save_temporary_scope_as") {
                if let Some(name) = bv.expect_value() {
                    sc.save_current_scope(name.as_str());
                }
            } else if name.is("weighted_calc_true_if") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    if let Some(bv) = vd.field_any_cmp("amount") {
                        if let Some(token) = bv.expect_value() {
                            token.expect_number();
                        }
                    }
                    for (_, block) in vd.integer_blocks() {
                        validate_normal_trigger(block, data, sc, tooltipped);
                    }
                }
            } else if name.is("switch") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("trigger");
                    if let Some(target) = vd.field_value("trigger") {
                        let target = target.clone();
                        for (key, bv) in vd.unknown_keys() {
                            if !key.is("fallback") {
                                let synthetic_bv = BV::Value(key.clone());
                                validate_trigger_key_bv(
                                    &target,
                                    Comparator::Eq,
                                    &synthetic_bv,
                                    data,
                                    sc,
                                    tooltipped,
                                );
                            }
                            if let Some(block) = bv.expect_block() {
                                validate_normal_trigger(block, data, sc, tooltipped);
                            }
                        }
                    }
                }
            }
            // TODO: time_of_year
        }
        Trigger::UncheckedValue => {
            bv.expect_value();
        }
    }

    if matches!(cmp, Comparator::Eq | Comparator::QEq | Comparator::EEq) {
        if warn_if_eq {
            let msg = format!("`{name} {cmp}` means exactly equal to that amount, which is usually not what you want");
            warn(name, ErrorKey::Logic, &msg);
        }
    } else if must_be_eq {
        let msg = format!("unexpected comparator {cmp}");
        warn(name, ErrorKey::Validation, &msg);
    }
}

pub fn validate_target(token: &Token, data: &Everything, sc: &mut ScopeContext, outscopes: Scopes) {
    if token.is_number() {
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
        let mut part = &part_vec[i];
        let store_part;

        if let Some((new_part, arg)) = part.split_after('(') {
            if let Some((arg, _)) = arg.split_once(')') {
                let arg = arg.trim();
                if new_part.is("vassal_contract_obligation_level_score(") {
                    validate_target(&arg, data, sc, Scopes::VassalContract);
                } else if new_part.is("squared_distance(") {
                    validate_target(&arg, data, sc, Scopes::Province);
                } else {
                    warn(arg, ErrorKey::Validation, "unexpected argument");
                }
                store_part = new_part;
                part = &store_part;
            }
        }

        if let Some((prefix, mut arg)) = part.split_once(':') {
            if prefix.is("event_id") {
                arg = token.split_once(':').unwrap().1;
            }
            if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{prefix}:` makes no sense except as first part");
                    warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, &prefix);
                validate_prefix_reference(&prefix, &arg, data);
                if prefix.is("scope") {
                    sc.replace_named_scope(arg.as_str(), part);
                } else {
                    sc.replace(outscope, part.clone());
                }
                if prefix.is("event_id") {
                    break; // force last part
                }
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
                sc.replace_prev();
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
        } else if data.scriptvalues.exists(part.as_str()) {
            data.scriptvalues.validate_call(part, data, sc);
            sc.replace(Scopes::Value, part.clone());
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
        // TODO: warn if trying to use iterator here
        } else {
            let msg = format!("unknown token `{part}`");
            error(part, ErrorKey::Validation, &msg);
            sc.close();
            return;
        }
    }
    let (final_scopes, because) = sc.scopes_token();
    if !outscopes.intersects(final_scopes | Scopes::None) {
        let part = &part_vec[part_vec.len() - 1];
        let msg = format!("`{part}` produces {final_scopes} but expected {outscopes}");
        if part != because {
            let msg2 = format!("scope was deduced from `{because}` here");
            warn2(part, ErrorKey::Scopes, &msg, because, &msg2);
        } else {
            warn(part, ErrorKey::Scopes, &msg);
        }
    }
    sc.close();
}
