use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, Comparator, Date};
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, warn, warn_info};
use crate::everything::Everything;
use crate::scopes::{
    scope_iterator, scope_prefix, scope_to_scope, scope_trigger_bool, scope_trigger_target,
    scope_value, Scopes,
};
use crate::token::Token;
use crate::validate::validate_prefix_reference;

pub fn validate_trigger(
    block: &Block,
    data: &Everything,
    mut scopes: Scopes,
    ignore_keys: &[&str],
) -> Scopes {
    'outer: for (key, cmp, bv) in block.iter_items() {
        if let Some(key) = key {
            if ignore_keys.contains(&key.as_str()) {
                continue;
            }
            if let Some((it_type, it_name)) = key.as_str().split_once('_') {
                if it_type == "any"
                    || it_type == "ordered"
                    || it_type == "every"
                    || it_type == "random"
                {
                    if let Some((inscope, outscope)) = scope_iterator(it_name) {
                        if it_type != "any" {
                            let msg = format!("cannot use `{}` in a trigger", key);
                            error(key, ErrorKey::Validation, &msg);
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
                        if let Some(b) = bv.get_block() {
                            validate_trigger_iterator(it_name, b, data, outscope);
                        } else {
                            error(bv, ErrorKey::Validation, "expected block, found value");
                        }
                        continue;
                    }
                }
            }

            if key.is("exists") {
                if let Some(token) = bv.expect_value() {
                    (scopes, _) = validate_target(token, data, scopes, Scopes::non_primitive());

                    if let Some(firstpart) = token.as_str().strip_suffix(".holder") {
                        advice_info(
                            key,
                            ErrorKey::Tooltip,
                            &format!(
                                "could rewrite this as `{} = {{ is_title_created = yes }}`",
                                firstpart
                            ),
                            "it gives a nicer tooltip",
                        );
                    }
                }
                continue;
            }

            if key.is("has_character_flag") || key.is("has_character_modifier") {
                scopes.expect_scope(key, Scopes::Character);
                bv.expect_value();
                continue;
            }

            if key.is("save_temporary_scope_as") {
                bv.expect_value();
                continue;
            }

            if key.is("AND")
                || key.is("OR")
                || key.is("NOT")
                || key.is("NOR")
                || key.is("NAND")
                || key.is("all_false")
                || key.is("any_false")
            {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(block, data, scopes, &[]);
                }
                continue;
            }

            if key.is("has_trait") {
                scopes.expect_scope(key, Scopes::Character);
                if let Some(token) = bv.expect_value() {
                    data.traits.verify_exists(token);
                }
                continue;
            }

            if key.is("has_county_modifier") {
                scopes.expect_scope(key, Scopes::LandedTitle);
                // TODO: validate
                bv.expect_value();
                continue;
            }

            if key.is("has_variable") {
                scopes.expect_scope(key, Scopes::non_primitive());
                bv.expect_value();
                continue;
            }

            if key.is("can_start_scheme") {
                scopes.expect_scope(key, Scopes::Character);
                if let Some(block) = bv.expect_block() {
                    verify_trigger_can_start_scheme(block, data, scopes);
                }
                continue;
            }

            if key.is("religion_tag") {
                scopes.expect_scope(key, Scopes::Faith);
                if let Some(token) = bv.expect_value() {
                    data.religions.verify_religion_exists(token);
                }
                continue;
            }

            if key.is("current_date") {
                if let Some(token) = bv.expect_value() {
                    if Date::try_from(token).is_err() {
                        error(token, ErrorKey::Validation, "expected date");
                    }
                }
                continue;
            }

            // TODO: check macro substitutions
            // TODO: check scope types;
            // if we narrowed it, validate scripted trigger with knowledge of our scope
            if data.triggers.exists(key) || data.events.trigger_exists(key) {
                if let Some(token) = bv.get_value() {
                    if !(token.is("yes") || token.is("no")) {
                        warn(token, ErrorKey::Validation, "expected yes or no");
                    }
                }
                // if it's a block instead, then it should contain macro arguments
                continue;
            }

            let part_vec = key.split('.');
            let mut part_scopes = scopes;
            for i in 0..part_vec.len() {
                let first = i == 0;
                let last = i + 1 == part_vec.len();
                let part = &part_vec[i];

                if let Some((prefix, arg)) = part.split_once(':') {
                    if let Some((inscope, outscope)) = scope_prefix(prefix.as_str()) {
                        if inscope == Scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        if !inscope.intersects(part_scopes | Scopes::None) {
                            let msg = format!(
                                "{}: is for {} but scope seems to be {}",
                                prefix, inscope, part_scopes
                            );
                            warn(part, ErrorKey::Scopes, &msg);
                        } else if first && inscope != Scopes::None {
                            scopes &= inscope;
                        }
                        validate_prefix_reference(&prefix, &arg, data);
                        part_scopes = outscope;
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        continue 'outer;
                    }
                } else if part.is("root")
                    || part.is("prev")
                    || part.is("this")
                    || part.is("ROOT")
                    || part.is("PREV")
                    || part.is("THIS")
                {
                    if !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if part.is("this") {
                        part_scopes = scopes;
                    } else {
                        part_scopes = Scopes::all();
                    }
                } else if let Some((inscope, outscope)) = scope_to_scope(part.as_str()) {
                    if inscope == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if !inscope.intersects(part_scopes | Scopes::None) {
                        let msg = format!(
                            "{} is for {} but scope seems to be {}",
                            part, inscope, part_scopes
                        );
                        warn(part, ErrorKey::Scopes, &msg);
                    } else if first && inscope != Scopes::None {
                        scopes &= inscope;
                    }
                    part_scopes = outscope;
                } else if let Some(inscope) = scope_value(part.as_str()) {
                    if !last {
                        let msg = format!("`{}` should be the last part", part);
                        warn(part, ErrorKey::Validation, &msg);
                        continue 'outer;
                    }
                    if inscope == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as only part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    } else if !inscope.intersects(part_scopes | Scopes::None) {
                        let msg = format!(
                            "{} is for {} but scope seems to be {}",
                            part, inscope, part_scopes
                        );
                        warn(part, ErrorKey::Scopes, &msg);
                    } else if first && inscope != Scopes::None {
                        scopes &= inscope;
                    }
                    part_scopes = Scopes::Value;
                } else if let Some((inscope, outscope)) = scope_trigger_target(part.as_str()) {
                    if !last {
                        let msg = format!("`{}` should be the last part", part);
                        error(part, ErrorKey::Validation, &msg);
                        continue 'outer;
                    } else if !inscope.intersects(part_scopes) {
                        let msg = format!(
                            "{} is for {} but scope seems to be {}",
                            part, inscope, part_scopes
                        );
                        warn(part, ErrorKey::Scopes, &msg);
                    } else if first {
                        scopes &= inscope;
                    }
                    part_scopes = outscope;
                } else if let Some(inscope) = scope_trigger_bool(part.as_str()) {
                    if !last {
                        let msg = format!("`{}` should be the last part", part);
                        warn(part, ErrorKey::Validation, &msg);
                        continue 'outer;
                    }
                    if inscope == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as only part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    } else if !inscope.intersects(part_scopes | Scopes::None) {
                        let msg = format!(
                            "{} is for {} but scope seems to be {}",
                            part, inscope, part_scopes
                        );
                        warn(part, ErrorKey::Scopes, &msg);
                    } else if first && inscope != Scopes::None {
                        scopes &= inscope;
                    }
                    part_scopes = Scopes::Bool;
                // TODO: warn if trying to use iterator here
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    continue 'outer;
                }
            }
            if !matches!(cmp, Comparator::Eq) {
                if part_scopes.intersects(Scopes::Value) {
                    scopes = ScriptValue::validate_bv(bv, data, scopes);
                    continue;
                } else {
                    let msg = format!("unexpected comparator {}", cmp);
                    warn(key, ErrorKey::Validation, &msg);
                }
            }
            // TODO: this needs to accept more constructions
            if part_scopes == Scopes::Bool {
                if let Some(token) = bv.expect_value() {
                    if !(token.is("yes") || token.is("no")) {
                        warn(token, ErrorKey::Validation, "expected yes or no");
                    }
                }
            } else if part_scopes == Scopes::Value {
                scopes = ScriptValue::validate_bv(bv, data, scopes);
            } else {
                match bv {
                    BlockOrValue::Token(t) => {
                        (scopes, _) = validate_target(t, data, scopes, part_scopes)
                    }
                    BlockOrValue::Block(b) => _ = validate_trigger(b, data, part_scopes, &[]),
                }
            }
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
    scopes
}

pub fn validate_trigger_iterator(name: &str, block: &Block, data: &Everything, mut scopes: Scopes) {
    let mut ignore = vec!["count", "percent"];
    for (key, _, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("percent") {
                if let Some(token) = bv.get_value() {
                    if let Ok(num) = token.as_str().parse::<f64>() {
                        if num > 1.0 {
                            warn(
                                token,
                                ErrorKey::Range,
                                "'percent' here needs to be between 0 and 1",
                            );
                        }
                    }
                }
                scopes = ScriptValue::validate_bv(bv, data, scopes);
            } else if key.is("count") {
                if let Some(token) = bv.get_value() {
                    if token.is("all") {
                        continue;
                    }
                }
                scopes = ScriptValue::validate_bv(bv, data, scopes);
            } else if name == "relation" && key.is("type") {
                if let Some(token) = bv.expect_value() {
                    data.relations.verify_exists(token);
                }
                ignore.push("type");
            }
        }
    }
    validate_trigger(block, data, scopes, &ignore);
}

pub fn validate_character_trigger(block: &Block, data: &Everything) {
    validate_trigger(block, data, Scopes::Character, &[]);
}

pub fn validate_target(
    token: &Token,
    data: &Everything,
    mut scopes: Scopes,
    outscopes: Scopes,
) -> (Scopes, Scopes) {
    if token.as_str().parse::<f64>().is_ok() {
        if !outscopes.intersects(Scopes::Value | Scopes::None) {
            let msg = format!("expected {}", outscopes);
            warn(token, ErrorKey::Scopes, &msg);
        }
        return (scopes, Scopes::Value);
    }
    let part_vec = token.split('.');
    let mut part_scopes = scopes;
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];

        if let Some((prefix, arg)) = part.split_once(':') {
            if let Some((inscope, outscope)) = scope_prefix(prefix.as_str()) {
                if inscope == Scopes::None && !first {
                    let msg = format!("`{}:` makes no sense except as first part", prefix);
                    warn(part, ErrorKey::Validation, &msg);
                }
                if !inscope.intersects(part_scopes | Scopes::None) {
                    let msg = format!(
                        "{}: is for {} but scope seems to be {}",
                        prefix, inscope, part_scopes
                    );
                    warn(part, ErrorKey::Scopes, &msg);
                } else if first && inscope != Scopes::None {
                    scopes &= inscope;
                }
                validate_prefix_reference(&prefix, &arg, data);
                part_scopes = outscope;
            } else {
                let msg = format!("unknown prefix `{}:`", prefix);
                error(part, ErrorKey::Validation, &msg);
                return (scopes, Scopes::all());
            }
        } else if part.is("root") || part.is("prev") || part.is("this") {
            if !first {
                let msg = format!("`{}` makes no sense except as first part", part);
                warn(part, ErrorKey::Validation, &msg);
            }
            if part.is("this") {
                part_scopes = scopes;
            } else {
                part_scopes = Scopes::all();
            }
        } else if let Some((inscope, outscope)) = scope_to_scope(part.as_str()) {
            if inscope == Scopes::None && !first {
                let msg = format!("`{}` makes no sense except as first part", part);
                warn(part, ErrorKey::Validation, &msg);
            }
            if !inscope.intersects(part_scopes | Scopes::None) {
                let msg = format!(
                    "{} is for {} but scope seems to be {}",
                    part, inscope, part_scopes
                );
                warn(part, ErrorKey::Scopes, &msg);
            } else if first && inscope != Scopes::None {
                scopes &= inscope;
            }
            part_scopes = outscope;
        } else if let Some(inscope) = scope_value(part.as_str()) {
            if !last {
                let msg = format!("`{}` only makes sense as the last part", part);
                warn(part, ErrorKey::Scopes, &msg);
                return (scopes, Scopes::all());
            }
            if inscope == Scopes::None && !first {
                let msg = format!("`{}` makes no sense except as first part", part);
                warn(part, ErrorKey::Validation, &msg);
            } else if !inscope.intersects(part_scopes | Scopes::None) {
                let msg = format!(
                    "{} is for {} but scope seems to be {}",
                    part, inscope, part_scopes
                );
                warn(part, ErrorKey::Scopes, &msg);
            } else if first && inscope != Scopes::None {
                scopes &= inscope;
            }
            part_scopes = Scopes::Value;
        // TODO: warn if trying to use iterator here
        } else {
            let msg = format!("unknown token `{}`", part);
            error(part, ErrorKey::Validation, &msg);
            return (scopes, Scopes::all());
        }
    }
    if !outscopes.intersects(part_scopes | Scopes::None) {
        let part = &part_vec[part_vec.len() - 1];
        let msg = format!(
            "`{}` produces {} but expected {}",
            part, part_scopes, outscopes
        );
        warn(part, ErrorKey::Scopes, &msg);
    }
    return (scopes, part_scopes);
}

fn verify_trigger_can_start_scheme(block: &Block, data: &Everything, scopes: Scopes) {
    let mut vd = Validator::new(block, data);

    vd.req_field("type");
    vd.req_field("target");
    vd.field_value("type");
    // TODO: validate scheme type
    if let Some(token) = vd.field_value("target") {
        validate_target(token, data, scopes, Scopes::Character);
    }

    vd.warn_remaining();
}
