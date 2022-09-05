/// A module for validation functions that are useful for more than one data module.
use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, Comparator};
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, warn, warn_info};
use crate::everything::Everything;
use crate::scopes::{
    scope_iterator, scope_prefix, scope_to_scope, scope_trigger_bool, scope_trigger_target,
    scope_value, Scopes,
};
use crate::token::Token;

pub fn validate_theme_background(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    // TODO: verify the background is defined
    vd.field_value("event_background");
    // TODO: check if `reference` actually works or is a mistake in vanilla
    vd.field_value("reference");
    vd.warn_remaining();
}

pub fn validate_theme_icon(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    // TODO: verify the file exists
    vd.field_value("reference"); // file
    vd.warn_remaining();
}

pub fn validate_theme_sound(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    vd.field_value("reference"); // event:/ resource reference
    vd.warn_remaining();
}

pub fn validate_cooldown(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    let mut count = 0;
    count += isize::from(vd.field_integer("years"));
    count += isize::from(vd.field_integer("months"));
    count += isize::from(vd.field_integer("days"));
    if count != 1 {
        warn(
            block,
            ErrorKey::Validation,
            "cooldown must have one of `years`, `months`, or `days`",
        );
    }
    vd.warn_remaining();
}

pub fn validate_color(block: &Block, _data: &Everything) {
    let mut count = 0;
    for (k, _, v) in block.iter_items() {
        if let Some(key) = k {
            error(key, ErrorKey::Validation, "expected color value");
        } else {
            match v {
                BlockOrValue::Token(t) => {
                    if let Ok(i) = t.as_str().parse::<isize>() {
                        if !(0..=255).contains(&i) {
                            error(
                                t,
                                ErrorKey::Validation,
                                "color values should be between 0 and 255",
                            );
                        }
                    } else if let Ok(f) = t.as_str().parse::<f64>() {
                        if !(0.0..=1.0).contains(&f) {
                            error(
                                t,
                                ErrorKey::Validation,
                                "color values should be between 0.0 and 1.0",
                            );
                        }
                    } else {
                        error(t, ErrorKey::Validation, "expected color value");
                    }
                    count += 1;
                }
                BlockOrValue::Block(b) => {
                    error(b, ErrorKey::Validation, "expected color value");
                }
            }
        }
    }
    if count != 3 {
        error(block, ErrorKey::Validation, "expected 3 color values");
    }
}

pub fn validate_scope_reference(prefix: &Token, arg: &Token, data: &Everything) {
    // TODO there are more to match
    match prefix.as_str() {
        "character" => data.characters.verify_exists(arg),
        "dynasty" => data.dynasties.verify_exists(arg),
        "faith" => data.religions.verify_faith_exists(arg),
        "house" => data.houses.verify_exists(arg),
        "province" => data.provinces.verify_exists(arg),
        "religion" => data.religions.verify_religion_exists(arg),
        "title" => data.titles.verify_exists(arg),
        &_ => (),
    }
}

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
                    (scopes, _) = validate_target(token, data, scopes, Scopes::all());

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

            if key.is("has_character_flag") {
                if !scopes.intersects(Scopes::Character) {
                    let msg = format!(
                        "{} is for {} but scope seems to be {}",
                        key,
                        Scopes::Character,
                        scopes
                    );
                    warn(key, ErrorKey::Scopes, &msg);
                }
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
                        validate_scope_reference(&prefix, &arg, data);
                        part_scopes = outscope;
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        continue 'outer;
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

pub fn validate_trigger_iterator(
    _name: &str,
    block: &Block,
    data: &Everything,
    mut scopes: Scopes,
) {
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
            }
        }
    }
    validate_trigger(block, data, scopes, &["count", "percent"]);
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
                validate_scope_reference(&prefix, &arg, data);
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
