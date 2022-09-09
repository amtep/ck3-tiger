use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, Comparator, Date};
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, warn, warn_info};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::{
    scope_iterator, scope_prefix, scope_to_scope, scope_trigger_bool, scope_trigger_item,
    scope_trigger_target, scope_value, Scopes,
};
use crate::token::Token;
use crate::validate::{validate_days_months_years, validate_prefix_reference};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Caller {
    Normal,
    If,
    CustomDescription,
    CustomTooltip,
    CalcTrueIf,

    // All lists should be below this entry, all non-lists above.
    AnyList,
    AnyInList,
    AnyRelationType,
    AnyCourtPositionType,
    AnyProvince,
    AnyInvolvement,
    AnyRegion,
    AnyClaim,
    AnyHierarchy,
}

pub fn validate_normal_trigger(block: &Block, data: &Everything, scopes: Scopes) -> Scopes {
    validate_trigger(Caller::Normal, block, data, scopes)
}

pub fn validate_trigger(
    caller: Caller,
    block: &Block,
    data: &Everything,
    mut scopes: Scopes,
) -> Scopes {
    let mut seen_if = false;

    'outer: for (key, cmp, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("limit") {
                if caller == Caller::If {
                    if let Some(block) = bv.expect_block() {
                        scopes = validate_normal_trigger(block, data, scopes);
                    }
                } else {
                    warn(key, ErrorKey::Validation, "can only use `limit` in `trigger_if` or `trigger_else_if` or `trigger_else`");
                }
                continue;
            }
            if key.is("trigger_if") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(Caller::If, block, data, scopes);
                }
                seen_if = true;
                continue;
            } else if key.is("trigger_else_if") {
                if !seen_if {
                    error(
                        key,
                        ErrorKey::Validation,
                        "must follow `trigger_if` or `trigger_else_if`",
                    );
                }
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(Caller::If, block, data, scopes);
                }
                continue;
            } else if key.is("trigger_else") {
                if !seen_if {
                    error(
                        key,
                        ErrorKey::Validation,
                        "must follow `trigger_if` or `trigger_else_if`",
                    );
                }
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(Caller::If, block, data, scopes);
                }
                seen_if = false;
                continue;
            }
            seen_if = false;

            if key.is("percent") {
                if caller < Caller::AnyList {
                    warn(
                        key,
                        ErrorKey::Validation,
                        "can only use `percent =` in an `any_` list",
                    );
                    continue;
                }
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
                continue;
            }
            if key.is("count") {
                if caller < Caller::AnyList {
                    warn(
                        key,
                        ErrorKey::Validation,
                        "can only use `count =` in an `any_` list",
                    );
                    continue;
                }
                if let Some(token) = bv.get_value() {
                    if token.is("all") {
                        continue;
                    }
                }
                scopes = ScriptValue::validate_bv(bv, data, scopes);
                continue;
            }

            if key.is("list") || key.is("variable") {
                if caller != Caller::AnyInList {
                    let msg = format!("can only use `{} =` in `any_in_list`, `any_in_global_list`, or `any_in_local_list`", key);
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                bv.expect_value(); // TODO
                continue;
            }

            if key.is("type") {
                if caller == Caller::AnyRelationType {
                    if let Some(token) = bv.expect_value() {
                        data.verify_exists(Item::Relation, token);
                    }
                } else if caller == Caller::AnyCourtPositionType {
                    bv.expect_value(); // TODO
                } else {
                    let msg = format!("can only use `{} =` in `any_relation` list", key);
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if key.is("province") {
                if caller != Caller::AnyProvince {
                    let msg = format!("can only use `{} =` in `any_pool_character` list", key);
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(token) = bv.expect_value() {
                    (scopes, _) = validate_target(token, data, scopes, Scopes::Province);
                }
                continue;
            }

            if key.is("even_if_dead") || key.is("only_if_dead") {
                if caller < Caller::AnyList || !scopes.intersects(Scopes::Character) {
                    let msg = format!("can only use `{} =` in a character list", key);
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                // TODO Might be literal yes/no expected, might be any bool value
                bv.expect_value();
                continue;
            }

            if key.is("involvement") {
                if caller != Caller::AnyInvolvement {
                    let msg = format!("can only use `{} =` in `any_character_struggle` list", key);
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                bv.expect_value(); // TODO "involved" or "interloper"
                continue;
            }

            if key.is("region") {
                if caller != Caller::AnyRegion {
                    let msg = format!("can only use `{} =` in `any_county_in_region` list", key);
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(Item::Region, token);
                }
                continue;
            }

            if key.is("filter") || key.is("continue") {
                if caller != Caller::AnyHierarchy {
                    let msg = format!("can only use `{} =` in `..._hierarchy` list", key);
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                if let Some(block) = bv.expect_block() {
                    scopes = validate_normal_trigger(block, data, scopes);
                }
                continue;
            }

            if key.is("pressed") || key.is("explicit") {
                if caller != Caller::AnyClaim {
                    let msg = format!("can only use `{} =` in `any_claim` list", key);
                    warn(key, ErrorKey::Validation, &msg);
                    continue;
                }
                bv.expect_value(); // TODO: check yes/no/all
                continue;
            }

            if key.is("text") {
                if caller == Caller::CustomDescription {
                    // TODO: validate trigger_localization
                    bv.expect_value();
                } else if caller == Caller::CustomTooltip {
                    if let Some(token) = bv.expect_value() {
                        data.localization.verify_exists(token);
                    }
                } else {
                    let msg = format!(
                        "can only use `{} =` in `custom_description` or `custom_tooltip`",
                        key
                    );
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if key.is("subject") {
                if caller == Caller::CustomDescription || caller == Caller::CustomTooltip {
                    if let Some(token) = bv.expect_value() {
                        (scopes, _) = validate_target(token, data, scopes, Scopes::non_primitive());
                    }
                } else {
                    let msg = format!(
                        "can only use `{} =` in `custom_description` or `custom_tooltip`",
                        key
                    );
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }
            if key.is("object") {
                if caller == Caller::CustomDescription {
                    if let Some(token) = bv.expect_value() {
                        (scopes, _) = validate_target(token, data, scopes, Scopes::non_primitive());
                    }
                } else {
                    let msg = format!("can only use `{} =` in `custom_description`", key);
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if key.is("amount") {
                if caller == Caller::CalcTrueIf {
                    if let Some(token) = bv.expect_value() {
                        if token.as_str().parse::<i32>().is_err() {
                            warn(token, ErrorKey::Validation, "expected a number");
                        }
                    }
                } else {
                    let msg = format!("can only use `{} =` in `calc_true_if`", key);
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
                    if let Some((inscope, outscope)) = scope_iterator(&it_name, data) {
                        if !it_type.is("any") {
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
                            validate_trigger_iterator(&it_name, b, data, outscope);
                        } else {
                            error(bv, ErrorKey::Validation, "expected block, found value");
                        }
                        continue;
                    }
                }
            }

            if key.is("custom_description") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(Caller::CustomDescription, block, data, scopes);
                }
                continue;
            }

            if key.is("custom_tooltip") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(Caller::CustomTooltip, block, data, scopes);
                }
                continue;
            }

            if key.is("calc_true_if") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(Caller::CalcTrueIf, block, data, scopes);
                }
                continue;
            }

            if let Some((inscope, item)) = scope_trigger_item(key.as_str()) {
                if !inscope.intersects(scopes | Scopes::None) {
                    let msg = format!(
                        "{} is for {} but scope seems to be {}",
                        key, inscope, scopes
                    );
                    warn(key, ErrorKey::Scopes, &msg);
                }
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(item, token);
                }
                continue;
            }

            let (scopes2, handled) = validate_trigger_keys(key, bv, data, scopes);
            if handled {
                continue;
            }
            scopes = scopes2; // TODO: this is ugly

            // TODO: check macro substitutions
            // TODO: check scope types;
            // if we narrowed it, validate scripted trigger with knowledge of our scope
            if data.triggers.exists(key.as_str()) || data.events.trigger_exists(key) {
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
                } else if let Some(inscope) = scope_value(part, data) {
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
                } else if let Some((inscope, outscope)) = scope_trigger_target(part, data) {
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
                } else if data.scriptvalues.exists(part.as_str()) {
                    if !last {
                        let msg = "script value should be the last part";
                        warn(part, ErrorKey::Validation, msg);
                        continue 'outer;
                    }
                    // TODO: check script value's scoping
                    part_scopes = Scopes::Value;
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
                } else {
                    let msg = format!("unexpected comparator {}", cmp);
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }

            if part_scopes == Scopes::Bool {
                if let Some(_token) = bv.expect_value() {
                    scopes = ScriptValue::validate_bv(bv, data, scopes);
                    // TODO: get outscope from ScriptValue because it can be either Value or Bool.
                    // Then check if it's Bool here.
                }
            } else if part_scopes == Scopes::Value {
                scopes = ScriptValue::validate_bv(bv, data, scopes);
            } else {
                match bv {
                    BlockOrValue::Token(t) => {
                        (scopes, _) = validate_target(t, data, scopes, part_scopes);
                    }
                    BlockOrValue::Block(b) => _ = validate_normal_trigger(b, data, part_scopes),
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

pub fn validate_trigger_iterator(name: &Token, block: &Block, data: &Everything, scopes: Scopes) {
    let caller = match name.as_str() {
        "in_list" | "in_global_list" | "in_local_list" => Caller::AnyInList,
        "relation" => Caller::AnyRelationType,
        "court_position_holder" => Caller::AnyCourtPositionType,
        "pool_character" => Caller::AnyProvince,
        "character_struggle" => Caller::AnyInvolvement,
        "county_in_region" => Caller::AnyRegion,
        "in_de_jure_hierarchy" | "in_de_facto_hierarchy" => Caller::AnyHierarchy,
        "claim" => Caller::AnyClaim,
        _ => Caller::AnyList,
    };
    validate_trigger(caller, block, data, scopes);
}

pub fn validate_character_trigger(block: &Block, data: &Everything) {
    validate_normal_trigger(block, data, Scopes::Character);
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
        } else if let Some(inscope) = scope_value(part, data) {
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
    (scopes, part_scopes)
}

/// Validate the keys that don't follow a consistent pattern in what they require from their
/// block or value.
/// Returns true iff the key was recognized (and handled)
/// LAST UPDATED VERSION 1.6.2.2
/// See `triggers.log` for details
#[allow(clippy::match_same_arms)] // many of these "same arms" just need further coding
fn validate_trigger_keys(
    key: &Token,
    bv: &BlockOrValue,
    data: &Everything,
    mut scopes: Scopes,
) -> (Scopes, bool) {
    let match_key: &str = &key.as_str().to_lowercase();
    match match_key {
        "add_to_temporary_list" => {
            // TODO: if inside an any_ iterator, this should be at the end.
            bv.expect_value();
        }

        "ai_diplomacy_stance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_ai_diplomacy_stance(block, data, scopes);
            }
        }

        "ai_values_divergence" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "and" | "or" | "not" | "nor" | "nand" | "all_false" | "any_false" => {
            if let Some(block) = bv.expect_block() {
                scopes = validate_normal_trigger(block, data, scopes);
            }
        }

        "amenity_level" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_amenity_level(block, data, scopes);
            }
        }

        "aptitude" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_aptitude(block, data, scopes);
            }
        }

        "all_court_artifact_slots" | "all_inventory_artifact_slots" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "can_add_hook" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_add_hook(block, data, scopes);
            }
        }

        "can_be_employed_as" | "can_employ_court_position_type" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "can_create_faction" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_target(block, data, Item::Faction, scopes, Scopes::Character);
            }
        }

        "can_declare_war" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_declare_war(block, data, scopes);
            }
        }

        "can_join_or_create_faction_against" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_join_or_create_faction_against(block, data, scopes);
            }
        }

        "can_start_scheme" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_target(block, data, Item::Scheme, scopes, Scopes::Character);
            }
        }

        "can_start_tutorial_lesson" => {
            bv.expect_value();
        }

        "can_title_create_faction" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_target(block, data, Item::Faction, scopes, Scopes::Character);
            }
        }

        "county_opinion_target" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "create_faction_type_chance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_create_faction_type_chance(block, data, scopes);
            }
        }

        "cultural_acceptance" => {
            scopes.expect_scope(key, Scopes::Culture);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Culture);
            }
        }

        "current_computer_date" | "current_date" | "game_start_date" => {
            if let Some(token) = bv.expect_value() {
                if Date::try_from(token).is_err() {
                    error(token, ErrorKey::Validation, "expected date");
                }
            }
        }

        "de_jure_drift_progress" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::LandedTitle);
            }
        }

        "death_reason" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "diplomacy_diff" | "intrigue_diff" | "learning_diff" | "martial_diff" | "prowess_diff"
        | "stewardship_diff" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_skill_diff(block, data, scopes);
            }
        }

        "dread_modified_ai_boldness" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_dread_modified_ai_boldness(block, data, scopes);
            }
        }

        "employs_court_position" | "is_court_position_employer" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "exists" => {
            if let Some(token) = bv.expect_value() {
                if token.is("yes") || token.is("no") {
                    // TODO: check scope is not none?
                } else {
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
            }
        }

        "faith_hostility_level" => {
            scopes.expect_scope(key, Scopes::Faith);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Faith);
            }
        }

        "faith_hostility_level_comparison" => {
            scopes.expect_scope(key, Scopes::Faith);
            if let Some(block) = bv.expect_block() {
                validate_trigger_faith_hostility_level_comparison(block, data, scopes);
            }
        }

        "global_variable_list_size"
        | "list_size"
        | "local_variable_list_size"
        | "variable_list_size" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_list_size(block, data, scopes);
            }
        }

        "government_allows" | "government_disallows" | "government_has_flag" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_all_innovations" => {
            scopes.expect_scope(key, Scopes::Culture);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_all_innovations(block, data, scopes);
            }
        }

        "has_building_with_flag" => {
            scopes.expect_scope(key, Scopes::Province);
            match bv {
                BlockOrValue::Block(block) => {
                    validate_trigger_has_building_with_flag(block, data, scopes);
                }
                BlockOrValue::Token(token) => {
                    data.verify_exists(Item::BuildingFlag, token);
                }
            }
        }

        "has_cb_on" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_cb_on(block, data, scopes);
            }
        }

        "has_character_flag"
        | "has_council_position"
        | "has_councillor_for_skill"
        | "has_court_language"
        | "has_court_position"
        | "has_court_type"
        | "has_trait_with_flag"
        | "highest_skill" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_dlc"
        | "has_dlc_feature"
        | "has_game_rule"
        | "has_global_variable"
        | "has_global_variable_list"
        | "has_local_variable"
        | "has_local_variable_list"
        | "has_map_mode"
        | "has_variable"
        | "has_variable_list"
        | "has_war_result_message_with_outcome"
        | "is_bad_nickname"
        | "is_game_view_open"
        | "is_in_list"
        | "is_tooltip_with_name_open"
        | "is_tutorial_lesson_active"
        | "is_tutorial_lesson_chain_completed"
        | "is_tutorial_lesson_completed"
        | "is_tutorial_lesson_step_completed"
        | "is_war_overview_tab_open"
        | "is_widget_open" => {
            bv.expect_value();
        }

        "has_dread_level_towards" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_dread_level_towards(block, data, scopes);
            }
        }

        "has_election_vote_of" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_election_vote_of(block, data, scopes);
            }
        }

        "has_focus" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_gene" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_gene(block, data, scopes);
            }
        }

        "has_government" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_hook_of_type" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_hook_of_type(block, data, scopes);
            }
        }

        "has_memory_category" | "has_memory_type" => {
            scopes.expect_scope(key, Scopes::CharacterMemory);
            bv.expect_value();
        }

        "has_nickname" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_opinion_modifier" | "reverse_has_opinion_modifier" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_opinion_modifier(block, data, scopes);
            }
        }

        "has_order_of_succession" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            // The only known value for this is "election"
            bv.expect_value();
        }

        "has_perk" | "has_realm_law" | "has_realm_law_flag" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_relation_flag" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_relation_flag(block, data, scopes);
            }
        }

        "has_sexuality" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_trait_rank" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_trait_rank(block, data, scopes);
            }
        }

        "important_action_is_valid_but_invisible"
        | "important_action_is_visible"
        | "in_activity_type" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "is_character_interaction_potentially_accepted"
        | "is_character_interaction_shown"
        | "is_character_interaction_valid" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_character_interaction(block, data, scopes);
            }
        }

        "is_connected_to" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_connected_to(block, data, scopes);
            }
        }

        "is_council_task_valid" | "is_in_prison_type" | "is_performing_council_task" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "is_in_family" => {
            scopes.expect_scope(key, Scopes::Religion);
            bv.expect_value();
        }

        "is_scheming_against" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_scheming_against(block, data, scopes);
            }
        }

        "is_target_in_global_variable_list"
        | "is_target_in_local_variable_list"
        | "is_target_in_variable_list" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_target_in_list(block, data, scopes);
            }
        }

        "join_faction_chance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Faction);
            }
        }

        "join_scheme_chance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_join_scheme_chance(block, data, scopes);
            }
        }

        "knows_language" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "max_number_maa_soldiers_of_base_type"
        | "number_maa_regiments_of_base_type"
        | "number_maa_soldiers_of_base_type" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_value(block, data, "type", Item::MenAtArmsBase, scopes);
            }
        }

        "max_number_maa_soldiers_of_type"
        | "number_maa_regiments_of_type"
        | "number_maa_soldiers_of_type" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_value(block, data, "type", Item::MenAtArms, scopes);
            }
        }

        "morph_gene_attribute" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_morph_gene_attribute(block, data, scopes);
            }
        }

        "morph_gene_value" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_morph_gene_value(block, data, scopes);
            }
        }

        "number_of_commander_traits_in_common"
        | "number_of_opposing_personality_traits"
        | "number_of_opposing_traits"
        | "number_of_personality_traits_in_common"
        | "number_of_traits_in_common" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "number_of_election_votes" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_number_of_election_votes(block, data, scopes);
            }
        }

        "number_of_sinful_traits_in_common" | "number_of_virtue_traits_in_common" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "opinion" | "reverse_opinion" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "perks_in_tree" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_perks_in_tree(block, data, scopes);
            }
        }

        "place_in_line_of_succession" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "player_heir_position" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "realm_to_title_distance_squared" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_realm_to_title_distance_squared(block, data, scopes);
            }
        }

        "save_temporary_opinion_value_as" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_save_temporary_opinion_value_as(block, data, scopes);
            }
        }

        "save_temporary_scope_as" => {
            bv.expect_value();
        }

        "save_temporary_scope_value_as" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_save_temporary_scope_value_as(block, data, scopes);
            }
        }

        "squared_distance" => {
            scopes.expect_scope(key, Scopes::LandedTitle | Scopes::Province);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(
                    block,
                    data,
                    scopes,
                    Scopes::LandedTitle | Scopes::Province,
                );
            }
        }

        "time_to_hook_expiry" | "trait_compatibility" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "title_join_faction_chance" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_value(block, data, "faction", Item::Faction, scopes);
            }
        }

        "recent_history" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_recent_history(block, data, scopes);
            }
        }

        "tier_difference" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "time_in_prison" | "time_in_prison_type" | "time_since_death" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_days_months_years(block, data, scopes);
            }
        }

        "time_of_year" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_time_of_year(block, data, scopes);
            }
        }

        "title_create_faction_type_chance" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_create_faction_type_chance(block, data, scopes);
            }
        }

        "vassal_contract_has_flag"
        | "vassal_contract_obligation_level_can_be_decreased"
        | "vassal_contract_obligation_level_can_be_increased" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "war_contribution" => {
            scopes.expect_scope(key, Scopes::War);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "yields_alliance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_yields_alliance(block, data, scopes);
            }
        }

        _ => {
            return (scopes, false);
        }
    }
    (scopes, true)
}

fn validate_trigger_target_value(
    block: &Block,
    data: &Everything,
    scopes: Scopes,
    outscopes: Scopes,
) {
    let mut vd = Validator::new(block, data);

    vd.req_field("target");
    vd.req_field("value");
    if let Some(token) = vd.field_value("target") {
        validate_target(token, data, scopes, outscopes);
    }
    if let Some(bv) = vd.field_any_cmp("value") {
        ScriptValue::validate_bv(bv, data, scopes);
    }

    vd.warn_remaining();
}

fn validate_trigger_type_value(
    block: &Block,
    data: &Everything,
    field: &'static str,
    itype: Item,
    scopes: Scopes,
) {
    let mut vd = Validator::new(block, data);

    vd.req_field(field);
    vd.req_field("value");
    vd.field_value_item(field, itype);
    if let Some(bv) = vd.field_any_cmp("value") {
        ScriptValue::validate_bv(bv, data, scopes);
    }

    vd.warn_remaining();
}

fn validate_trigger_type_target(
    block: &Block,
    data: &Everything,
    itype: Item,
    scopes: Scopes,
    outscopes: Scopes,
) {
    let mut vd = Validator::new(block, data);

    vd.req_field("type");
    vd.req_field("target");
    vd.field_value_item("type", itype);
    if let Some(token) = vd.field_value("target") {
        validate_target(token, data, scopes, outscopes);
    }

    vd.warn_remaining();
}

fn validate_trigger_recent_history(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_join_faction_chance(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_all_innovations(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_faith_hostility_level_comparison(
    _block: &Block,
    _data: &Everything,
    _scopes: Scopes,
) {
    // TODO
}

fn validate_trigger_has_building_with_flag(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_list_size(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_is_target_in_list(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_save_temporary_scope_value_as(
    _block: &Block,
    _data: &Everything,
    _scopes: Scopes,
) {
    // TODO
}

fn validate_trigger_save_temporary_opinion_value_as(
    _block: &Block,
    _data: &Everything,
    _scopes: Scopes,
) {
    // TODO
}

fn validate_trigger_time_of_year(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_ai_diplomacy_stance(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_can_create_faction(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_amenity_level(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_aptitude(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_can_add_hook(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_can_start_scheme(block: &Block, data: &Everything, scopes: Scopes) {
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

fn validate_trigger_can_declare_war(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_can_join_or_create_faction_against(
    _block: &Block,
    _data: &Everything,
    _scopes: Scopes,
) {
    // TODO
}

fn validate_trigger_create_faction_type_chance(
    _block: &Block,
    _data: &Everything,
    _scopes: Scopes,
) {
    // TODO
}

fn validate_trigger_skill_diff(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_dread_modified_ai_boldness(
    _block: &Block,
    _data: &Everything,
    _scopes: Scopes,
) {
    // TODO
}

fn validate_trigger_has_cb_on(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_dread_level_towards(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_election_vote_of(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_number_of_election_votes(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_gene(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_hook_of_type(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_opinion_modifier(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_relation_flag(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_has_trait_rank(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_is_character_interaction(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_is_connected_to(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_is_scheming_against(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_join_scheme_chance(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_morph_gene_attribute(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_morph_gene_value(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_number_maa_of_base_type(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_number_maa_of_type(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_perks_in_tree(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}

fn validate_trigger_realm_to_title_distance_squared(
    _block: &Block,
    _data: &Everything,
    _scopes: Scopes,
) {
    // TODO
}

fn validate_trigger_yields_alliance(_block: &Block, _data: &Everything, _scopes: Scopes) {
    // TODO
}
