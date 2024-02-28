//! Validate triggers, which are parts of the script that specify yes or no conditions.

use bitflags::bitflags;

use std::str::FromStr;

use crate::block::{Block, Comparator, Eq::*, Field, BV};
use crate::context::{Reason, ScopeContext};
use crate::data::genes::Gene;
use crate::data::trigger_localization::TriggerLocalization;
use crate::date::Date;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::Game;
use crate::helpers::stringify_choices;
use crate::item::Item;
use crate::lowercase::Lowercase;
#[cfg(feature = "vic3")]
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::report::{err, fatal, tips, warn, ErrorKey, Severity};
use crate::scopes::{
    needs_prefix, scope_iterator, scope_prefix, scope_to_scope, ArgumentValue, Scopes,
};
use crate::script_value::validate_script_value;
use crate::token::{Loc, Token};
use crate::tooltipped::Tooltipped;
use crate::validate::{
    precheck_iterator_fields, validate_ifelse_sequence, validate_inside_iterator,
    validate_iterator_fields, ListType,
};
use crate::validator::Validator;

/// Look up a trigger token that evaluates to a trigger value.
///
/// `name` is the token. `data` is used in special cases to verify the name dynamically,
/// for example, `<lifestyle>_xp` is only a valid trigger if `<lifestyle>` is present in
/// the database.
///
/// Returns the inscopes valid for the trigger and the output trigger value type.
pub fn scope_trigger(name: &Token, data: &Everything) -> Option<(Scopes, Trigger)> {
    let scope_trigger = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::triggers::scope_trigger,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::triggers::scope_trigger,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::triggers::scope_trigger,
    };
    scope_trigger(name, data)
}

/// The standard interface to trigger validation. Validates a trigger in the given [`ScopeContext`].
///
/// `tooltipped` determines what warnings are emitted related to tooltippability of the triggers
/// inside the block.
///
/// Returns true iff the trigger had side effects (such as saving scopes).
pub fn validate_trigger(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) -> bool {
    validate_trigger_internal(
        Lowercase::empty(),
        false,
        block,
        data,
        sc,
        tooltipped,
        false,
        Severity::Error,
    )
}

/// Like [`validate_trigger`] but specifies a maximum [`Severity`] for the reports emitted by this
/// validation. Used to validate triggers in item definitions that don't warrant the `Error` level.
///
/// Returns true iff the trigger had side effects (such as saving scopes).
pub fn validate_trigger_max_sev(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    max_sev: Severity,
) -> bool {
    validate_trigger_internal(
        Lowercase::empty(),
        false,
        block,
        data,
        sc,
        tooltipped,
        false,
        max_sev,
    )
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
/// Returns true iff the trigger had side effects (such as saving scopes).
// TODO: `in_list` could be removed if the code checks directly for the `any_` prefix instead.
#[allow(clippy::too_many_arguments)]
pub fn validate_trigger_internal(
    caller: &Lowercase,
    in_list: bool,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: Tooltipped,
    negated: bool,
    max_sev: Severity,
) -> bool {
    let mut side_effects = false;
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
        warn(ErrorKey::Tooltip).msg(msg).info(info).loc(block).push();
    }

    if caller == "trigger_if" || caller == "trigger_else_if" || caller == "trigger_else" {
        if caller != "trigger_else" {
            vd.req_field_warn("limit");
        }
        vd.field_validated_key_block("limit", |key, block, data| {
            if caller == "trigger_else" {
                let msg = "`trigger_else` with a `limit` does work, but may indicate a mistake";
                let info = "normally you would use `trigger_else_if` instead.";
                tips(ErrorKey::IfElse).msg(msg).info(info).loc(key).push();
            }
            side_effects |= validate_trigger(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("limit", || "`trigger_if`, `trigger_else_if` or `trigger_else`");
    }

    if in_list {
        vd.field_validated_block("filter", |block, data| {
            side_effects |= validate_trigger(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("filter", || "lists");
    }

    let list_type = if in_list { ListType::Any } else { ListType::None };
    validate_iterator_fields(caller, list_type, data, sc, &mut vd, &mut tooltipped, false);

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
        // desc is handled in the loop
        // mark add and factor as known
        vd.multi_field("add");
        vd.multi_field("factor");
        vd.field_validated_block("trigger", |block, data| {
            side_effects |= validate_trigger(block, data, sc, Tooltipped::No);
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
        vd.multi_field_any_cmp("amount");
    } else if !in_list {
        vd.ban_field("amount", || "`calc_true_if`");
    }

    validate_ifelse_sequence(block, "trigger_if", "trigger_else_if", "trigger_else");

    vd.unknown_fields_any_cmp(|key, cmp, bv| {
        if key.is("value") {
            validate_script_value(bv, data, sc);
            side_effects = true;
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
                if let Some((inscopes, outscope)) = scope_iterator(&it_name, data, sc) {
                    if !it_type.is("any") {
                        let msg = format!("cannot use `{it_type}_` list in a trigger");
                        err(ErrorKey::Validation).msg(msg).loc(key).push();
                        return;
                    }
                    sc.expect(inscopes, &Reason::Token(key.clone()));
                    if let Some(b) = bv.expect_block() {
                        precheck_iterator_fields(ListType::Any, it_name.as_str(), b, data, sc);
                        sc.open_scope(outscope, key.clone());
                        side_effects |= validate_trigger_internal(
                            &Lowercase::new(it_name.as_str()),
                            true,
                            b,
                            data,
                            sc,
                            tooltipped,
                            negated,
                            max_sev,
                        );
                        sc.close();
                    }
                    return;
                }
            }
        }

        side_effects |=
            validate_trigger_key_bv(key, cmp, bv, data, sc, tooltipped, negated, max_sev);
    });

    if caller == "modifier" {
        // check add and factor at the end, accounting for any temporary scope saved
        // elsewhere in the block.
        vd.multi_field_validated("add", |bv, data| {
            validate_script_value(bv, data, sc);
            side_effects = true;
        });

        vd.multi_field_validated("factor", |bv, data| {
            validate_script_value(bv, data, sc);
            side_effects = true;
        });
    }

    side_effects
}

/// Validate a trigger given its key and argument. It is like [`validate_trigger_internal`] except
/// that all special cases are assumed to have been handled. This is the interface used for the
/// `switch` effect, where the key and argument are not together in the script.
///
/// Returns true iff the trigger had side effects (such as saving scopes).
#[allow(clippy::too_many_arguments)] // nothing can be cut
pub fn validate_trigger_key_bv(
    key: &Token,
    cmp: Comparator,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    negated: bool,
    max_sev: Severity,
) -> bool {
    let mut side_effects = false;

    // Scripted trigger?
    if let Some(trigger) = data.get_trigger(key) {
        match bv {
            BV::Value(token) => {
                if !(token.is("yes") || token.is("no") || token.is("YES") || token.is("NO")) {
                    warn(ErrorKey::Validation).msg("expected yes or no").loc(token).push();
                }
                if !trigger.macro_parms().is_empty() {
                    fatal(ErrorKey::Macro).msg("expected macro arguments").loc(token).push();
                    return side_effects;
                }
                let negated = if token.is("no") { !negated } else { negated };
                // TODO: check side_effects
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
                            return side_effects;
                        }
                    }
                    vd.unknown_value_fields(|key, _value| {
                        let msg = format!("this scripted trigger does not need parameter {key}");
                        let info = "supplying an unneeded parameter often causes a crash";
                        fatal(ErrorKey::Macro).msg(msg).info(info).loc(key).push();
                    });

                    let args: Vec<_> = parms.into_iter().zip(vec).collect();
                    // TODO: check side_effects
                    trigger.validate_macro_expansion(key, &args, data, sc, tooltipped, negated);
                }
            }
        }
        return side_effects;
    }

    // `10 < script value` is a valid trigger
    if key.is_number() {
        validate_script_value(bv, data, sc);
        return side_effects;
    }

    let part_vec = partition(key);
    sc.open_builder();
    for i in 0..part_vec.len() {
        let mut part_flags = PartFlags::empty();
        if i == 0 {
            part_flags |= PartFlags::First;
        }
        if i + 1 == part_vec.len() {
            part_flags |= PartFlags::Last;
        }
        if matches!(cmp, Comparator::Equals(Question)) {
            part_flags |= PartFlags::Question;
        }
        let part = &part_vec[i];

        match part {
            Part::TokenArgument(func, arg) => validate_argument(part_flags, func, arg, data, sc),
            Part::Token(part) => {
                let part_lc = Lowercase::new(part.as_str());
                // prefixed scope transition, e.g. cp:councillor_steward
                if let Some((prefix, mut arg)) = part.split_once(':') {
                    // event_id have multiple parts separated by `.`
                    let is_event_id = prefix.lowercase_is("event_id");
                    if is_event_id {
                        arg = key.split_once(':').unwrap().1;
                    }
                    // known prefix
                    if let Some(entry) = scope_prefix(&prefix) {
                        validate_argument_scope(part_flags, entry, &prefix, &arg, data, sc);
                        if is_event_id {
                            break; // force last part
                        }
                    } else {
                        let msg = format!("unknown prefix `{prefix}:`");
                        err(ErrorKey::Validation).msg(msg).loc(prefix).push();
                        sc.close();
                        return side_effects;
                    }
                } else if part_lc == "root" {
                    sc.replace_root();
                } else if part_lc == "prev" {
                    if !part_flags.contains(PartFlags::First) && !Game::is_imperator() {
                        warn_not_first(part);
                    }
                    sc.replace_prev();
                } else if part_lc == "this" {
                    sc.replace_this();
                } else if data.script_values.exists(part.as_str()) {
                    // TODO: check side_effects
                    data.script_values.validate_call(part, data, sc);
                    sc.replace(Scopes::Value, part.clone());
                } else if let Some((inscopes, outscope)) = scope_to_scope(part, sc.scopes()) {
                    #[cfg(feature = "imperator")]
                    if let Some((inscopes, trigger)) = scope_trigger(part, data) {
                        // If a trigger of the same name exists, and it's compatible with this
                        // location and scope context, then that trigger takes precedence.
                        if part_flags.contains(PartFlags::Last)
                            && (inscopes.contains(Scopes::None) || sc.scopes().intersects(inscopes))
                        {
                            validate_inscopes(part_flags, part, inscopes, sc);
                            sc.close();
                            side_effects |= match_trigger_bv(
                                &trigger,
                                &part.clone(),
                                cmp,
                                bv,
                                data,
                                sc,
                                tooltipped,
                                negated,
                                max_sev,
                            );
                            return side_effects;
                        }
                    }
                    validate_inscopes(part_flags, part, inscopes, sc);
                    sc.replace(outscope, part.clone());
                } else if let Some((inscopes, trigger)) = scope_trigger(part, data) {
                    if !part_flags.contains(PartFlags::Last) {
                        let msg = format!("`{part}` should be the last part");
                        warn(ErrorKey::Validation).msg(msg).loc(part).push();
                        sc.close();
                        return side_effects;
                    }
                    validate_inscopes(part_flags, part, inscopes, sc);
                    if sc.scopes() == Scopes::None && part_lc == "current_year" {
                        warn(ErrorKey::Bugs)
                            .msg("current_year does not work in empty scope")
                            .info("try using current_date, or dummy_male.current_year")
                            .loc(part)
                            .push();
                    }
                    sc.close();
                    side_effects |= match_trigger_bv(
                        &trigger,
                        &part.clone(),
                        cmp,
                        bv,
                        data,
                        sc,
                        tooltipped,
                        negated,
                        max_sev,
                    );
                    return side_effects;
                } else {
                    // TODO: warn if trying to use iterator here
                    let msg = format!("unknown token `{part}`");
                    err(ErrorKey::UnknownField).msg(msg).loc(part).push();
                    sc.close();
                    return side_effects;
                }
            }
        }
    }

    if !matches!(cmp, Comparator::Equals(Single | Question)) {
        if sc.can_be(Scopes::Value) {
            sc.close();
            // TODO: check side_effects
            validate_script_value(bv, data, sc);
        } else if matches!(cmp, Comparator::NotEquals | Comparator::Equals(Double)) {
            let scopes = sc.scopes();
            sc.close();
            if let Some(token) = bv.expect_value() {
                validate_target_ok_this(token, data, sc, scopes);
            }
        } else {
            let msg = format!("unexpected comparator {cmp}");
            warn(ErrorKey::Validation).msg(msg).loc(key).push();
            sc.close();
        }
        return side_effects;
    }

    match bv {
        BV::Value(t) => {
            let scopes = sc.scopes();
            sc.close();
            validate_target_ok_this(t, data, sc, scopes);
        }
        BV::Block(b) => {
            sc.finalize_builder();
            side_effects |= validate_trigger_internal(
                Lowercase::empty(),
                false,
                b,
                data,
                sc,
                tooltipped,
                negated,
                max_sev,
            );
            sc.close();
        }
    }
    side_effects
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
///
/// Returns true iff the trigger had side effects (such as saving scopes).
fn match_trigger_fields(
    fields: &[(&str, Trigger)],
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
    negated: bool,
    max_sev: Severity,
) -> bool {
    let mut side_effects = false;
    let mut vd = Validator::new(block, data);
    vd.set_max_severity(max_sev);
    for (field, _) in fields {
        if let Some(opt) = field.strip_prefix('?') {
            vd.field_any_cmp(opt);
        } else if let Some(mlt) = field.strip_prefix('*') {
            vd.multi_field_any_cmp(mlt);
        } else if let Some(mlt) = field.strip_prefix('+') {
            vd.req_field(mlt);
            vd.multi_field_any_cmp(mlt);
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
                side_effects |= match_trigger_bv(
                    trigger, key, *cmp, bv, data, sc, tooltipped, negated, max_sev,
                );
            }
        }
    }
    side_effects
}

/// Takes a [`Trigger`] and a trigger field, and validates that the constraints
/// specified by the `Trigger` hold.
///
/// Returns true iff the trigger had side effects (such as saving scopes).
#[allow(clippy::too_many_arguments)]
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
) -> bool {
    let mut side_effects = false;
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
            // TODO: check side_effects
            validate_script_value(bv, data, sc);
        }
        #[cfg(feature = "ck3")]
        Trigger::CompareValueWarnEq => {
            must_be_eq = false;
            warn_if_eq = true;
            // TODO: check side_effects
            validate_script_value(bv, data, sc);
        }
        #[cfg(feature = "ck3")]
        Trigger::SetValue => {
            // TODO: check side_effects
            validate_script_value(bv, data, sc);
        }
        Trigger::CompareDate => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                if Date::from_str(token.as_str()).is_err() {
                    let msg = format!("{name} expects a date value");
                    warn(ErrorKey::Validation).msg(msg).loc(token).push();
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
                // TODO: check side_effects
                validate_script_value(bv, data, sc);
            } else {
                bv.expect_value();
            }
        }
        Trigger::ScopeOkThis(s) => {
            if let Some(token) = bv.get_value() {
                validate_target_ok_this(token, data, sc, *s);
            } else if s.contains(Scopes::Value) {
                // TODO: check side_effects
                validate_script_value(bv, data, sc);
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
                    warn(ErrorKey::Validation).msg(msg).info(info).loc(token).push();
                }
            }
        }
        Trigger::CompareChoice(choices) => {
            must_be_eq = false;
            if let Some(token) = bv.expect_value() {
                if !choices.contains(&token.as_str()) {
                    let msg = format!("{name} expects one of {}", stringify_choices(choices));
                    warn(ErrorKey::Validation).msg(msg).loc(token).push();
                }
            }
        }
        Trigger::Block(fields) => {
            if let Some(block) = bv.expect_block() {
                side_effects |=
                    match_trigger_fields(fields, block, data, sc, tooltipped, negated, max_sev);
            }
        }
        #[cfg(feature = "ck3")]
        Trigger::ScopeOrBlock(s, fields) => match bv {
            BV::Value(token) => validate_target(token, data, sc, *s),
            BV::Block(block) => {
                side_effects |=
                    match_trigger_fields(fields, block, data, sc, tooltipped, negated, max_sev);
            }
        },
        #[cfg(feature = "ck3")]
        Trigger::ItemOrBlock(i, fields) => match bv {
            BV::Value(token) => data.verify_exists_max_sev(*i, token, max_sev),
            BV::Block(block) => {
                side_effects |=
                    match_trigger_fields(fields, block, data, sc, tooltipped, negated, max_sev);
            }
        },
        #[cfg(feature = "ck3")]
        Trigger::CompareValueOrBlock(fields) => match bv {
            BV::Value(t) => {
                validate_target(t, data, sc, Scopes::Value);
                must_be_eq = false;
            }
            BV::Block(b) => {
                side_effects |=
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
                    warn(ErrorKey::Validation).msg(msg).loc(block).push();
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
                let name_lc = name.as_str().to_lowercase();
                if name_lc == "all_false"
                    || name_lc == "not"
                    || name_lc == "nand"
                    || name_lc == "nor"
                {
                    negated = !negated;
                }
                let mut tooltipped = tooltipped;
                if name_lc == "custom_description" {
                    tooltipped = Tooltipped::No;
                }
                side_effects |= validate_trigger_internal(
                    &Lowercase::from_string_unchecked(name_lc),
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
                            warn(ErrorKey::Scopes).msg(msg).loc(token).push();
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
                                tips(ErrorKey::Tooltip).msg(msg).info(info).loc(name).push();
                            }
                        }
                    }
                }
            } else if name.is("custom_tooltip") {
                match bv {
                    BV::Value(t) => data.verify_exists_max_sev(Item::Localization, t, max_sev),
                    BV::Block(b) => {
                        side_effects |= validate_trigger_internal(
                            &Lowercase::new(name.as_str()),
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
                        side_effects = true;
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
                        BV::Block(_) => validate_script_value(bv, data, sc),
                    });
                    // TODO: figure out the scope type of `value` and use that
                    if let Some(name) = vd.field_value("name") {
                        sc.define_name_token(name.as_str(), Scopes::primitive(), name);
                        side_effects = true;
                    }
                }
            } else if name.is("save_temporary_scope_as") {
                if let Some(name) = bv.expect_value() {
                    sc.save_current_scope(name.as_str());
                    side_effects = true;
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
                        side_effects |= validate_trigger(block, data, sc, tooltipped);
                    }
                }
            } else if name.is("switch") {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_max_severity(max_sev);
                    vd.req_field("trigger");
                    if let Some(target) = vd.field_value("trigger") {
                        let target = target.clone();
                        let mut count = 0;
                        vd.unknown_block_fields(|key, block| {
                            count += 1;
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
                            side_effects |= validate_trigger(block, data, sc, tooltipped);
                        });
                        if count == 0 {
                            let msg = "switch with no branches";
                            err(ErrorKey::Logic).msg(msg).loc(name).push();
                        }
                    }
                }
            } else if name.is("add_to_temporary_list") {
                if let Some(value) = bv.expect_value() {
                    sc.define_or_expect_list(value);
                    side_effects = true;
                }
            } else if name.is("is_in_list") {
                if let Some(value) = bv.expect_value() {
                    sc.expect_list(value);
                }
            } else if name.is("is_researching_technology") {
                #[cfg(feature = "vic3")]
                if let Some(value) = bv.expect_value() {
                    if !value.is("any") {
                        data.verify_exists(Item::Technology, value);
                    }
                }
            }
            // TODO: time_of_year
        }
        #[cfg(any(feature = "ck3", feature = "vic3"))]
        Trigger::Removed(msg, info) => {
            err(ErrorKey::Removed).msg(*msg).info(*info).loc(name).push();
        }
        Trigger::UncheckedValue => {
            bv.expect_value();
            side_effects = true; // have to assume it's possible
        }
    }

    if matches!(cmp, Comparator::Equals(_)) {
        if warn_if_eq {
            let msg = format!("`{name} {cmp}` means exactly equal to that amount, which is usually not what you want");
            warn(ErrorKey::Logic).msg(msg).loc(name).push();
        }
    } else if must_be_eq {
        let msg = format!("unexpected comparator {cmp}");
        warn(ErrorKey::Validation).msg(msg).loc(name).push();
    }
    side_effects
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
            warn(ErrorKey::Scopes).msg(msg).loc(token).push();
        }
        return;
    }
    let part_vec = partition(token);
    sc.open_builder();
    for i in 0..part_vec.len() {
        let mut part_flags = PartFlags::empty();
        if i == 0 {
            part_flags |= PartFlags::First;
        }
        if i + 1 == part_vec.len() {
            part_flags |= PartFlags::Last;
        }
        let part = &part_vec[i];

        match part {
            Part::TokenArgument(func, arg) => validate_argument(part_flags, func, arg, data, sc),
            Part::Token(part) => {
                let part_lc = Lowercase::new(part.as_str());
                // prefixed scope transition, e.g. cp:councillor_steward
                if let Some((prefix, mut arg)) = part.split_once(':') {
                    // event_id have multiple parts separated by `.`
                    let is_event_id = prefix.lowercase_is("event_id");
                    if is_event_id {
                        arg = token.split_once(':').unwrap().1;
                    }
                    // known prefix
                    if let Some(entry) = scope_prefix(&prefix) {
                        validate_argument_scope(part_flags, entry, &prefix, &arg, data, sc);
                        if is_event_id {
                            break; // force last part
                        }
                    } else {
                        let msg = format!("unknown prefix `{prefix}:`");
                        err(ErrorKey::Validation).msg(msg).loc(prefix).push();
                        sc.close();
                        return;
                    }
                } else if part_lc == "root" {
                    sc.replace_root();
                } else if part_lc == "prev" {
                    if !part_flags.contains(PartFlags::First) && !Game::is_imperator() {
                        warn_not_first(part);
                    }
                    sc.replace_prev();
                } else if part_lc == "this" {
                    sc.replace_this();
                } else if data.script_values.exists(part.as_str()) {
                    // TODO: check side_effects
                    data.script_values.validate_call(part, data, sc);
                    sc.replace(Scopes::Value, part.clone());
                } else if let Some((inscopes, outscope)) = scope_to_scope(part, sc.scopes()) {
                    #[cfg(feature = "imperator")]
                    if let Some(inscopes) = trigger_comparevalue(part, data) {
                        // If a trigger of the same name exists, and it's compatible with this
                        // location and scope context, then that trigger takes precedence.
                        if part_flags.contains(PartFlags::Last)
                            && (inscopes.contains(Scopes::None) || sc.scopes().intersects(inscopes))
                        {
                            validate_inscopes(part_flags, part, inscopes, sc);
                            sc.replace(Scopes::Value, part.clone());
                            continue;
                        }
                    }
                    validate_inscopes(part_flags, part, inscopes, sc);
                    sc.replace(outscope, part.clone());
                } else if let Some(inscopes) = trigger_comparevalue(part, data) {
                    if !part_flags.contains(PartFlags::Last) {
                        let msg = format!("`{part}` should be the last part");
                        warn(ErrorKey::Validation).msg(msg).loc(part).push();
                        sc.close();
                        return;
                    }
                    validate_inscopes(part_flags, part, inscopes, sc);
                    if sc.scopes() == Scopes::None && part_lc == "current_year" {
                        warn(ErrorKey::Bugs)
                            .msg("current_year does not work in empty scope")
                            .info("try using current_date, or dummy_male.current_year")
                            .loc(part)
                            .push();
                    }
                    sc.replace(Scopes::Value, part.clone());
                } else {
                    // See if the user forgot a prefix like `faith:` or `culture:`
                    let mut opt_info = None;
                    if part_flags.contains(PartFlags::First | PartFlags::Last) {
                        if let Some(prefix) = needs_prefix(part.as_str(), data, outscopes) {
                            opt_info = Some(format!("did you mean `{prefix}:{part}` ?"));
                        }
                    }

                    // TODO: warn if trying to use iterator here
                    let msg = format!("unknown token `{part}`");
                    err(ErrorKey::UnknownField).msg(msg).opt_info(opt_info).loc(part).push();
                    sc.close();
                    return;
                }
            }
        }
    }
    let (final_scopes, because) = sc.scopes_reason();
    if !outscopes.intersects(final_scopes | Scopes::None) {
        let part = &part_vec[part_vec.len() - 1];
        let msg = format!("`{part}` produces {final_scopes} but expected {outscopes}");
        // Must not be at the same location to avoid spurious error messages
        let opt_loc = (part.loc() != because.token().loc).then(|| because.token());
        let msg2 = format!("scope was {}", because.msg());
        warn(ErrorKey::Scopes).msg(msg).loc(part).opt_loc_msg(opt_loc, msg2).push();
    }
    sc.close();
}

/// Just like [`validate_target_ok_this`], but warns if the target is a literal `this` because that
/// is usually a mistake.
pub fn validate_target(token: &Token, data: &Everything, sc: &mut ScopeContext, outscopes: Scopes) {
    validate_target_ok_this(token, data, sc, outscopes);
    if token.is("this") {
        let msg = "target `this` makes no sense here";
        warn(ErrorKey::UseOfThis).msg(msg).loc(token).push();
    }
}

/// A part in a token chain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// A simple token
    Token(Token),
    /// Function and argument tokens
    TokenArgument(Token, Token),
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Token(token) => token.fmt(f),
            Part::TokenArgument(func, arg) => write!(f, "{func}({arg})"),
        }
    }
}

impl Part {
    fn loc(&self) -> Loc {
        match self {
            Part::Token(t) | Part::TokenArgument(t, _) => t.loc,
        }
    }
}

/// This function partitions the input token into parts separated by `.`. Each part may contain either a token
/// or a token-argument pair when using the `()`-syntax, e.g. `"prowess_diff(liege)"`. It does not validate the tokens
/// or arguments, but simply parses and detects any syntactical errors.
pub fn partition(token: &Token) -> Vec<Part> {
    let mut parts = Vec::new();

    let mut has_part_argument = false;
    let mut has_part_argument_erred = false;
    let mut paren_depth = 0;
    let (mut part_idx, mut part_col) = (0, 0);
    let (mut first_paren_idx, mut first_paren_col) = (0, 0);
    let (mut second_paren_idx, mut second_paren_col) = (0, 0);

    for (col, (idx, ch)) in token.as_str().char_indices().enumerate() {
        let col = u16::try_from(col).expect("internal error: 2^16 columns");
        match ch {
            '.' => {
                if paren_depth == 0 {
                    if part_idx == idx {
                        // Empty part; err but skip it since it's likely a typo
                        let mut loc = token.loc;
                        loc.column += col;
                        err(ErrorKey::Validation).msg("empty part").loc(loc).push();
                    } else if !has_part_argument {
                        // The just completed part has no argument
                        let mut part_loc = token.loc;
                        part_loc.column += part_col;
                        #[allow(unused_mut)]
                        let mut part_token = token.subtoken(part_idx..idx, part_loc);
                        #[cfg(feature = "imperator")]
                        // Imperator has a `hidden:` prefix that can go before other prefixes so it
                        // has to be handled specially.
                        if let Some(hidden_arg) = part_token.strip_prefix("hidden:") {
                            part_token = hidden_arg;
                        }
                        parts.push(Part::Token(part_token));
                    }
                    has_part_argument = false;
                    has_part_argument_erred = false;
                    part_col = col + 1;
                    part_idx = idx + 1;
                }
            }
            '(' => {
                if paren_depth == 0 {
                    first_paren_col = col;
                    first_paren_idx = idx;
                } else if paren_depth == 1 {
                    second_paren_col = col;
                    second_paren_idx = idx;
                }

                paren_depth += 1;
            }
            ')' => {
                if paren_depth == 0 {
                    // Missing opening parenthesis `(`
                    let mut loc = token.loc;
                    loc.column += col;
                    err(ErrorKey::Validation)
                        .msg("closing without opening parenthesis `(`")
                        .loc(loc)
                        .push();
                } else if paren_depth == 1 {
                    // Argument between parentheses
                    let mut func_loc = token.loc;
                    func_loc.column += part_col;
                    let func_token = token.subtoken(part_idx..first_paren_idx, func_loc);

                    let mut arg_loc = token.loc;
                    arg_loc.column += first_paren_col + 1;
                    let arg_token = token.subtoken_stripped(first_paren_idx + 1..idx, arg_loc);

                    parts.push(Part::TokenArgument(func_token, arg_token));
                    has_part_argument = true;
                    paren_depth -= 1;
                } else if paren_depth == 2 {
                    // Cannot have nested parentheses
                    let mut loc = token.loc;
                    loc.column += second_paren_col;
                    let nested_paren_token = token.subtoken(second_paren_idx..=idx, loc);
                    err(ErrorKey::Validation)
                        .msg("cannot have nested parentheses")
                        .loc(nested_paren_token)
                        .push();
                    paren_depth -= 1;
                }
            }
            _ => {
                // an argument can only be the last part or followed by dot `.` AND hasn't erred from it yet
                if has_part_argument && !has_part_argument_erred {
                    let mut loc = token.loc;
                    loc.column += col;
                    err(ErrorKey::Validation)
                        .msg("argument can only be the last part or followed by dot `.`")
                        .loc(loc)
                        .push();
                    has_part_argument_erred = true;
                }
            }
        }
    }

    if paren_depth > 0 {
        // Missing closing parenthesis `)`
        let mut loc = token.loc;
        loc.column += first_paren_col;
        let broken_token = token.subtoken(first_paren_idx.., loc);
        err(ErrorKey::Validation)
            .msg("opening without closing parenthesis `)`")
            .loc(broken_token)
            .push();
    }

    if part_idx == token.as_str().len() {
        // Trailing `.`
        let mut loc = token.loc;
        loc.column += part_col;
        err(ErrorKey::Validation).msg("trailing dot `.`").loc(loc).push();
    } else if !has_part_argument {
        // final part (without argument)
        let mut part_loc = token.loc;
        part_loc.column += part_col;
        // SAFETY: part_idx < token.as_str.len()
        #[allow(unused_mut)]
        let mut part_token = token.subtoken(part_idx.., part_loc);
        #[cfg(feature = "imperator")]
        // see above
        if let Some(hidden_arg) = part_token.strip_prefix("hidden:") {
            part_token = hidden_arg;
        }
        parts.push(Part::Token(part_token));
    }
    parts
}

bitflags! {
    // Attributes can be applied to flags types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PartFlags: u8 {
        const First = 0b_0000_0001;
        const Last = 0b_0000_0010;
        const Question = 0b_0000_0100;
    }
}

#[inline]
pub fn warn_not_first(name: &Token) {
    let msg = format!("`{name}:` makes no sense except as first part");
    warn(ErrorKey::Validation).msg(msg).loc(name).push();
}

pub fn validate_inscopes(
    part_flags: PartFlags,
    name: &Token,
    inscopes: Scopes,
    sc: &mut ScopeContext,
) {
    // If the part does not use its inscope then any parts that come before it are useless
    // and probably indicate a mistake is being made.
    if inscopes == Scopes::None && !part_flags.contains(PartFlags::First) {
        warn_not_first(name);
    }
    sc.expect(inscopes, &Reason::Token(name.clone()));
}

fn validate_argument_internal(
    arg: &Token,
    validation: ArgumentValue,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    match validation {
        ArgumentValue::Item(item) => data.verify_exists(item, arg),
        ArgumentValue::Scope(scope) => validate_target(arg, data, sc, scope),
        #[cfg(feature = "ck3")]
        ArgumentValue::ScopeOrItem(scope, item) => {
            if !data.item_exists(item, arg.as_str()) {
                validate_target(arg, data, sc, scope);
            }
        }
        #[cfg(feature = "vic3")]
        ArgumentValue::Modif => {
            // TODO: deduce the ModifKinds from the `this` scope
            verify_modif_exists(arg, data, ModifKinds::all(), Severity::Warning);
        }
        ArgumentValue::UncheckedValue => (),
    }
}

/// Validate for scope and not trigger arguments
pub fn validate_argument_scope(
    part_flags: PartFlags,
    (inscopes, outscopes, validation): (Scopes, Scopes, ArgumentValue),
    func: &Token,
    arg: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    validate_inscopes(part_flags, func, inscopes, sc);
    validate_argument_internal(arg, validation, data, sc);

    let mut outscopes_token = func.clone();
    outscopes_token.combine(arg, ':');
    if func.lowercase_is("scope") {
        if part_flags.contains(PartFlags::Last | PartFlags::Question) {
            sc.exists_scope(arg.as_str(), outscopes_token.clone());
        }
        sc.replace_named_scope(arg.as_str(), outscopes_token);
    } else {
        sc.replace(outscopes, outscopes_token);
    }
}

/// Validate that the argument passed through is valid, either being of a complex trigger compare value,
/// or a scope prefix.
#[allow(unreachable_code, unused_variables)]
pub fn validate_argument(
    part_flags: PartFlags,
    func: &Token,
    arg: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    #[cfg(feature = "imperator")]
    if Game::is_imperator() {
        // Imperator does not use `()`
        let msg = "imperator does not support the `()` syntax";
        let mut opening_paren_loc = arg.loc;
        opening_paren_loc.column -= 1;
        err(ErrorKey::Validation).msg(msg).loc(opening_paren_loc).push();
        return;
    }

    let scope_trigger_complex: fn(&str) -> Option<(Scopes, ArgumentValue)> = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::triggers::scope_trigger_complex,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::triggers::scope_trigger_complex,
        #[cfg(feature = "imperator")]
        Game::Imperator => unreachable!(),
    };

    let func_lc = func.as_str().to_lowercase();
    if let Some((inscopes, validation)) = scope_trigger_complex(&func_lc) {
        sc.expect(inscopes, &Reason::Token(func.clone()));
        validate_argument_internal(arg, validation, data, sc);
        sc.replace(Scopes::Value, func.clone());
    } else if let Some(entry) = scope_prefix(func) {
        validate_argument_scope(part_flags, entry, func, arg, data, sc);
    } else {
        let msg = format!("unknown token `{func}`");
        err(ErrorKey::Validation).msg(msg).loc(func).push();
    }
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
    /// value is from a list given here that can be compared
    CompareChoice(&'static [&'static str]),
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

    #[cfg(any(feature = "ck3", feature = "vic3"))]
    Removed(&'static str, &'static str),

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
        #[cfg(feature = "imperator")]
        Some((s, Trigger::CompareValue | Trigger::CompareDate)) => Some(s),
        _ => std::option::Option::None,
    }
}
