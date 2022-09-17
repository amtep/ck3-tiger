use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, Comparator, Date};
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, warn, warn_info};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, scope_value, Scopes};
use crate::tables::triggers::{scope_trigger_bool, scope_trigger_item, scope_trigger_target};
use crate::token::Token;
use crate::validate::{validate_days_weeks_months_years, validate_prefix_reference};

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

pub fn validate_normal_trigger(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    validate_trigger(Caller::Normal, block, data, sc, tooltipped);
}

pub fn validate_trigger(
    caller: Caller,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
    let mut seen_if = false;

    'outer: for (key, cmp, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("limit") {
                if caller == Caller::If {
                    if let Some(block) = bv.expect_block() {
                        validate_normal_trigger(block, data, sc, tooltipped);
                    }
                } else {
                    warn(key, ErrorKey::Validation, "can only use `limit` in `trigger_if` or `trigger_else_if` or `trigger_else`");
                }
                continue;
            }
            if key.is("trigger_if") {
                if let Some(block) = bv.expect_block() {
                    validate_trigger(Caller::If, block, data, sc, tooltipped);
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
                    validate_trigger(Caller::If, block, data, sc, tooltipped);
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
                    validate_trigger(Caller::If, block, data, sc, tooltipped);
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
                ScriptValue::validate_bv(bv, data, sc);
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
                ScriptValue::validate_bv(bv, data, sc);
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
                    validate_target(token, data, sc, Scopes::Province);
                }
                continue;
            }

            if key.is("even_if_dead") || key.is("only_if_dead") {
                if caller < Caller::AnyList || !sc.can_be(Scopes::Character) {
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
                    validate_normal_trigger(block, data, sc, false);
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
                        validate_target(token, data, sc, Scopes::non_primitive());
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
                        validate_target(token, data, sc, Scopes::non_primitive());
                    }
                } else {
                    let msg = format!("can only use `{} =` in `custom_description`", key);
                    warn(key, ErrorKey::Validation, &msg);
                }
                continue;
            }
            if key.is("value") {
                if caller == Caller::CustomDescription {
                    ScriptValue::validate_bv(bv, data, sc);
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
                    if let Some((inscopes, outscope)) = scope_iterator(&it_name, data) {
                        if !it_type.is("any") {
                            let msg = format!("cannot use `{}_` list in a trigger", key);
                            error(key, ErrorKey::Validation, &msg);
                            continue;
                        }
                        sc.expect(inscopes, key);
                        if let Some(b) = bv.get_block() {
                            sc.open_scope(outscope, key.clone());
                            validate_trigger_iterator(&it_name, b, data, sc, tooltipped);
                            sc.close();
                        } else {
                            error(bv, ErrorKey::Validation, "expected block, found value");
                        }
                        continue;
                    }
                }
            }

            if key.is("custom_description") {
                if let Some(block) = bv.expect_block() {
                    validate_trigger(Caller::CustomDescription, block, data, sc, false);
                }
                continue;
            }

            if key.is("custom_tooltip") {
                if let Some(block) = bv.expect_block() {
                    validate_trigger(Caller::CustomTooltip, block, data, sc, false);
                }
                continue;
            }

            if key.is("calc_true_if") {
                if let Some(block) = bv.expect_block() {
                    validate_trigger(Caller::CalcTrueIf, block, data, sc, tooltipped);
                }
                continue;
            }
            if key.is("weighted_calc_true_if") {
                bv.expect_block();
                // TODO
                continue;
            }

            if let Some((inscopes, item)) = scope_trigger_item(key.as_str()) {
                sc.expect(inscopes, key);
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(item, token);
                }
                continue;
            }

            let handled = validate_trigger_keys(key, bv, data, sc, tooltipped);
            if handled {
                continue;
            }

            if let Some(trigger) = data.get_trigger(key) {
                match bv {
                    BlockOrValue::Token(token) => {
                        if !(token.is("yes") || token.is("no")) {
                            warn(token, ErrorKey::Validation, "expected yes or no");
                        }
                        if !trigger.macro_parms().is_empty() {
                            error(token, ErrorKey::Macro, "expected macro arguments");
                        }
                        trigger.validate_call(&key.loc, data, sc, tooltipped);
                    }
                    BlockOrValue::Block(block) => {
                        let parms = trigger.macro_parms();
                        if parms.is_empty() {
                            error(
                                block,
                                ErrorKey::Macro,
                                "trigger does not need macro arguments",
                            );
                        } else {
                            let mut vec = Vec::new();
                            let mut vd = Validator::new(block, data);
                            for parm in &parms {
                                vd.req_field(parm);
                                if let Some(token) = vd.field_value(parm.as_str()) {
                                    vec.push(token.clone());
                                } else {
                                    continue 'outer;
                                }
                            }
                            let args = parms.into_iter().zip(vec.into_iter()).collect();
                            trigger.validate_macro_expansion(args, data, sc, tooltipped);
                        }
                    }
                }
                continue;
            }

            // `10 < scriptvalue` is a valid trigger
            if key.as_str().parse::<f64>().is_ok() {
                ScriptValue::validate_bv(bv, data, sc);
                continue;
            }

            let part_vec = key.split('.');
            sc.open_builder();
            let mut warn_against_eq = None;
            for i in 0..part_vec.len() {
                let first = i == 0;
                let last = i + 1 == part_vec.len();
                let part = &part_vec[i];

                if let Some((prefix, arg)) = part.split_once(':') {
                    if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                        if inscopes == Scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        sc.expect(inscopes, &prefix);
                        validate_prefix_reference(&prefix, &arg, data);
                        sc.replace(outscope, part.clone());
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        sc.close();
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
                    if part.is("root") || part.is("ROOT") {
                        sc.replace_root();
                    } else if part.is("prev") || part.is("PREV") {
                        sc.replace_prev(part);
                    } else {
                        sc.replace_this();
                    }
                } else if let Some((inscopes, outscope)) = scope_to_scope(part.as_str()) {
                    if inscopes == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    sc.expect(inscopes, part);
                    sc.replace(outscope, part.clone());
                } else if let Some(inscopes) = scope_value(part, data) {
                    if !last {
                        let msg = format!("`{}` should be the last part", part);
                        warn(part, ErrorKey::Validation, &msg);
                        sc.close();
                        continue 'outer;
                    }
                    if inscopes == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as only part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if WARN_AGAINST_EQ.contains(&part.as_str()) {
                        warn_against_eq = Some(part);
                    }
                    sc.expect(inscopes, part);
                    sc.replace(Scopes::Value, part.clone());
                } else if let Some((inscopes, outscope)) = scope_trigger_target(part, data) {
                    if !last {
                        let msg = format!("`{}` should be the last part", part);
                        error(part, ErrorKey::Validation, &msg);
                        sc.close();
                        continue 'outer;
                    }
                    sc.expect(inscopes, part);
                    sc.replace(outscope, part.clone());
                } else if let Some(inscopes) = scope_trigger_bool(part.as_str()) {
                    if !last {
                        let msg = format!("`{}` should be the last part", part);
                        warn(part, ErrorKey::Validation, &msg);
                        sc.close();
                        continue 'outer;
                    }
                    if inscopes == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as only part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    sc.expect(inscopes, part);
                    sc.replace(Scopes::Bool, part.clone());
                } else if data.scriptvalues.exists(part.as_str()) {
                    if !last {
                        let msg = "script value should be the last part";
                        warn(part, ErrorKey::Validation, msg);
                        sc.close();
                        continue 'outer;
                    }
                    data.scriptvalues.validate_call(part, data, sc);
                    sc.replace(Scopes::Value, part.clone());
                // TODO: warn if trying to use iterator here
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    sc.close();
                    continue 'outer;
                }
            }

            if matches!(cmp, Comparator::Eq) {
                if let Some(token) = warn_against_eq {
                    let msg = format!("`{} =` means exactly equal to that amount, which is usually not what you want", token);
                    warn(token, ErrorKey::Logic, &msg);
                }
            } else {
                if sc.can_be(Scopes::Value) {
                    sc.close();
                    ScriptValue::validate_bv(bv, data, sc);
                } else {
                    let msg = format!("unexpected comparator {}", cmp);
                    warn(key, ErrorKey::Validation, &msg);
                    sc.close();
                }
                continue;
            }

            if sc.must_be(Scopes::Bool) {
                sc.close();
                if let Some(_token) = bv.expect_value() {
                    ScriptValue::validate_bv(bv, data, sc);
                    // TODO: get outscope from ScriptValue because it can be either Value or Bool.
                    // Then check if it's Bool here.
                }
            } else if sc.must_be(Scopes::Value) {
                sc.close();
                ScriptValue::validate_bv(bv, data, sc);
            } else {
                match bv {
                    BlockOrValue::Token(t) => {
                        let scopes = sc.scopes();
                        sc.close();
                        validate_target(t, data, sc, scopes);
                    }
                    BlockOrValue::Block(b) => {
                        validate_normal_trigger(b, data, sc, tooltipped);
                        sc.close();
                    }
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
}

fn validate_trigger_iterator(
    name: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: bool,
) {
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
    // TODO: recommend custom = for iterators?
    validate_trigger(caller, block, data, sc, tooltipped);
}

pub fn validate_target(token: &Token, data: &Everything, sc: &mut ScopeContext, outscopes: Scopes) {
    if token.as_str().parse::<f64>().is_ok() {
        if !outscopes.intersects(Scopes::Value | Scopes::None) {
            let msg = format!("expected {}", outscopes);
            warn(token, ErrorKey::Scopes, &msg);
        }
        return;
    }
    let part_vec = token.split('.');
    sc.open_builder();
    for i in 0..part_vec.len() {
        let first = i == 0;
        let last = i + 1 == part_vec.len();
        let part = &part_vec[i];

        if let Some((prefix, arg)) = part.split_once(':') {
            if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                if inscopes == Scopes::None && !first {
                    let msg = format!("`{}:` makes no sense except as first part", prefix);
                    warn(part, ErrorKey::Validation, &msg);
                }
                sc.expect(inscopes, &prefix);
                validate_prefix_reference(&prefix, &arg, data);
                sc.replace(outscope, part.clone());
            } else {
                let msg = format!("unknown prefix `{}:`", prefix);
                error(part, ErrorKey::Validation, &msg);
                sc.close();
                return;
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
            if part.is("root") || part.is("ROOT") {
                sc.replace_root();
            } else if part.is("prev") || part.is("PREV") {
                sc.replace_prev(part);
            } else {
                sc.replace_this();
            }
        } else if let Some((inscopes, outscope)) = scope_to_scope(part.as_str()) {
            if inscopes == Scopes::None && !first {
                let msg = format!("`{}` makes no sense except as first part", part);
                warn(part, ErrorKey::Validation, &msg);
            }
            sc.expect(inscopes, part);
            sc.replace(outscope, part.clone());
        } else if let Some(inscopes) = scope_value(part, data) {
            if !last {
                let msg = format!("`{}` only makes sense as the last part", part);
                warn(part, ErrorKey::Scopes, &msg);
                sc.close();
                return;
            }
            if inscopes == Scopes::None && !first {
                let msg = format!("`{}` makes no sense except as first part", part);
                warn(part, ErrorKey::Validation, &msg);
            }
            sc.expect(inscopes, part);
            sc.replace(Scopes::Value, part.clone());
        } else if data.scriptvalues.exists(part.as_str()) {
            // TODO: validate inscope of the script value against sc
            if !last {
                let msg = format!("`{}` only makes sense as the last part", part);
                warn(part, ErrorKey::Scopes, &msg);
                sc.close();
                return;
            }
            sc.replace(Scopes::Value, part.clone());
        // TODO: warn if trying to use iterator here
        } else {
            let msg = format!("unknown token `{}`", part);
            error(part, ErrorKey::Validation, &msg);
            sc.close();
            return;
        }
    }
    if !outscopes.intersects(sc.scopes() | Scopes::None) {
        let part = &part_vec[part_vec.len() - 1];
        let msg = format!(
            "`{}` produces {} but expected {}",
            part,
            sc.scopes(),
            outscopes
        );
        warn(part, ErrorKey::Scopes, &msg);
    }
    sc.close();
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
    sc: &mut ScopeContext,
    tooltipped: bool,
) -> bool {
    let match_key: &str = &key.as_str().to_lowercase();
    match match_key {
        "add_to_temporary_list" => {
            // TODO: if inside an any_ iterator, this should be at the end.
            bv.expect_value();
        }

        "ai_diplomacy_stance" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_ai_diplomacy_stance(block, data, sc);
            }
        }

        "ai_values_divergence" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "and" | "or" | "not" | "nor" | "nand" | "all_false" | "any_false" => {
            if let Some(block) = bv.expect_block() {
                validate_normal_trigger(block, data, sc, tooltipped);
            }
        }

        "amenity_level" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_amenity_level(block, data, sc);
            }
        }

        "aptitude" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_aptitude(block, data, sc);
            }
        }

        "all_court_artifact_slots" | "all_inventory_artifact_slots" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "can_add_hook" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_add_hook(block, data, sc);
            }
        }

        "can_be_employed_as" | "can_employ_court_position_type" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "can_create_faction" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_target(block, data, Item::Faction, sc, Scopes::Character);
            }
        }

        "can_declare_war" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_declare_war(block, data, sc);
            }
        }

        "can_join_or_create_faction_against" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_join_or_create_faction_against(block, data, sc);
            }
        }

        "can_start_scheme" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_target(block, data, Item::Scheme, sc, Scopes::Character);
            }
        }

        "can_start_tutorial_lesson" => {
            bv.expect_value();
        }

        "can_title_create_faction" => {
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_target(block, data, Item::Faction, sc, Scopes::Character);
            }
        }

        "county_opinion_target" => {
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "create_faction_type_chance" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_create_faction_type_chance(block, data, sc);
            }
        }

        "cultural_acceptance" => {
            sc.expect(Scopes::Culture, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Culture);
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
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::LandedTitle);
            }
        }

        "death_reason" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "diplomacy_diff" | "intrigue_diff" | "learning_diff" | "martial_diff" | "prowess_diff"
        | "stewardship_diff" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_skill_diff(block, data, sc);
            }
        }

        "dread_modified_ai_boldness" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_dread_modified_ai_boldness(block, data, sc);
            }
        }

        "employs_court_position" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "is_court_position_employer" => {
            sc.expect(Scopes::Character, key);
            // undocumented, but it expects court_position = Item::CourtPosition
            // and who = target character
            bv.expect_block();
        }

        "exists" => {
            if let Some(token) = bv.expect_value() {
                if token.is("yes") || token.is("no") {
                    // TODO: check scope is not none?
                } else {
                    validate_target(token, data, sc, Scopes::non_primitive());

                    if tooltipped {
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
        }

        "faith_hostility_level" => {
            sc.expect(Scopes::Faith, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Faith);
            }
        }

        "faith_hostility_level_comparison" => {
            sc.expect(Scopes::Faith, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_faith_hostility_level_comparison(block, data, sc);
            }
        }

        "global_variable_list_size"
        | "list_size"
        | "local_variable_list_size"
        | "variable_list_size" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_list_size(block, data, sc);
            }
        }

        "government_allows" | "government_disallows" | "government_has_flag" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "has_all_innovations" => {
            sc.expect(Scopes::Culture, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_all_innovations(block, data, sc);
            }
        }

        "has_building_with_flag" => {
            sc.expect(Scopes::Province, key);
            match bv {
                BlockOrValue::Block(block) => {
                    validate_trigger_has_building_with_flag(block, data, sc);
                }
                BlockOrValue::Token(token) => {
                    data.verify_exists(Item::BuildingFlag, token);
                }
            }
        }

        "has_cb_on" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_cb_on(block, data, sc);
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
            sc.expect(Scopes::Character, key);
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
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_dread_level_towards(block, data, sc);
            }
        }

        "has_election_vote_of" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_election_vote_of(block, data, sc);
            }
        }

        "has_focus" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "has_gene" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_gene(block, data, sc);
            }
        }

        "has_government" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "has_hook_of_type" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_hook_of_type(block, data, sc);
            }
        }

        "has_memory_category" | "has_memory_type" => {
            sc.expect(Scopes::CharacterMemory, key);
            bv.expect_value();
        }

        "has_nickname" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "has_opinion_modifier" | "reverse_has_opinion_modifier" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_opinion_modifier(block, data, sc);
            }
        }

        "has_order_of_succession" => {
            sc.expect(Scopes::LandedTitle, key);
            // The only known value for this is "election"
            bv.expect_value();
        }

        "has_perk" | "has_realm_law" | "has_realm_law_flag" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "has_relation_flag" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_relation_flag(block, data, sc);
            }
        }

        "has_sexuality" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "has_trait_rank" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_trait_rank(block, data, sc);
            }
        }

        "important_action_is_valid_but_invisible"
        | "important_action_is_visible"
        | "in_activity_type" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "is_character_interaction_potentially_accepted"
        | "is_character_interaction_shown"
        | "is_character_interaction_valid" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_character_interaction(block, data, sc);
            }
        }

        "is_connected_to" => {
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_connected_to(block, data, sc);
            }
        }

        "is_council_task_valid" | "is_in_prison_type" | "is_performing_council_task" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "is_in_family" => {
            sc.expect(Scopes::Religion, key);
            bv.expect_value();
        }

        "is_scheming_against" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_scheming_against(block, data, sc);
            }
        }

        "is_target_in_global_variable_list"
        | "is_target_in_local_variable_list"
        | "is_target_in_variable_list" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_target_in_list(block, data, sc);
            }
        }

        "join_faction_chance" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Faction);
            }
        }

        "join_scheme_chance" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_join_scheme_chance(block, data, sc);
            }
        }

        "knows_language" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "max_number_maa_soldiers_of_base_type"
        | "number_maa_regiments_of_base_type"
        | "number_maa_soldiers_of_base_type" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_value(block, data, "type", Item::MenAtArmsBase, sc);
            }
        }

        "max_number_maa_soldiers_of_type"
        | "number_maa_regiments_of_type"
        | "number_maa_soldiers_of_type" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_value(block, data, "type", Item::MenAtArms, sc);
            }
        }

        "morph_gene_attribute" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_morph_gene_attribute(block, data, sc);
            }
        }

        "morph_gene_value" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_morph_gene_value(block, data, sc);
            }
        }

        "number_of_commander_traits_in_common"
        | "number_of_opposing_personality_traits"
        | "number_of_opposing_traits"
        | "number_of_personality_traits_in_common"
        | "number_of_traits_in_common" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "number_of_election_votes" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_number_of_election_votes(block, data, sc);
            }
        }

        "number_of_sinful_traits_in_common" | "number_of_virtue_traits_in_common" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "opinion" | "reverse_opinion" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "perks_in_tree" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_perks_in_tree(block, data, sc);
            }
        }

        "place_in_line_of_succession" => {
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "player_heir_position" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "realm_to_title_distance_squared" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_realm_to_title_distance_squared(block, data, sc);
            }
        }

        "save_temporary_opinion_value_as" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_save_temporary_opinion_value_as(block, data, sc);
            }
        }

        "save_temporary_scope_as" => {
            bv.expect_value();
        }

        "save_temporary_scope_value_as" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_save_temporary_scope_value_as(block, data, sc);
            }
        }

        "squared_distance" => {
            sc.expect(Scopes::LandedTitle | Scopes::Province, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(
                    block,
                    data,
                    sc,
                    Scopes::LandedTitle | Scopes::Province,
                );
            }
        }

        "time_to_hook_expiry" | "trait_compatibility" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "title_join_faction_chance" => {
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_type_value(block, data, "faction", Item::Faction, sc);
            }
        }

        "recent_history" => {
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_recent_history(block, data, sc);
            }
        }

        "tier_difference" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "time_in_prison" | "time_in_prison_type" | "time_since_death" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_days_weeks_months_years(block, data, sc);
            }
        }

        "time_of_year" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_time_of_year(block, data, sc);
            }
        }

        "title_create_faction_type_chance" => {
            sc.expect(Scopes::LandedTitle, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_create_faction_type_chance(block, data, sc);
            }
        }

        "vassal_contract_has_flag"
        | "vassal_contract_obligation_level_can_be_decreased"
        | "vassal_contract_obligation_level_can_be_increased" => {
            sc.expect(Scopes::Character, key);
            bv.expect_value();
        }

        "war_contribution" => {
            sc.expect(Scopes::War, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, sc, Scopes::Character);
            }
        }

        "yields_alliance" => {
            sc.expect(Scopes::Character, key);
            if let Some(block) = bv.expect_block() {
                validate_trigger_yields_alliance(block, data, sc);
            }
        }

        _ => {
            return false;
        }
    }
    true
}

fn validate_trigger_target_value(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    outscopes: Scopes,
) {
    let mut vd = Validator::new(block, data);

    vd.req_field("target");
    vd.req_field("value");
    if let Some(token) = vd.field_value("target") {
        validate_target(token, data, sc, outscopes);
    }
    if let Some(bv) = vd.field_any_cmp("value") {
        ScriptValue::validate_bv(bv, data, sc);
    }
}

fn validate_trigger_type_value(
    block: &Block,
    data: &Everything,
    field: &'static str,
    itype: Item,
    sc: &mut ScopeContext,
) {
    let mut vd = Validator::new(block, data);

    vd.req_field(field);
    vd.req_field("value");
    vd.field_value_item(field, itype);
    if let Some(bv) = vd.field_any_cmp("value") {
        ScriptValue::validate_bv(bv, data, sc);
    }
}

fn validate_trigger_type_target(
    block: &Block,
    data: &Everything,
    itype: Item,
    sc: &mut ScopeContext,
    outscopes: Scopes,
) {
    let mut vd = Validator::new(block, data);

    vd.req_field("type");
    vd.req_field("target");
    vd.field_value_item("type", itype);
    if let Some(token) = vd.field_value("target") {
        validate_target(token, data, sc, outscopes);
    }
}

fn validate_trigger_recent_history(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_has_all_innovations(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_faith_hostility_level_comparison(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_has_building_with_flag(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_list_size(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_is_target_in_list(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_save_temporary_scope_value_as(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_save_temporary_opinion_value_as(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_time_of_year(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_ai_diplomacy_stance(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_amenity_level(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_aptitude(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_can_add_hook(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_can_declare_war(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_can_join_or_create_faction_against(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_create_faction_type_chance(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_skill_diff(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_dread_modified_ai_boldness(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_has_cb_on(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_has_dread_level_towards(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_has_election_vote_of(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_number_of_election_votes(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_has_gene(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_has_hook_of_type(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_has_opinion_modifier(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_has_relation_flag(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_has_trait_rank(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_is_character_interaction(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_is_connected_to(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_is_scheming_against(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_join_scheme_chance(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_morph_gene_attribute(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_morph_gene_value(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_perks_in_tree(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

fn validate_trigger_realm_to_title_distance_squared(
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
) {
    // TODO
}

fn validate_trigger_yields_alliance(_block: &Block, _data: &Everything, _sc: &mut ScopeContext) {
    // TODO
}

// LAST UPDATED VERSION 1.7.0
const WARN_AGAINST_EQ: &[&str] = &[
    "gold",
    "prestige",
    "piety",
    "dynasty_prestige",
    "title_held_years",
    "years_as_ruler",
    "culture_age",
    "ghw_war_chest_gold",
    "ghw_war_chest_piety",
    "ghw_war_chest_prestige",
    "available_loot",
    "long_term_gold",
    "long_term_gold_maximum",
    "reserved_gold",
    "reserved_gold_maximum",
    "short_term_gold",
    "short_term_gold_maximum",
    "war_chest_gold",
    "war_chest_gold_maximum",
    "yearly_character_balance",
    "yearly_character_expenses",
    "yearly_character_income",
    "inspiration_gold_invested",
    "monthly_character_income",
    "monthly_character_income_long_term",
    "monthly_character_income_reserved",
    "monthly_character_income_short_term",
    "monthly_character_income_war_chest",
    "monthly_income",
    "num_total_troops",
];
