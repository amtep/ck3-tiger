use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, Date, Comparator, BV};
use crate::context::ScopeContext;
use crate::data::genes::Gene;
use crate::data::trigger_localization::TriggerLocalization;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::helpers::stringify_choices;
use crate::item::Item;
use crate::report::{advice_info, error, warn, warn2, warn_info, ErrorKey};
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, Scopes};
use crate::scriptvalue::validate_scriptvalue;
use crate::tables::triggers::{scope_trigger, trigger_comparevalue, Trigger};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{
    precheck_iterator_fields, validate_ifelse_sequence, validate_inside_iterator,
    validate_iterator_fields, validate_prefix_reference, ListType,
};

pub fn validate_normal_trigger(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    validate_trigger("", false, block, data, sc, tooltipped, false);
}

pub fn validate_trigger(
    caller: &str,
    in_list: bool,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: Tooltipped,
    negated: bool,
) {
    let mut vd = Validator::new(block, data);

    if caller == "custom_description"
        || caller == "custom_tooltip"
        || block.get_field_value("custom_tooltip").is_some()
    {
        tooltipped = Tooltipped::No;
    }

    // If this condition looks weird, it's because the negation from for example NOR has already
    // been applied to the `negated` value.
    if tooltipped == Tooltipped::FailuresOnly
        && ((negated && (caller == "and" || caller == "nand"))
            || (!negated && (caller == "or" || caller == "nor" || caller == "all_false")))
    {
        let true_negated = if caller == "nor" || caller == "all_false" || caller == "and" {
            "negated "
        } else {
            ""
        };
        let msg = format!(
            "{true_negated}{} is a too complex trigger to be tooltipped in a trigger that shows failures only.",
            caller.to_uppercase()
        );
        let info = "Try adding a custom_description or custom_tooltip, or simplifying the trigger";
        warn_info(block, ErrorKey::Tooltip, &msg, info);
    }

    if caller == "trigger_if" || caller == "trigger_else_if" || caller == "trigger_else" {
        if caller != "trigger_else" {
            vd.req_field_warn("limit");
        }
        vd.field_validated_key_block("limit", |key, block, data| {
            if caller == "trigger_else" {
                let msg = "`trigger_else` with a `limit` does work, but may indicate a mistake";
                let info = "normally you would use `trigger_else_if` instead.";
                advice_info(key, ErrorKey::IfElse, msg, info);
            }
            validate_normal_trigger(block, data, sc, Tooltipped::No);
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
        } else if let Some(token) = vd.field_value("text") {
            data.verify_exists(Item::TriggerLocalization, token);
            if let Some((key, block)) = data
                .database
                .get_key_block(Item::TriggerLocalization, token.as_str())
            {
                TriggerLocalization::validate_use(key, block, data, token, tooltipped, negated);
            }
        }
        vd.field_target_ok_this("subject", sc, Scopes::non_primitive());
    } else {
        vd.ban_field("text", || "`custom_description` or `custom_tooltip`");
        vd.ban_field("subject", || "`custom_description` or `custom_tooltip`");
    }

    if caller == "custom_description" {
        // object and value are handled in the loop
    } else {
        vd.ban_field("object", || "`custom_description`");
        vd.ban_field("value", || "`custom_description`");
    }

    if caller == "modifier" {
        // add, factor and desc are handled in the loop
        vd.field_validated_block("trigger", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("add", || "`modifier` or script values");
        vd.ban_field("factor", || "`modifier` blocks");
        vd.ban_field("desc", || "`modifier` or script values");
        vd.ban_field("trigger", || "`modifier` blocks");
    }

    if caller == "calc_true_if" {
        vd.req_field("amount");
        // TODO: verify these are integers
        vd.fields_any_cmp("amount");
    } else if !in_list {
        vd.ban_field("amount", || "`calc_true_if`");
    }

    validate_ifelse_sequence(block, "trigger_if", "trigger_else_if", "trigger_else");

    for (key, cmp, bv) in vd.unknown_fields_any_cmp() {
        if key.is("add") || key.is("factor") || key.is("value") {
            validate_scriptvalue(bv, data, sc);
            continue;
        }

        if key.is("desc") || key.is("DESC") {
            validate_desc(bv, data, sc);
            continue;
        }

        if key.is("object") {
            if let Some(token) = bv.expect_value() {
                validate_target_ok_this(token, data, sc, Scopes::non_primitive());
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
                        precheck_iterator_fields(ListType::Any, b, data, sc);
                        sc.open_scope(outscope, key.clone());
                        validate_trigger(it_name.as_str(), true, b, data, sc, tooltipped, negated);
                        sc.close();
                    } else {
                        error(bv, ErrorKey::Validation, "expected block, found value");
                    }
                    continue;
                }
            }
        }

        validate_trigger_key_bv(key, cmp, bv, data, sc, tooltipped, negated);
    }
}

pub fn validate_trigger_key_bv(
    key: &Token,
    cmp: Comparator,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    negated: bool,
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
                let negated = if token.is("no") { !negated } else { negated };
                trigger.validate_call(key, data, sc, tooltipped, negated);
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
                    trigger.validate_macro_expansion(key, args, data, sc, tooltipped, negated);
                }
            }
        }
        return;
    }

    // `10 < scriptvalue` is a valid trigger
    if key.is_number() {
        validate_scriptvalue(bv, data, sc);
        return;
    }

    let mut new_key = key;
    let mut store;
    if let Some((before, after)) = key.split_after('(') {
        if let Some((arg, after)) = after.split_once(')') {
            let arg = arg.trim();
            for part in before.split('.') {
                if part.as_str().ends_with('(') {
                    if part.is("vassal_contract_obligation_level_score(") {
                        validate_target(&arg, data, sc, Scopes::VassalContract);
                    } else if part.is("squared_distance(") {
                        validate_target(&arg, data, sc, Scopes::Province);
                    } else {
                        warn(&arg, ErrorKey::Validation, "unexpected argument");
                    }
                }
            }
            store = before;
            if !after.as_str().is_empty() {
                store.combine(&after, '.');
            }
            new_key = &store;
        }
    }

    let part_vec = new_key.split('.');
    sc.open_builder();
    let mut found_trigger = None;
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];

        if let Some((prefix, mut arg)) = part.split_once(':') {
            if prefix.is("event_id") {
                arg = new_key.split_once(':').unwrap().1;
            }
            if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{prefix}:` makes no sense except as first part");
                    warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, &prefix);
                validate_prefix_reference(&prefix, &arg, data);
                if prefix.is("scope") {
                    if last && matches!(cmp, Comparator::ConditionalEquals) {
                        // If the comparator is ?=, it's an implicit existence check
                        sc.exists_scope(arg.as_str(), part);
                    }
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
            error(part, ErrorKey::UnknownField, &msg);
            sc.close();
            return;
        }
    }

    if let Some((trigger, name)) = found_trigger {
        sc.close();
        match_trigger_bv(&trigger, &name, cmp, bv, data, sc, tooltipped, negated);
        return;
    }

    if !matches!(cmp, Comparator::Eq | Comparator::ConditionalEquals) {
        if sc.can_be(Scopes::Value) {
            sc.close();
            validate_scriptvalue(bv, data, sc);
        } else if matches!(cmp, Comparator::NotEquals | Comparator::Equals) {
            let scopes = sc.scopes();
            sc.close();
            if let Some(token) = bv.expect_value() {
                validate_target_ok_this(token, data, sc, scopes);
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
            validate_target_ok_this(t, data, sc, scopes);
        }
        BV::Block(b) => {
            sc.finalize_builder();
            validate_trigger("", false, b, data, sc, tooltipped, negated);
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
    negated: bool,
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
                    match_trigger_bv(trigger, key, *cmp, bv, data, sc, tooltipped, negated);
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
    negated: bool,
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
            validate_scriptvalue(bv, data, sc);
        }
        Trigger::CompareValueWarnEq => {
            must_be_eq = false;
            warn_if_eq = true;
            validate_scriptvalue(bv, data, sc);
        }
        Trigger::SetValue => {
            validate_scriptvalue(bv, data, sc);
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
                validate_scriptvalue(bv, data, sc);
            } else {
                bv.expect_value();
            }
        }
        Trigger::ScopeOkThis(s) => {
            if let Some(token) = bv.get_value() {
                validate_target_ok_this(token, data, sc, *s);
            } else if s.contains(Scopes::Value) {
                validate_scriptvalue(bv, data, sc);
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
                match_trigger_fields(fields, block, data, sc, tooltipped, negated);
            }
        }
        Trigger::ScopeOrBlock(s, fields) => match bv {
            BV::Value(token) => validate_target(token, data, sc, *s),
            BV::Block(block) => match_trigger_fields(fields, block, data, sc, tooltipped, negated),
        },
        Trigger::ItemOrBlock(i, fields) => match bv {
            BV::Value(token) => data.verify_exists(*i, token),
            BV::Block(block) => match_trigger_fields(fields, block, data, sc, tooltipped, negated),
        },
        Trigger::CompareValueOrBlock(fields) => match bv {
            BV::Value(t) => {
                validate_target(t, data, sc, Scopes::Value);
                must_be_eq = false;
            }
            BV::Block(b) => {
                match_trigger_fields(fields, b, data, sc, tooltipped, negated);
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
                let mut negated = negated;
                if name.lowercase_is("all_false")
                    || name.lowercase_is("not")
                    || name.lowercase_is("nand")
                    || name.lowercase_is("nor")
                {
                    negated = !negated;
                }
                validate_trigger(
                    &name.as_str().to_lowercase(),
                    false,
                    block,
                    data,
                    sc,
                    tooltipped,
                    negated,
                );
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
                    } else if token.starts_with("scope:") && !token.as_str().contains('.') {
                        // exists = scope:name is used to check if that scope name was set
                        if !negated {
                            sc.exists_scope(token.as_str().strip_prefix("scope:").unwrap(), name);
                        }
                    } else if token.starts_with("flag:") {
                        // exists = flag:$REASON$ is used in vanilla just to shut up their error.log,
                        // so accept it silently even though it's a no-op.
                    } else {
                        validate_target_ok_this(token, data, sc, Scopes::non_primitive());

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
                        validate_trigger(
                            name.as_str(),
                            false,
                            b,
                            data,
                            sc,
                            Tooltipped::No,
                            negated,
                        );
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
                        sc.define_name(name.as_str(), Scopes::Value, name);
                    }
                }
            } else if name.is("save_temporary_scope_value_as") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.req_field("name");
                    vd.req_field("value");
                    vd.field_validated("value", |bv, data| match bv {
                        BV::Value(token) => validate_target(token, data, sc, Scopes::primitive()),
                        BV::Block(_) => validate_scriptvalue(bv, data, sc),
                    });
                    // TODO: figure out the scope type of `value` and use that
                    if let Some(name) = vd.field_value("name") {
                        sc.define_name(name.as_str(), Scopes::primitive(), name);
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
                        for (key, block) in vd.unknown_block_fields() {
                            if !key.is("fallback") {
                                let synthetic_bv = BV::Value(key.clone());
                                validate_trigger_key_bv(
                                    &target,
                                    Comparator::Eq,
                                    &synthetic_bv,
                                    data,
                                    sc,
                                    tooltipped,
                                    negated,
                                );
                            }
                            validate_normal_trigger(block, data, sc, tooltipped);
                        }
                    }
                }
            } else if name.is("add_to_temporary_list") {
                if let Some(value) = bv.expect_value() {
                    sc.define_or_expect_list(value);
                }
            } else if name.is("is_in_list") {
                if let Some(value) = bv.expect_value() {
                    sc.expect_list(value);
                }
            }
            // TODO: time_of_year
        }
        Trigger::UncheckedValue => {
            bv.expect_value();
        }
    }

    if matches!(cmp, Comparator::Eq | Comparator::ConditionalEquals | Comparator::Equals) {
        if warn_if_eq {
            let msg = format!("`{name} {cmp}` means exactly equal to that amount, which is usually not what you want");
            warn(name, ErrorKey::Logic, &msg);
        }
    } else if must_be_eq {
        let msg = format!("unexpected comparator {cmp}");
        warn(name, ErrorKey::Validation, &msg);
    }
}

pub fn validate_target_ok_this(
    token: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    outscopes: Scopes,
) {
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
            error(part, ErrorKey::UnknownField, &msg);
            sc.close();
            return;
        }
    }
    let (final_scopes, because) = sc.scopes_token();
    if !outscopes.intersects(final_scopes | Scopes::None) {
        let part = &part_vec[part_vec.len() - 1];
        let msg = format!("`{part}` produces {final_scopes} but expected {outscopes}");
        if part == because {
            warn(part, ErrorKey::Scopes, &msg);
        } else {
            let msg2 = format!("scope was deduced from `{because}` here");
            warn2(part, ErrorKey::Scopes, &msg, because, &msg2);
        }
    }
    sc.close();
}

pub fn validate_target(token: &Token, data: &Everything, sc: &mut ScopeContext, outscopes: Scopes) {
    validate_target_ok_this(token, data, sc, outscopes);
    if token.is("this") {
        let msg = "target `this` makes no sense here";
        warn(token, ErrorKey::UseOfThis, msg);
    }
}
