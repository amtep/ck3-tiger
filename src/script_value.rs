//! Validation of script values, which are values that update dynamically based on rules given in
//! their math blocks.
//!
//! Scriptvalues can also be non-dynamic, in the sense of being just a literal or the name of
//! another script value.

use crate::block::{Block, BlockItem, Comparator, Eq::*, BV};
use crate::context::{Reason, ScopeContext};
use crate::everything::Everything;
use crate::helpers::TriBool;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, tips, untidy, warn, ErrorKey};
use crate::scopes::{scope_iterator, Scopes};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target_ok_this, validate_trigger};
use crate::validate::{
    precheck_iterator_fields, validate_ifelse_sequence, validate_inside_iterator,
    validate_iterator_fields, validate_scope_chain, ListType,
};
use crate::validator::Validator;

/// Validate a block that's part of a script value.
/// * `have_value`: indicates whether this script value has had some sort of value set already.
/// It's used to emit warnings when overwriting this value, or when assuming there is a value
/// when there isn't one. Has values `True`, `False`, or `Maybe`.
/// * `check_desc`: indicates whether localization errors are worth warning about. Some script values
/// only have their breakdowns displayed while debugging, and those can have raw (non-localized)
/// text in their descs.
/// Returns true iff this block did something useful. (Used for warnings)
fn validate_inner(
    mut vd: Validator,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut have_value: TriBool,
    check_desc: bool,
) -> bool {
    if check_desc {
        vd.field_item("desc", Item::Localization);
        vd.field_item("format", Item::Localization);
    } else {
        vd.field_value("desc");
        vd.field_value("format");
    }

    let mut made_changes = false;

    validate_ifelse_sequence(block, "if", "else_if", "else");
    vd.set_allow_questionmark_equals(true);
    vd.unknown_fields_cmp(|token, cmp, bv| {
        if token.is("save_temporary_scope_as") {
            // save_temporary_scope_as is now allowed in script values
            if let Some(name) = bv.expect_value() {
                sc.save_current_scope(name.as_str());
                made_changes = true;
            }
        } else if token.is("save_temporary_value_as") {
            // seen in vanilla
            if let Some(name) = bv.expect_value() {
                sc.define_name_token(name.as_str(), Scopes::Value, name);
                made_changes = true;
            }
        } else if token.is("value") {
            if have_value == TriBool::True {
                let msg = "setting value here will overwrite the previous calculations";
                warn(ErrorKey::Logic).msg(msg).loc(token).push();
            }
            have_value = TriBool::True;
            validate_bv(bv, data, sc, check_desc);
            made_changes = true;
        } else if token.is("add") || token.is("subtract") || token.is("min") || token.is("max") {
            have_value = TriBool::True;
            validate_bv(bv, data, sc, check_desc);
            made_changes = true;
        } else if token.is("multiply") || token.is("divide") || token.is("modulo") {
            if have_value == TriBool::False {
                let msg = format!("nothing to {token} yet");
                warn(ErrorKey::Logic).msg(msg).loc(token).push();
            }
            validate_bv(bv, data, sc, check_desc);
            made_changes = true;
        } else if token.is("round") || token.is("ceiling") || token.is("floor") || token.is("abs") {
            if have_value == TriBool::False {
                let msg = format!("nothing to {token} yet");
                warn(ErrorKey::Logic).msg(msg).loc(token).push();
            }
            if let Some(value) = bv.expect_value() {
                if !value.is("yes") && !value.is("no") {
                    let msg = "expected yes or no";
                    warn(ErrorKey::Validation).msg(msg).loc(value).push();
                }
                made_changes = true;
            }
        } else if token.is("fixed_range") || token.is("integer_range") {
            if have_value == TriBool::True {
                let msg = "using fixed_range here will overwrite the previous calculations";
                warn(ErrorKey::Logic).msg(msg).loc(token).push();
            }
            if let Some(block) = bv.expect_block() {
                validate_minmax_range(block, data, sc, check_desc);
                made_changes = true;
            }
            have_value = TriBool::True;
        } else if token.is("if") || token.is("else_if") {
            if let Some(block) = bv.expect_block() {
                validate_if(token, block, data, sc, check_desc);
                made_changes = true;
            }
            have_value = TriBool::Maybe;
        } else if token.is("else") {
            if let Some(block) = bv.expect_block() {
                validate_else(token, block, data, sc, check_desc);
                made_changes = true;
            }
            have_value = TriBool::Maybe;
        } else {
            if let Some((it_type, it_name)) = token.split_once('_') {
                if it_type.is("every")
                    || it_type.is("ordered")
                    || it_type.is("random")
                    || it_type.is("any")
                {
                    if let Some((inscopes, outscope)) = scope_iterator(&it_name, data, sc) {
                        if it_type.is("any") {
                            let msg = "cannot use `any_` iterators in a script value";
                            err(ErrorKey::Validation).msg(msg).loc(token).push();
                        }
                        sc.expect(inscopes, &Reason::Token(token.clone()));
                        if let Some(block) = bv.expect_block() {
                            let ltype = ListType::try_from(it_type.as_str()).unwrap();
                            precheck_iterator_fields(ltype, it_name.as_str(), block, data, sc);
                            sc.open_scope(outscope, token.clone());
                            validate_iterator(ltype, &it_name, block, data, sc, check_desc);
                            made_changes = true;
                            sc.close();
                            have_value = TriBool::Maybe;
                        }
                    }
                    return;
                }
            }

            // Check for target = { script_value }
            sc.open_builder();
            if validate_scope_chain(token, data, sc, matches!(cmp, Comparator::Equals(Question))) {
                if let Some(block) = bv.expect_block() {
                    sc.finalize_builder();
                    let vd = Validator::new(block, data);
                    made_changes |= validate_inner(vd, block, data, sc, have_value, check_desc);
                    have_value = TriBool::Maybe;
                }
            }
            sc.close();
        }
    });
    made_changes
}

/// Validate a block inside an iterator that's part of a script value.
/// Checks some fields and then relies on `validate_inner` for the heavy lifting.
fn validate_iterator(
    ltype: ListType,
    it_name: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    check_desc: bool,
) {
    let mut side_effects = false;
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("limit", |block, data| {
        side_effects |= validate_trigger(block, data, sc, Tooltipped::No);
    });

    let mut tooltipped = Tooltipped::No;
    validate_iterator_fields(Lowercase::empty(), ltype, data, sc, &mut vd, &mut tooltipped, true);

    validate_inside_iterator(
        &Lowercase::new(it_name.as_str()),
        ltype,
        block,
        data,
        sc,
        &mut vd,
        Tooltipped::No,
    );

    side_effects |= validate_inner(vd, block, data, sc, TriBool::Maybe, check_desc);
    if !side_effects {
        let msg = "this iterator does not change the script value";
        let info = "it should be either removed, or changed to do something useful";
        err(ErrorKey::Logic).msg(msg).info(info).loc(block).push();
    }
}

/// Validate one of the `fixed_range` or `integer_range` script value operators.
/// These are rarely used.
fn validate_minmax_range(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    check_desc: bool,
) {
    let mut vd = Validator::new(block, data);
    vd.req_field("min");
    vd.req_field("max");
    vd.multi_field_validated("min", |bv, data| {
        validate_bv(bv, data, sc, check_desc);
    });
    vd.multi_field_validated("max", |bv, data| {
        validate_bv(bv, data, sc, check_desc);
    });
}

/// Validate `if` or `else_if` blocks that are part of a script value.
/// Checks the `limit` field and then relies on `validate_inner` for the heavy lifting.
fn validate_if(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    check_desc: bool,
) {
    let mut side_effects = false;
    let mut vd = Validator::new(block, data);
    vd.req_field_warn("limit");
    vd.field_validated_block("limit", |block, data| {
        side_effects |= validate_trigger(block, data, sc, Tooltipped::No);
    });
    side_effects |= validate_inner(vd, block, data, sc, TriBool::Maybe, check_desc);

    if !side_effects {
        let msg = format!("this `{key}` does not change the script value");
        // weak because in an if-if_else sequence you might want some that deliberately do nothing
        // TODO: make this smarter so that it does not warn if followed by an else or else_if
        err(ErrorKey::Logic).weak().msg(msg).loc(key).push();
    }
}

/// Validate `else` blocks that are part of a script value.
/// Just like `validate_if`, but warns if it encounters a `limit` field.
fn validate_else(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    check_desc: bool,
) {
    let mut side_effects = false;
    let mut vd = Validator::new(block, data);
    vd.field_validated_key_block("limit", |key, block, data| {
        let msg = "`else` with a `limit` does work, but may indicate a mistake";
        let info = "normally you would use `else_if` instead.";
        tips(ErrorKey::IfElse).msg(msg).info(info).loc(key).push();
        side_effects |= validate_trigger(block, data, sc, Tooltipped::No);
    });
    side_effects |= validate_inner(vd, block, data, sc, TriBool::Maybe, check_desc);

    if !side_effects {
        let msg = format!("this `{key}` does not change the script value");
        let info = "it should be either removed, or changed to do something useful";
        // only untidy because an empty else is probably not a logic error
        untidy(ErrorKey::Logic).msg(msg).info(info).loc(key).push();
    }
}

/// Validate a script value. It can be a block or a value.
/// As a value, it may be an integer or boolean literal, or a target scope sequence, or a named script value.
/// As a block, it may be a { min max } range, or a calculation block which is validated with `validate_inner`.
/// (Boolean script values are rare but valid. They can't have calculation blocks.)
fn validate_bv(bv: &BV, data: &Everything, sc: &mut ScopeContext, check_desc: bool) {
    // Using validate_target_ok_this here because when chaining script values to each other, you often do `value = this`
    match bv {
        BV::Value(t) => validate_target_ok_this(t, data, sc, Scopes::Value | Scopes::Bool),
        BV::Block(b) => {
            let mut vd = Validator::new(b, data);
            if matches!(b.iter_items().next(), Some(BlockItem::Block(_) | BlockItem::Value(_))) {
                // It's a range like { 1 5 }
                let vec = vd.values();
                if vec.len() == 2 {
                    for v in vec {
                        validate_target_ok_this(v, data, sc, Scopes::Value | Scopes::Bool);
                    }
                } else {
                    warn(ErrorKey::Validation).msg("invalid script value range").loc(b).push();
                }
            } else {
                validate_inner(vd, b, data, sc, TriBool::False, check_desc);
            }
        }
    }
}

pub fn validate_script_value(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    validate_bv(bv, data, sc, true);
}

#[cfg(feature = "ck3")] // only used in ck3; silence dead code warning
pub fn validate_script_value_no_breakdown(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    validate_bv(bv, data, sc, false);
}

/// Validate a script value that's not allowed to do calculations. It must be a literal or the name of another script value
/// that's also not allowed to do calculations.
pub fn validate_non_dynamic_script_value(bv: &BV, data: &Everything) {
    if let Some(token) = bv.get_value() {
        if token.is_number() || token.is("yes") || token.is("no") {
            return;
        }
        if data.script_values.exists(token.as_str()) {
            data.script_values.validate_non_dynamic_call(token, data);
            return;
        }
    }
    let msg = "dynamic script values are not allowed here";
    let info = "only literal numbers or the name of a simple script value";
    err(ErrorKey::Validation).msg(msg).info(info).loc(bv).push();
}
