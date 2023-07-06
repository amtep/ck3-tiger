use crate::block::validator::Validator;
use crate::block::{Block, Comparator, Eq::*, BV};
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::TriBool;
use crate::item::Item;
use crate::report::{advice_info, error, error_info, old_warn, ErrorKey};
use crate::scopes::{scope_iterator, Scopes};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_normal_trigger, validate_target_ok_this};
use crate::validate::{
    precheck_iterator_fields, validate_ifelse_sequence, validate_inside_iterator,
    validate_iterator_fields, validate_scope_chain, ListType,
};

fn validate_inner(
    mut vd: Validator,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut have_value: TriBool,
    check_desc: bool,
) {
    if check_desc {
        vd.field_item("desc", Item::Localization);
        vd.field_item("format", Item::Localization);
    } else {
        vd.field_value("desc");
        vd.field_value("format");
    }

    validate_ifelse_sequence(block, "if", "else_if", "else");
    vd.allow_qeq(true);
    for (token, cmp, bv) in vd.unknown_fields_cmp() {
        // save_temporary_scope_as is now allowed in script values
        if token.is("save_temporary_scope_as") {
            if let Some(name) = bv.expect_value() {
                sc.save_current_scope(name.as_str());
            }
        } else if token.is("save_temporary_value_as") {
            // seen in vanilla
            if let Some(name) = bv.expect_value() {
                sc.define_name(name.as_str(), Scopes::Value, name);
            }
        } else if token.is("value") {
            if have_value == TriBool::True {
                let msg = "setting value here will overwrite the previous calculations";
                old_warn(token, ErrorKey::Logic, msg);
            }
            have_value = TriBool::True;
            validate_bv(bv, data, sc, check_desc);
        } else if token.is("add") || token.is("subtract") || token.is("min") || token.is("max") {
            have_value = TriBool::True;
            validate_bv(bv, data, sc, check_desc);
        } else if token.is("multiply") || token.is("divide") || token.is("modulo") {
            if have_value == TriBool::False {
                let msg = format!("nothing to {token} yet");
                old_warn(token, ErrorKey::Logic, &msg);
            }
            validate_bv(bv, data, sc, check_desc);
        } else if token.is("round") || token.is("ceiling") || token.is("floor") {
            if have_value == TriBool::False {
                let msg = format!("nothing to {token} yet");
                old_warn(token, ErrorKey::Logic, &msg);
            }
            if let Some(value) = bv.expect_value() {
                if !value.is("yes") && !value.is("no") {
                    let msg = "expected yes or no";
                    old_warn(value, ErrorKey::Validation, msg);
                }
            }
        } else if token.is("fixed_range") || token.is("integer_range") {
            if have_value == TriBool::True {
                let msg = "using fixed_range here will overwrite the previous calculations";
                old_warn(token, ErrorKey::Logic, msg);
            }
            if let Some(block) = bv.expect_block() {
                validate_minmax_range(block, data, sc, check_desc);
            }
            have_value = TriBool::True;
        } else if token.is("if") || token.is("else_if") {
            if let Some(block) = bv.expect_block() {
                validate_if(block, data, sc, check_desc);
            }
            have_value = TriBool::Maybe;
        } else if token.is("else") {
            if let Some(block) = bv.expect_block() {
                validate_else(block, data, sc, check_desc);
            }
            have_value = TriBool::Maybe;
        } else {
            if let Some((it_type, it_name)) = token.split_once('_') {
                if it_type.is("every")
                    || it_type.is("ordered")
                    || it_type.is("random")
                    || it_type.is("any")
                {
                    if let Some((inscopes, outscope)) = scope_iterator(&it_name, data) {
                        if it_type.is("any") {
                            let msg = "cannot use `any_` iterators in a script value";
                            error(token, ErrorKey::Validation, msg);
                        }
                        sc.expect(inscopes, token);
                        if let Some(block) = bv.expect_block() {
                            let ltype = ListType::try_from(it_type.as_str()).unwrap();
                            precheck_iterator_fields(ltype, block, data, sc);
                            sc.open_scope(outscope, token.clone());
                            validate_iterator(ltype, &it_name, block, data, sc, check_desc);
                            sc.close();
                            have_value = TriBool::Maybe;
                        }
                    }
                    continue;
                }
            }

            // Check for target = { script_value }
            sc.open_builder();
            if validate_scope_chain(token, data, sc, matches!(cmp, Comparator::Equals(Question))) {
                if let Some(block) = bv.expect_block() {
                    sc.finalize_builder();
                    let vd = Validator::new(block, data);
                    validate_inner(vd, block, data, sc, have_value, check_desc);
                    have_value = TriBool::Maybe;
                }
            }
            sc.close();
        }
    }
}

fn validate_iterator(
    ltype: ListType,
    it_name: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    check_desc: bool,
) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("limit", |block, data| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });

    let mut tooltipped = Tooltipped::No;
    validate_iterator_fields("", ltype, data, sc, &mut vd, &mut tooltipped);

    validate_inside_iterator(
        it_name.as_str(),
        ltype,
        block,
        data,
        sc,
        &mut vd,
        Tooltipped::No,
    );

    validate_inner(vd, block, data, sc, TriBool::Maybe, check_desc);
}

fn validate_minmax_range(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    check_desc: bool,
) {
    let mut vd = Validator::new(block, data);
    vd.req_field("min");
    vd.req_field("max");
    vd.field_validated_bvs("min", |bv, data| {
        validate_bv(bv, data, sc, check_desc);
    });
    vd.field_validated_bvs("max", |bv, data| {
        validate_bv(bv, data, sc, check_desc);
    });
}

fn validate_if(block: &Block, data: &Everything, sc: &mut ScopeContext, check_desc: bool) {
    let mut vd = Validator::new(block, data);
    vd.req_field_warn("limit");
    vd.field_validated_block("limit", |block, data| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
    validate_inner(vd, block, data, sc, TriBool::Maybe, check_desc);
}

fn validate_else(block: &Block, data: &Everything, sc: &mut ScopeContext, check_desc: bool) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_key_block("limit", |key, block, data| {
        let msg = "`else` with a `limit` does work, but may indicate a mistake";
        let info = "normally you would use `else_if` instead.";
        advice_info(key, ErrorKey::IfElse, msg, info);
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
    validate_inner(vd, block, data, sc, TriBool::Maybe, check_desc);
}

fn validate_bv(bv: &BV, data: &Everything, sc: &mut ScopeContext, check_desc: bool) {
    // Using validate_target_ok_this here because when chaining script values to each other, you often do `value = this`
    match bv {
        BV::Value(t) => validate_target_ok_this(t, data, sc, Scopes::Value | Scopes::Bool),
        BV::Block(b) => {
            let mut vd = Validator::new(b, data);
            if let Some((None, _, _)) = b.iter_items().next() {
                // It's a range like { 1 5 }
                let vec = vd.values();
                if vec.len() == 2 {
                    for v in vec {
                        validate_target_ok_this(v, data, sc, Scopes::Value | Scopes::Bool);
                    }
                } else {
                    old_warn(b, ErrorKey::Validation, "invalid script value range");
                }
            } else {
                validate_inner(vd, b, data, sc, TriBool::False, check_desc);
            }
        }
    }
}

pub fn validate_scriptvalue(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    validate_bv(bv, data, sc, true);
}

pub fn validate_scriptvalue_no_breakdown(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    validate_bv(bv, data, sc, false);
}

pub fn validate_non_dynamic_scriptvalue(bv: &BV, data: &Everything) {
    if let Some(token) = bv.get_value() {
        if token.is_number() || token.is("yes") || token.is("no") {
            return;
        }
        if data.scriptvalues.exists(token.as_str()) {
            data.scriptvalues.validate_non_dynamic_call(token, data);
            return;
        }
    }
    let msg = "dynamic script values are not allowed here";
    let info = "only literal numbers or the name of a simple script value";
    error_info(bv, ErrorKey::Validation, msg, info);
}
