use std::borrow::Cow;
use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, Comparator, Eq::*, Field, BV};
#[cfg(feature = "ck3")]
use crate::ck3::tables::triggers::scope_trigger;
use crate::context::{Reason, ScopeContext};
use crate::data::genes::Gene;
use crate::data::trigger_localization::TriggerLocalization;
use crate::date::Date;
use crate::desc::validate_desc;
use crate::everything::Everything;
#[cfg(feature = "vic3")]
use crate::everything::{APPROVALS, LEVELS};
use crate::helpers::stringify_choices;
#[cfg(feature = "vic3")]
use crate::helpers::stringify_list;
#[cfg(feature = "imperator")]
use crate::imperator::tables::triggers::scope_trigger;
use crate::item::Item;
use crate::report::{
    advice_info, err, error, fatal, old_warn, warn2, warn_info, ErrorKey, Severity,
};
use crate::scopes::{
    scope_iterator, scope_prefix, scope_to_scope, validate_prefix_reference, Scopes,
};
use crate::scriptvalue::validate_scriptvalue;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{
    precheck_iterator_fields, validate_ifelse_sequence, validate_inside_iterator,
    validate_iterator_fields, ListType,
};
#[cfg(feature = "vic3")]
use crate::vic3::tables::triggers::scope_trigger;

/// The standard interface to trigger validation. Validates a trigger in the given [`ScopeContext`].
///
/// `tooltipped` determines what warnings are emitted related to tooltippability of the triggers
/// inside the block.
pub fn validate_trigger(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    validate_trigger_internal("", false, block, data, sc, tooltipped, false, Severity::Error);
}

/// Like [`validate_trigger`] but specifies a maximum [`Severity`] for the reports emitted by this
/// validation. Used to validate triggers in item definitions that don't warrant the `Error` level.
pub fn validate_trigger_max_sev(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    max_sev: Severity,
) {
    validate_trigger_internal("", false, block, data, sc, tooltipped, false, max_sev);
}

/// The interface to trigger validation when [`validate_trigger`] is too limited.
///
/// `caller` is the key that opened this trigger. It is used to determine which special cases apply.
/// For example, if `caller` is `trigger_if` then a `limit` block is expected.
///
/// `in_list` specifies whether this trigger is directly in an `any_` iterator. It is also used to
/// determine which special cases apply.
///
/// `negated` is true iff this trigger is tested in a negative sense, for example if it is
/// somewhere inside a `NOT = { ... }` block. `negated` is propagated to all sub-blocks and is
/// flipped when another `NOT` or similar is encountered inside this one.
///
/// TODO: `in_list` could be removed if the code checks directly for the `any_` prefix instead.
pub fn validate_trigger_internal(
    caller: &str,
    in_list: bool,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: Tooltipped,
    negated: bool,
    max_sev: Severity,
) {
    let mut vd = Validator::new(block, data);
    vd.set_max_severity(max_sev);

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
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("limit", || "`trigger_if`, `trigger_else_if` or `trigger_else`");
    }

    if in_list {
        vd.field_validated_block("filter", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("filter", || "lists");
    }

    let list_type = if in_list { ListType::Any } else { ListType::None };
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
            data.verify_exists_max_sev(Item::TriggerLocalization, token, max_sev);
            if let Some((key, block)) =
                data.get_key_block(Item::TriggerLocalization, token.as_str())
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
            validate_trigger(block, data, sc, Tooltipped::No);
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

    vd.unknown_fields_any_cmp(|key, cmp, bv| {
        if key.is("add") || key.is("factor") || key.is("value") {
            validate_scriptvalue(bv, data, sc);
            return;
        }

        if key.is("desc") || key.is("DESC") {
            validate_desc(bv, data, sc);
            return;
        }

        if key.is("object") {
            if let Some(token) = bv.expect_value() {
                validate_target_ok_this(token, data, sc, Scopes::non_primitive());
            }
            return;
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
                        return;
                    }
                    sc.expect(inscopes, &Reason::Token(key.clone()));
                    if let Some(b) = bv.get_block() {
                        precheck_iterator_fields(ListType::Any, b, data, sc);
                        sc.open_scope(outscope, key.clone());
                        validate_trigger_internal(
                            it_name.as_str(),
                            true,
                            b,
                            data,
                            sc,
                            tooltipped,
                            negated,
                            max_sev,
                        );
                        sc.close();
                    } else {
                        error(bv, ErrorKey::Validation, "expected block, found value");
                    }
                    return;
                }
            }
        }

        validate_trigger_key_bv(key, cmp, bv, data, sc, tooltipped, negated, max_sev);
    });
}

/// Validate a trigger given its key and argument. It is like [`validate_trigger_internal`] except
/// that all special cases are assumed to have been handled. This is the interface used for the
/// `switch` effect, where the key and argument are not together in the script.
pub fn validate_trigger_key_bv(
    key: &Token,
    cmp: Comparator,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    negated: bool,
    max_sev: Severity,
) {
    // Scripted trigger?
    if let Some(trigger) = data.get_trigger(key) {
        match bv {
            BV::Value(token) => {
                if !(token.is("yes") || token.is("no") || token.is("YES") || token.is("NO")) {
                    old_warn(token, ErrorKey::Validation, "expected yes or no");
                }
                if !trigger.macro_parms().is_empty() {
                    fatal(ErrorKey::Macro).msg("expected macro arguments").loc(token).push();
                    return;
                }
                let negated = if token.is("no") { !negated } else { negated };
                trigger.validate_call(key, data, sc, tooltipped, negated);
            }
            BV::Block(block) => {
                let parms = trigger.macro_parms();
                if parms.is_empty() {
                    let msg = "this scripted trigger does not need macro arguments";
                    fatal(ErrorKey::Macro).msg(msg).loc(block).push();
                } else {
                    let mut vec = Vec::new();
                    let mut vd = Validator::new(block, data);
                    vd.set_max_severity(max_sev);
                    for parm in &parms {
                        if let Some(token) = vd.field_value(parm) {
                            vec.push(token.clone());
                        } else {
                            let msg = format!("this scripted trigger needs parameter {parm}");
                            err(ErrorKey::Macro).msg(msg).loc(block).push();
                            return;
                        }
                    }
                    vd.unknown_value_fields(|key, _value| {
                        let msg = format!("this scripted trigger does not need parameter {key}");
                        let info = "supplying an unneeded parameter often causes a crash";
                        fatal(ErrorKey::Macro).msg(msg).info(info).loc(key).push();
                    });

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

    let key = handle_argument(key, data, sc);
    let part_vec = key.split('.');
    sc.open_builder();
    let mut found_trigger = None;
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];

        if let Some((prefix, mut arg)) = part.split_once(':') {
            if prefix.is("event_id") {
                arg = key.split_once(':').unwrap().1;
            }
            if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{prefix}:` makes no sense except as first part");
                    old_warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, &Reason::Token(prefix.clone()));
                validate_prefix_reference(&prefix, &arg, data, sc);
                if prefix.is("scope") {
                    if last && matches!(cmp, Comparator::Equals(Question)) {
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
                old_warn(part, ErrorKey::Validation, &msg);
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
        } else if let Some((inscopes, outscope)) = scope_to_scope(part, sc.scopes()) {
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as first part");
                old_warn(part, ErrorKey::Validation, &msg);
            }
            sc.expect(inscopes, &Reason::Token(part.clone()));
            sc.replace(outscope, part.clone());
        } else if let Some((inscopes, trigger)) = scope_trigger(part, data) {
            if !last {
                let msg = format!("`{part}` should be the last part");
                old_warn(part, ErrorKey::Validation, &msg);
                sc.close();
                return;
            }
            found_trigger = Some((trigger, part.clone()));
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as only part");
                old_warn(part, ErrorKey::Validation, &msg);
            }
            if part.is("current_year") && sc.scopes() == Scopes::None {
                warn_info(
                    part,
                    ErrorKey::Bugs,
                    "current_year does not work in empty scope",
                    "try using current_date, or dummy_male.current_year",
                );
            } else {
                sc.expect(inscopes, &Reason::Token(part.clone()));
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
        match_trigger_bv(&trigger, &name, cmp, bv, data, sc, tooltipped, negated, max_sev);
        return;
    }

    if !matches!(cmp, Comparator::Equals(Single | Question)) {
        if sc.can_be(Scopes::Value) {
            sc.close();
            validate_scriptvalue(bv, data, sc);
        } else if matches!(cmp, Comparator::NotEquals | Comparator::Equals(Double)) {
            let scopes = sc.scopes();
            sc.close();
            if let Some(token) = bv.expect_value() {
                validate_target_ok_this(token, data, sc, scopes);
            }
        } else {
            let msg = format!("unexpected comparator {cmp}");
            old_warn(key.into_owned(), ErrorKey::Validation, &msg);
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
            validate_trigger_internal("", false, b, data, sc, tooltipped, negated, max_sev);
            sc.close();
        }
    }
}

/// Implementation of the [`Trigger::Block`] variant and its friends. It takes a list of known
/// fields and their own `Trigger` validators, and checks that the given `block` contains only
/// fields from that list and validates them.
///
/// The field names may have a prefix to indicate how they are to be used.
/// * `?` means the field is optional
/// * `*` means the field is optional and may occur multiple times
/// * `+` means the field is required and may occur multiple times
/// The default is that the field is required and may occur only once.
fn match_trigger_fields(
    fields: &[(&str, Trigger)],
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    negated: bool,
    max_sev: Severity,
) {
    let mut vd = Validator::new(block, data);
    vd.set_max_severity(max_sev);
    for (field, _) in fields {
        if let Some(opt) = field.strip_prefix('?') {
            vd.field_any_cmp(opt);
        } else if let Some(mlt) = field.strip_prefix('*') {
            vd.fields_any_cmp(mlt);
        } else if let Some(mlt) = field.strip_prefix('+') {
            vd.req_field(mlt);
            vd.fields_any_cmp(mlt);
        } else {
            vd.req_field(field);
            vd.field_any_cmp(field);
        }
    }

    for Field(key, cmp, bv) in block.iter_fields() {
        for (field, trigger) in fields {
            let fieldname = if let Some(opt) = field.strip_prefix('?') {
                opt
            } else if let Some(mlt) = field.strip_prefix('*') {
                mlt
            } else if let Some(mlt) = field.strip_prefix('+') {
                mlt
            } else {
                field
            };
            if key.is(fieldname) {
                match_trigger_bv(trigger, key, *cmp, bv, data, sc, tooltipped, negated, max_sev);
            }
        }
    }
}

#[cfg(feature = "vic3")]
pub const STANCES: &[&str] =
    &["strongly_disapprove", "disapprove", "neutral", "approve", "strongly_approve"];

/// Takes a [`Trigger`] and a trigger field, and validates that the constraints
/// specified by the `Trigger` hold.
fn match_trigger_bv(
    trigger: &Trigger,
    name: &Token,
    cmp: Comparator,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    negated: bool,
    max_sev: Severity,
) {
    // True iff the comparator must be Comparator::Equals
    let mut must_be_eq = true;
    // True iff it's probably a mistake if the comparator is Comparator::Equals
    #[cfg(feature = "ck3")]
    let mut warn_if_eq = false;
    #[cfg(any(feature = "imperator", feature = "vic3"))]
    let warn_if_eq = false;

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
        #[cfg(feature = "ck3")]
        Trigger::CompareValueWarnEq => {
            must_be_eq = false;
            warn_if_eq = true;
            validate_scriptvalue(bv, data, sc);
        }
        #[cfg(feature = "ck3")]
        Trigger::SetValue => {
            validate_scriptvalue(bv, data, sc);
        }
        Trigger::CompareDate => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                if Date::from_str(token.as_str()).is_err() {
                    let msg = format!("{name} expects a date value");
                    old_warn(token, ErrorKey::Validation, &msg);
                }
            }
        }
        #[cfg(feature = "vic3")]
        Trigger::CompareLevel => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                if !LEVELS.contains(&token.as_str()) {
                    let msg = format!("{name} expects one of {}", stringify_list(LEVELS));
                    old_warn(token, ErrorKey::Validation, &msg);
                }
            }
        }
        #[cfg(feature = "vic3")]
        Trigger::CompareStance => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                if !STANCES.contains(&token.as_str()) {
                    let msg = format!("{name} expects one of {}", stringify_list(STANCES));
                    old_warn(token, ErrorKey::Validation, &msg);
                }
            }
        }
        #[cfg(feature = "vic3")]
        Trigger::CompareApproval => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                if !token.is_number() && !APPROVALS.contains(&token.as_str()) {
                    let msg = format!("{name} expects one of {}", stringify_list(APPROVALS));
                    old_warn(token, ErrorKey::Validation, &msg);
                }
            }
        }
        #[cfg(feature = "vic3")]
        Trigger::ItemOrCompareValue(i) => {
            if let Some(token) = bv.expect_value() {
                if !data.item_exists(*i, token.as_str()) {
                    must_be_eq = false;
                    validate_target(token, data, sc, Scopes::Value);
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
                data.verify_exists_max_sev(*i, token, max_sev);
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
                match_trigger_fields(fields, block, data, sc, tooltipped, negated, max_sev);
            }
        }
        #[cfg(feature = "ck3")]
        Trigger::ScopeOrBlock(s, fields) => match bv {
            BV::Value(token) => validate_target(token, data, sc, *s),
            BV::Block(block) => {
                match_trigger_fields(fields, block, data, sc, tooltipped, negated, max_sev)
            }
        },
        #[cfg(feature = "ck3")]
        Trigger::ItemOrBlock(i, fields) => match bv {
            BV::Value(token) => data.verify_exists_max_sev(*i, token, max_sev),
            BV::Block(block) => {
                match_trigger_fields(fields, block, data, sc, tooltipped, negated, max_sev)
            }
        },
        #[cfg(feature = "ck3")]
        Trigger::CompareValueOrBlock(fields) => match bv {
            BV::Value(t) => {
                validate_target(t, data, sc, Scopes::Value);
                must_be_eq = false;
            }
            BV::Block(b) => {
                match_trigger_fields(fields, b, data, sc, tooltipped, negated, max_sev);
            }
        },
        #[cfg(feature = "ck3")]
        Trigger::ScopeList(s) => {
            if let Some(block) = bv.expect_block() {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(max_sev);
                for token in vd.values() {
                    validate_target(token, data, sc, *s);
                }
            }
        }
        #[cfg(feature = "ck3")]
        Trigger::ScopeCompare(s) => {
            if let Some(block) = bv.expect_block() {
                if block.iter_items().count() != 1 {
                    let msg = "unexpected number of items in block";
                    old_warn(block, ErrorKey::Validation, msg);
                }
                for Field(key, _, bv) in block.iter_fields_warn() {
                    validate_target(key, data, sc, *s);
                    if let Some(token) = bv.expect_value() {
                        validate_target(token, data, sc, *s);
                    }
                }
            }
        }
        #[cfg(feature = "ck3")]
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
                let mut tooltipped = tooltipped;
                if name.lowercase_is("custom_description") {
                    tooltipped = Tooltipped::No;
                }
                validate_trigger_internal(
                    &name.as_str().to_lowercase(),
                    false,
                    block,
                    data,
                    sc,
                    tooltipped,
                    negated,
                    max_sev,
                );
            }
        }
        Trigger::Special => {
            if name.is("exists") {
                if let Some(token) = bv.expect_value() {
                    if token.is("yes") || token.is("no") {
                        if sc.must_be(Scopes::None) {
                            let msg = "`exists = {token}` does nothing in None scope";
                            old_warn(token, ErrorKey::Scopes, msg);
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
                    BV::Value(t) => data.verify_exists_max_sev(Item::Localization, t, max_sev),
                    BV::Block(b) => {
                        validate_trigger_internal(
                            name.as_str(),
                            false,
                            b,
                            data,
                            sc,
                            Tooltipped::No,
                            negated,
                            max_sev,
                        );
                    }
                }
            } else if name.is("has_gene") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_max_severity(max_sev);
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
                    vd.set_max_severity(max_sev);
                    vd.req_field("name");
                    vd.req_field("target");
                    vd.field_target("target", sc, Scopes::Character);
                    if let Some(name) = vd.field_value("name") {
                        sc.define_name_token(name.as_str(), Scopes::Value, name);
                    }
                }
            } else if name.is("save_temporary_scope_value_as") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_max_severity(max_sev);
                    vd.req_field("name");
                    vd.req_field("value");
                    vd.field_validated("value", |bv, data| match bv {
                        BV::Value(token) => validate_target(token, data, sc, Scopes::primitive()),
                        BV::Block(_) => validate_scriptvalue(bv, data, sc),
                    });
                    // TODO: figure out the scope type of `value` and use that
                    if let Some(name) = vd.field_value("name") {
                        sc.define_name_token(name.as_str(), Scopes::primitive(), name);
                    }
                }
            } else if name.is("save_temporary_scope_as") {
                if let Some(name) = bv.expect_value() {
                    sc.save_current_scope(name.as_str());
                }
            } else if name.is("weighted_calc_true_if") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_max_severity(max_sev);
                    if let Some(bv) = vd.field_any_cmp("amount") {
                        if let Some(token) = bv.expect_value() {
                            token.expect_number();
                        }
                    }
                    for (_, block) in vd.integer_blocks() {
                        validate_trigger(block, data, sc, tooltipped);
                    }
                }
            } else if name.is("switch") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_max_severity(max_sev);
                    vd.req_field("trigger");
                    if let Some(target) = vd.field_value("trigger") {
                        let target = target.clone();
                        vd.unknown_block_fields(|key, block| {
                            if !key.is("fallback") {
                                let synthetic_bv = BV::Value(key.clone());
                                validate_trigger_key_bv(
                                    &target,
                                    Comparator::Equals(Single),
                                    &synthetic_bv,
                                    data,
                                    sc,
                                    tooltipped,
                                    negated,
                                    max_sev,
                                );
                            }
                            validate_trigger(block, data, sc, tooltipped);
                        });
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

    if matches!(cmp, Comparator::Equals(_)) {
        if warn_if_eq {
            let msg = format!("`{name} {cmp}` means exactly equal to that amount, which is usually not what you want");
            old_warn(name, ErrorKey::Logic, &msg);
        }
    } else if must_be_eq {
        let msg = format!("unexpected comparator {cmp}");
        old_warn(name, ErrorKey::Validation, &msg);
    }
}

/// Validate that `token` is valid as the right-hand side of a field.
///
/// `outscopes` is the set of scope types that this target is allowed to produce.
/// * Example: in `has_claim_on = title:e_byzantium`, the target is `title:e_byzantium` and it
/// should produce a [`Scopes::LandedTitle`] scope in order to be valid for `has_claim_on`.
pub fn validate_target_ok_this(
    token: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    outscopes: Scopes,
) {
    if token.is_number() {
        if !outscopes.intersects(Scopes::Value | Scopes::None) {
            let msg = format!("expected {outscopes}");
            old_warn(token, ErrorKey::Scopes, &msg);
        }
        return;
    }
    let token = handle_argument(token, data, sc);
    let part_vec = token.split('.');
    sc.open_builder();
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];

        if let Some((prefix, mut arg)) = part.split_once(':') {
            if prefix.is("event_id") {
                arg = token.split_once(':').unwrap().1;
            }
            if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{prefix}:` makes no sense except as first part");
                    old_warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, &Reason::Token(prefix.clone()));
                validate_prefix_reference(&prefix, &arg, data, sc);
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
                old_warn(part, ErrorKey::Validation, &msg);
            }
            if part.lowercase_is("root") {
                sc.replace_root();
            } else if part.lowercase_is("prev") {
                sc.replace_prev();
            } else {
                sc.replace_this();
            }
        } else if let Some((inscopes, outscope)) = scope_to_scope(part, sc.scopes()) {
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as first part");
                old_warn(part, ErrorKey::Validation, &msg);
            }
            sc.expect(inscopes, &Reason::Token(part.clone()));
            sc.replace(outscope, part.clone());
        } else if data.scriptvalues.exists(part.as_str()) {
            data.scriptvalues.validate_call(part, data, sc);
            sc.replace(Scopes::Value, part.clone());
        } else if let Some(inscopes) = trigger_comparevalue(part, data) {
            if !last {
                let msg = format!("`{part}` only makes sense as the last part");
                old_warn(part, ErrorKey::Scopes, &msg);
                sc.close();
                return;
            }
            if inscopes == Scopes::None && !first {
                let msg = format!("`{part}` makes no sense except as first part");
                old_warn(part, ErrorKey::Validation, &msg);
            }
            if part.is("current_year") && sc.scopes() == Scopes::None {
                warn_info(
                    part,
                    ErrorKey::Bugs,
                    "current_year does not work in empty scope",
                    "try using current_date, or dummy_male.current_year",
                );
            } else {
                sc.expect(inscopes, &Reason::Token(part.clone()));
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
    let (final_scopes, because) = sc.scopes_reason();
    if !outscopes.intersects(final_scopes | Scopes::None) {
        let part = &part_vec[part_vec.len() - 1];
        let msg = format!("`{part}` produces {final_scopes} but expected {outscopes}");
        if part == because.token() && part.loc == because.token().loc {
            old_warn(part, ErrorKey::Scopes, &msg);
        } else {
            let msg2 = format!("scope was {}", because.msg());
            warn2(part, ErrorKey::Scopes, &msg, because.token(), &msg2);
        }
    }
    sc.close();
}

/// Just like [`validate_target_ok_this`], but warns if the target is a literal `this` because that
/// is usually a mistake.
pub fn validate_target(token: &Token, data: &Everything, sc: &mut ScopeContext, outscopes: Scopes) {
    validate_target_ok_this(token, data, sc, outscopes);
    if token.is("this") {
        let msg = "target `this` makes no sense here";
        old_warn(token, ErrorKey::UseOfThis, msg);
    }
}

/// This function is for keys that use the unusual syntax `"scope:province.squared_distance(scope:other)"`
/// The function will extract the argument from between the parentheses and validate it.
/// It will return the key without this argument, or return the key unchanged if there wasn't any.
/// When deleting the argument, it will leave the '(' in place to signal that there was an argument here.
fn handle_argument<'a>(key: &'a Token, data: &Everything, sc: &mut ScopeContext) -> Cow<'a, Token> {
    if let Some((before, after)) = key.split_after('(') {
        if let Some((arg, after)) = after.split_once(')') {
            let arg = arg.trim();
            for part in before.split('.') {
                if part.as_str().ends_with('(') {
                    #[cfg(feature = "ck3")]
                    if part.is("vassal_contract_obligation_level_score(") {
                        validate_target(&arg, data, sc, Scopes::VassalContract);
                    } else if part.is("squared_distance(") {
                        validate_target(&arg, data, sc, Scopes::Province);
                    } else {
                        let msg = "unexpected argument";
                        err(ErrorKey::Validation).weak().msg(msg).loc(&arg).push();
                    }
                    // TODO there's got to be a better way to do this
                    #[cfg(feature = "vic3")]
                    if part.is("ai_army_comparison(")
                        || part.is("ai_gdp_comparison(")
                        || part.is("ai_ideological_opinion(")
                        || part.is("ai_navy_comparison(")
                        || part.is("average_defense(")
                        || part.is("average_offense(")
                        || part.is("diplomatic_pact_other_country(")
                        || part.is("num_total_battalions(")
                        || part.is("num_defending_battalions(")
                        || part.is("tension(")
                        || part.is("num_alliances_and_defensive_pacts_with_allies(")
                        || part.is("num_alliances_and_defensive_pacts_with_rivals(")
                        || part.is("num_mutual_trade_route_levels_with_country(")
                        || part.is("relations(")
                    {
                        validate_target(&arg, data, sc, Scopes::Country);
                    } else if part.is("num_enemy_units(") {
                        validate_target(&arg, data, sc, Scopes::Character);
                    } else {
                        let msg = "unexpected argument";
                        err(ErrorKey::Validation).weak().msg(msg).loc(&arg).push();
                    }
                }
            }
            let mut new_key = before;
            if !after.as_str().is_empty() {
                new_key.combine(&after, '.');
            }
            return Cow::Owned(new_key);
        }
    }
    Cow::Borrowed(key)
}

/// A description of the constraints on the right-hand side of a given trigger.
/// In other words, how it can be used.
///
/// It is used recursively in variants like [`Trigger::Block`], where each of the sub fields have
/// their own `Trigger`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Trigger {
    /// trigger = no or trigger = yes
    Boolean,
    /// can be a script value
    CompareValue,
    /// can be a script value; warn if =
    #[cfg(feature = "ck3")]
    CompareValueWarnEq,
    /// can be a script value; no < or >
    #[cfg(feature = "ck3")]
    SetValue,
    /// value must be a valid date
    CompareDate,
    /// value is a level from LEVELS array
    #[cfg(feature = "vic3")]
    CompareLevel,
    /// value is a stance from `strongly_disapprove` to `strongly_approve`
    #[cfg(feature = "vic3")]
    CompareStance,
    /// value is a number, or a token from `angry` to `loyal`
    #[cfg(feature = "vic3")]
    CompareApproval,
    /// trigger is either = item or compared to another trigger
    #[cfg(feature = "vic3")]
    ItemOrCompareValue(Item),
    /// trigger is compared to a scope object
    Scope(Scopes),
    /// trigger is compared to a scope object which may be `this`
    ScopeOkThis(Scopes),
    /// value is chosen from an item type
    Item(Item),
    ScopeOrItem(Scopes, Item),
    /// value is chosen from a list given here
    Choice(&'static [&'static str]),
    /// For Block, if a field name in the array starts with ? it means that field is optional
    /// trigger takes a block with these fields
    Block(&'static [(&'static str, Trigger)]),
    /// trigger takes a block with these fields
    #[cfg(feature = "ck3")]
    ScopeOrBlock(Scopes, &'static [(&'static str, Trigger)]),
    /// trigger takes a block with these fields
    #[cfg(feature = "ck3")]
    ItemOrBlock(Item, &'static [(&'static str, Trigger)]),
    /// can be part of a scope chain but also a standalone trigger
    #[cfg(feature = "ck3")]
    CompareValueOrBlock(&'static [(&'static str, Trigger)]),
    /// trigger takes a block of values of this scope type
    #[cfg(feature = "ck3")]
    ScopeList(Scopes),
    /// trigger takes a block comparing two scope objects
    #[cfg(feature = "ck3")]
    ScopeCompare(Scopes),
    /// this is for inside a Block, where a key is compared to a scope object
    #[cfg(feature = "ck3")]
    CompareToScope(Scopes),

    /// this key opens another trigger block
    Control,
    /// this has specific code for validation
    Special,

    UncheckedValue,
}

/// This function checks if the trigger is one that can be used at the end of a scope chain on the
/// right-hand side of a comparator.
///
/// Only triggers that take `Scopes::Value` types can be used this way.
pub fn trigger_comparevalue(name: &Token, data: &Everything) -> Option<Scopes> {
    match scope_trigger(name, data) {
        #[cfg(feature = "ck3")]
        Some((
            s,
            Trigger::CompareValue
            | Trigger::CompareValueWarnEq
            | Trigger::CompareDate
            | Trigger::SetValue
            | Trigger::CompareValueOrBlock(_),
        )) => Some(s),
        #[cfg(feature = "vic3")]
        Some((
            s,
            Trigger::CompareValue | Trigger::CompareDate | Trigger::ItemOrCompareValue(_),
        )) => Some(s),
        _ => std::option::Option::None,
    }
}
