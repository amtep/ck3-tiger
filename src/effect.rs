use crate::block::validator::Validator;
use crate::block::Block;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::scopes::{scope_iterator, Scopes};
use crate::tables::effects::{scope_effect, Effect};
use crate::trigger::{validate_normal_trigger, validate_target};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ListType {
    None,
    Any,
    Every,
    Ordered,
    Random,
}

pub fn validate_normal_effect<'a>(
    block: &Block,
    data: &'a Everything,
    scopes: Scopes,
    tooltipped: bool,
) -> Scopes {
    let vd = Validator::new(block, data);
    validate_effect("", ListType::None, block, data, scopes, vd, tooltipped)
}

pub fn validate_effect<'a>(
    caller: &str,
    list_type: ListType,
    block: &Block,
    data: &'a Everything,
    mut scopes: Scopes,
    mut vd: Validator<'a>,
    tooltipped: bool,
) -> Scopes {
    if let Some(b) = vd.field_block("limit") {
        if caller == "if" || caller == "else_if" || caller == "else" || list_type != ListType::None
        {
            scopes = validate_normal_trigger(b, data, scopes, tooltipped);
        } else {
            warn(
                block.get_key("limit").unwrap(),
                ErrorKey::Validation,
                "`limit` can only be used in if/else_if or lists",
            );
        }
    }

    if let Some(bv) = vd.field("order_by") {
        if list_type == ListType::Ordered {
            scopes = ScriptValue::validate_bv(bv, data, scopes);
        } else {
            warn(
                block.get_key("order_by").unwrap(),
                ErrorKey::Validation,
                "`order_by` can only be used in `ordered_` lists",
            );
        }
    }

    if let Some(token) = vd.field_value("position") {
        if list_type == ListType::Ordered {
            if token.as_str().parse::<i32>().is_err() {
                warn(token, ErrorKey::Validation, "expected an integer");
            }
        } else {
            warn(
                block.get_key("position").unwrap(),
                ErrorKey::Validation,
                "`position` can only be used in `ordered_` lists",
            );
        }
    }

    if let Some(token) = vd.field_value("min") {
        if list_type == ListType::Ordered {
            if token.as_str().parse::<i32>().is_err() {
                warn(token, ErrorKey::Validation, "expected an integer");
            }
        } else {
            warn(
                block.get_key("min").unwrap(),
                ErrorKey::Validation,
                "`min` can only be used in `ordered_` lists",
            );
        }
    }

    if let Some(bv) = vd.field("max") {
        if list_type == ListType::Ordered {
            scopes = ScriptValue::validate_bv(bv, data, scopes);
        } else {
            warn(
                block.get_key("max").unwrap(),
                ErrorKey::Validation,
                "`max` can only be used in `ordered_` lists",
            );
        }
    }

    if let Some(token) = vd.field_value("check_range_bounds") {
        if list_type == ListType::Ordered {
            if !(token.is("yes") || token.is("no")) {
                warn(token, ErrorKey::Validation, "expected yes or no");
            }
        } else {
            warn(
                block.get_key("check_range_bounds").unwrap(),
                ErrorKey::Validation,
                "`check_range_bounds` can only be used in `ordered_` lists",
            );
        }
    }

    if let Some(_b) = vd.field_block("weight") {
        if list_type == ListType::Random {
            // TODO
        } else {
            warn(
                block.get_key("weight").unwrap(),
                ErrorKey::Validation,
                "`weight` can only be used in `random_` lists",
            );
        }
    }

    for (key, bv) in vd.unknown_keys() {
        if let Some((inscopes, effect)) = scope_effect(key, data) {
            if !inscopes.intersects(scopes | Scopes::None) {
                let msg = format!(
                    "effect is for {} but scope seems to be {}",
                    inscopes, scopes
                );
                warn(key, ErrorKey::Scopes, &msg);
            } else if inscopes != Scopes::None {
                scopes &= inscopes;
            }
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
                        (scopes, _) = validate_target(token, data, scopes, Scopes::Bool);
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
                    if let Some(token) = bv.expect_value() {
                        if let Ok(number) = token.as_str().parse::<i32>() {
                            if effect == Effect::NonNegativeValue && number < 0 {
                                let msg = format!("{} does not take negative numbers", key);
                                warn(token, ErrorKey::Validation, &msg);
                            }
                        }
                    }
                    scopes = ScriptValue::validate_bv(bv, data, scopes);
                }
                Effect::Scope(outscopes) => {
                    if let Some(token) = bv.expect_value() {
                        (scopes, _) = validate_target(token, data, scopes, outscopes);
                    }
                }
                Effect::Item(itype) => {
                    if let Some(token) = bv.expect_value() {
                        data.verify_exists(itype, token);
                    }
                }
                Effect::Unchecked => (),
                _ => (),
            }
            continue;
        }

        if let Some((it_type, it_name)) = key.split_once('_') {
            if it_type.is("any")
                || it_type.is("ordered")
                || it_type.is("every")
                || it_type.is("random")
            {
                if let Some((inscope, outscope)) = scope_iterator(&it_name, data) {
                    if it_type.is("any") {
                        let msg = "cannot use `any_` lists in an effect";
                        error(key, ErrorKey::Validation, msg);
                        continue;
                    }
                    if !inscope.intersects(scopes | Scopes::None) {
                        let msg = format!(
                            "iterator is for {} but scope seems to be {}",
                            inscope, scopes
                        );
                        warn(key, ErrorKey::Scopes, &msg);
                    } else if inscope != Scopes::None {
                        scopes &= inscope;
                    }
                    let ltype = match it_type.as_str() {
                        "every" => ListType::Every,
                        "ordered" => ListType::Ordered,
                        "random" => ListType::Random,
                        _ => unreachable!(),
                    };
                    if let Some(b) = bv.expect_block() {
                        let vd = Validator::new(b, data);
                        validate_effect(
                            it_name.as_str(),
                            ltype,
                            block,
                            data,
                            outscope,
                            vd,
                            tooltipped,
                        );
                    }
                    continue;
                }
            }
        }

        let msg = format!("unknown effect `{}`", key);
        warn(key, ErrorKey::Validation, &msg);
    }

    vd.warn_remaining();
    scopes
}
