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
use crate::validate::{validate_days_months_years, validate_prefix_reference};

pub fn validate_trigger(
    block: &Block,
    data: &Everything,
    mut scopes: Scopes,
    ignore_keys: &[&str],
) -> Scopes {
    let mut seen_if = false;

    'outer: for (key, cmp, bv) in block.iter_items() {
        if let Some(key) = key {
            if ignore_keys.contains(&key.as_str()) {
                continue;
            }
            if key.is("trigger_if") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger_if(block, data, scopes);
                }
                seen_if = true;
                continue;
            } else if key.is("trigger_else_if") {
                if !seen_if {
                    error(key, ErrorKey::Validation, "must follow `trigger_if`");
                }
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger_if(block, data, scopes);
                }
                continue;
            } else if key.is("trigger_else") {
                if !seen_if {
                    error(key, ErrorKey::Validation, "must follow `trigger_if`");
                }
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(block, data, scopes, &[]);
                }
                seen_if = false;
                continue;
            }
            seen_if = false;

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

            if key.is("custom_description") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_custom_description(block, data, scopes);
                }
                continue;
            }

            if key.is("custom_tooltip") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_custom_tooltip(block, data, scopes);
                }
                continue;
            }

            if key.is("calc_true_if") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_calc_true_if(block, data, scopes);
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
            } else if (name == "in_list" || name == "in_global_list" || name == "in_local_list")
                && (key.is("list") || key.is("variable"))
            {
                bv.expect_value();
                ignore.push(key.as_str());
            } else if name == "relation" && key.is("type") {
                if let Some(token) = bv.expect_value() {
                    data.relations.verify_exists(token);
                }
                ignore.push("type");
            } else if name == "pool_character" && key.is("province") {
                if let Some(token) = bv.expect_value() {
                    (scopes, _) = validate_target(token, data, scopes, Scopes::Province);
                }
                ignore.push("province");
            } else if name == "court_position_holder" && key.is("type") {
                bv.expect_value();
                ignore.push("type");
            } else if scopes == Scopes::Character && key.is("even_if_dead") {
                bv.expect_value();
                ignore.push("even_if_dead");
            }
        }
    }
    validate_trigger(block, data, scopes, &ignore);
}

pub fn validate_character_trigger(block: &Block, data: &Everything) {
    validate_trigger(block, data, Scopes::Character, &[]);
}

fn validate_trigger_if(block: &Block, data: &Everything, mut scopes: Scopes) -> Scopes {
    for (key, _, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("limit") {
                if let Some(block) = bv.expect_block() {
                    scopes = validate_trigger(block, data, scopes, &[]);
                }
            }
        }
    }
    validate_trigger(block, data, scopes, &["limit"])
}

fn validate_custom_description(block: &Block, data: &Everything, mut scopes: Scopes) -> Scopes {
    for (key, _, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("text") {
                // TODO: validate trigger_localization
                bv.expect_value();
            } else if key.is("subject") || key.is("object") {
                if let Some(token) = bv.expect_value() {
                    (scopes, _) = validate_target(token, data, scopes, Scopes::non_primitive());
                }
            }
        }
    }
    validate_trigger(block, data, scopes, &["text", "subject", "object"])
}

fn validate_custom_tooltip(block: &Block, data: &Everything, mut scopes: Scopes) -> Scopes {
    for (key, _, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("text") {
                if let Some(token) = bv.expect_value() {
                    data.localization.verify_exists(token);
                }
            } else if key.is("subject") {
                if let Some(token) = bv.expect_value() {
                    (scopes, _) = validate_target(token, data, scopes, Scopes::non_primitive());
                }
            }
        }
    }
    validate_trigger(block, data, scopes, &["text", "subject"])
}

fn validate_calc_true_if(block: &Block, data: &Everything, scopes: Scopes) -> Scopes {
    for (key, _, bv) in block.iter_items() {
        if let Some(key) = key {
            if key.is("amount") {
                if let Some(token) = bv.expect_value() {
                    if token.as_str().parse::<i32>().is_err() {
                        warn(token, ErrorKey::Validation, "expected a number");
                    }
                }
            }
        }
    }
    validate_trigger(block, data, scopes, &["amount"])
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
    return (scopes, part_scopes);
}

/// Validate the keys that don't follow a consistent pattern in what they require from their
/// block or value.
/// Returns true iff the key was recognized (and handled)
/// LAST UPDATED VERSION 1.6.2.2
/// See `triggers.log` for details
fn validate_trigger_keys(
    key: &Token,
    bv: &BlockOrValue,
    data: &Everything,
    mut scopes: Scopes,
) -> (Scopes, bool) {
    let match_key: &str = &key.as_str().to_lowercase();
    match match_key {
        "has_house_modifier" | "has_house_modifier_duration_remaining" => {
            scopes.expect_scope(key, Scopes::DynastyHouse);
            bv.expect_value();
        }

        "faction_is_type" => {
            scopes.expect_scope(key, Scopes::Faction);
            bv.expect_value();
        }

        "using_cb" => {
            scopes.expect_scope(key, Scopes::War);
            bv.expect_value();
        }

        "war_contribution" => {
            scopes.expect_scope(key, Scopes::War);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "has_maa_of_type" => {
            scopes.expect_scope(key, Scopes::CombatSide);
            bv.expect_value();
        }

        "artifact_slot_type"
        | "category"
        | "has_artifact_feature"
        | "has_artifact_feature_group"
        | "has_artifact_modifier"
        | "rarity" => {
            scopes.expect_scope(key, Scopes::Artifact);
            bv.expect_value();
        }

        "can_title_create_faction" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_create_faction(block, data, scopes);
            }
        }

        "county_opinion_target" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "de_jure_drift_progress" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::LandedTitle);
            }
        }

        "has_county_modifier"
        | "has_county_modifier_duration_remaining"
        | "has_holy_site_flag"
        | "has_order_of_succession"
        | "has_title_law"
        | "has_title_law_flag"
        | "is_target_of_council_task" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            bv.expect_value();
        }

        "place_in_line_of_succession" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "recent_history" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_recent_history(block, data, scopes);
            }
        }

        "title_create_faction_type_chance" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_create_faction_type_chance(block, data, scopes);
            }
        }

        "title_join_faction_chance" => {
            scopes.expect_scope(key, Scopes::LandedTitle);
            if let Some(block) = bv.expect_block() {
                validate_trigger_join_faction_chance(block, data, scopes);
            }
        }

        "join_faction_chance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_join_faction_chance(block, data, scopes);
            }
        }

        "cultural_acceptance" => {
            scopes.expect_scope(key, Scopes::Culture);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Culture);
            }
        }

        "culture_overlaps_geographical_region" => {
            scopes.expect_scope(key, Scopes::Culture);
            bv.expect_value();
        }

        "has_all_innovations" => {
            scopes.expect_scope(key, Scopes::Culture);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_all_innovations(block, data, scopes);
            }
        }

        "has_building_gfx"
        | "has_clothing_gfx"
        | "has_coa_gfx"
        | "has_cultural_era_or_later"
        | "has_cultural_parameter"
        | "has_cultural_pillar"
        | "has_cultural_tradition"
        | "has_innovation"
        | "has_innovation_flag"
        | "has_name_list"
        | "has_primary_name_list"
        | "has_unit_gfx" => {
            scopes.expect_scope(key, Scopes::Culture);
            bv.expect_value();
        }

        "story_type" => {
            scopes.expect_scope(key, Scopes::StoryCycle);
            bv.expect_value();
        }

        "controls_holy_site" | "controls_holy_site_with_flag" => {
            scopes.expect_scope(key, Scopes::Faith);
            bv.expect_value();
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

        "has_doctrine" | "has_doctrine_parameter" | "has_graphical_faith" | "has_icon" => {
            scopes.expect_scope(key, Scopes::Faith);
            bv.expect_value();
        }

        "religion_tag" => {
            scopes.expect_scope(key, Scopes::Faith);
            if let Some(token) = bv.expect_value() {
                data.religions.verify_religion_exists(token);
            }
        }

        "trait_is_sin" | "trait_is_virtue" => {
            scopes.expect_scope(key, Scopes::Faith);
            if let Some(token) = bv.expect_value() {
                data.traits.verify_exists(token);
            }
        }

        "geographical_region" | "has_building" | "has_building_or_higher" => {
            scopes.expect_scope(key, Scopes::Province);
            bv.expect_value();
        }

        "has_building_with_flag" => {
            scopes.expect_scope(key, Scopes::Province);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_building_with_flag(block, data, scopes);
            }
        }

        "has_construction_with_flag"
        | "has_holding_type"
        | "has_province_modifier"
        | "has_province_modifier_duration_remaining"
        | "terrain" => {
            scopes.expect_scope(key, Scopes::Province);
            bv.expect_value();
        }

        "has_struggle_phase_parameter"
        | "is_struggle_phase"
        | "is_struggle_type"
        | "phase_has_catalyst" => {
            scopes.expect_scope(key, Scopes::Struggle);
            bv.expect_value();
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

        "has_scheme_modifier" | "scheme_skill" | "scheme_type" => {
            scopes.expect_scope(key, Scopes::Scheme);
            bv.expect_value();
        }

        "has_inspiration_type" => {
            scopes.expect_scope(key, Scopes::Inspiration);
            bv.expect_value();
        }

        "secret_type" => {
            scopes.expect_scope(key, Scopes::Secret);
            bv.expect_value();
        }

        "has_dynasty_modifier" | "has_dynasty_modifier_duration_remaining" | "has_dynasty_perk" => {
            scopes.expect_scope(key, Scopes::Dynasty);
            bv.expect_value();
        }

        "add_to_temporary_list" => {
            // TODO: if inside an any_ iterator, this should be at the end.
            bv.expect_value();
        }

        "and" | "or" | "not" | "nor" | "nand" | "all_false" | "any_false" => {
            if let Some(block) = bv.expect_block() {
                scopes = validate_trigger(block, data, scopes, &[]);
            }
        }

        "can_start_tutorial_lesson" => {
            bv.expect_value();
        }

        "current_computer_date" | "current_date" | "game_start_date" => {
            if let Some(token) = bv.expect_value() {
                if Date::try_from(token).is_err() {
                    error(token, ErrorKey::Validation, "expected date");
                }
            }
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

        "global_variable_list_size"
        | "list_size"
        | "local_variable_list_size"
        | "variable_list_size" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_list_size(block, data, scopes);
            }
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
        | "is_in_list" => {
            bv.expect_value();
        }

        "is_target_in_global_variable_list"
        | "is_target_in_local_variable_list"
        | "is_target_in_variable_list" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_target_in_list(block, data, scopes);
            }
        }

        "is_tooltip_with_name_open"
        | "is_tutorial_lesson_active"
        | "is_tutorial_lesson_chain_completed"
        | "is_tutorial_lesson_completed"
        | "is_tutorial_lesson_step_completed"
        | "is_war_overview_tab_open"
        | "is_widget_open" => {
            bv.expect_value();
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

        "time_of_year" => {
            if let Some(block) = bv.expect_block() {
                validate_trigger_time_of_year(block, data, scopes);
            }
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

        "all_court_artifact_slots" | "all_inventory_artifact_slots" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
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

        "can_add_hook" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_add_hook(block, data, scopes);
            }
        }

        "can_be_employed_as"
        | "can_employ_court_position_type"
        | "employs_court_position"
        | "is_court_position_employer" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "can_create_faction" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_create_faction(block, data, scopes);
            }
        }

        "can_declare_war" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_can_declare_war(block, data, scopes);
            }
        }

        "can_execute_decision" | "is_decision_on_cooldown" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(token) = bv.expect_value() {
                data.decisions.verify_exists(token);
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
                validate_trigger_can_start_scheme(block, data, scopes);
            }
        }

        "completely_controls_region" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "create_faction_type_chance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_create_faction_type_chance(block, data, scopes);
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

        "government_allows" | "government_disallows" | "government_has_flag" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_cb_on" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_cb_on(block, data, scopes);
            }
        }

        "has_character_flag"
        | "has_character_modifier"
        | "has_character_modifier_duration_remaining"
        | "has_council_position"
        | "has_councillor_for_skill"
        | "has_court_language"
        | "has_court_position"
        | "has_court_type" => {
            scopes.expect_scope(key, Scopes::Character);
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

        "has_lifestyle" | "has_nickname" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "has_trait" | "has_inactive_trait" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(token) = bv.expect_value() {
                data.traits.verify_exists(token);
            }
        }

        "has_opinion_modifier" | "reverse_has_opinion_modifier" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_has_opinion_modifier(block, data, scopes);
            }
        }

        "has_opposite_relation" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(token) = bv.expect_value() {
                data.relations.verify_exists(token);
            }
        }

        "has_pending_interaction_of_type" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(token) = bv.expect_value() {
                data.interactions.verify_exists(token);
            }
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

        "has_trait_with_flag"
        | "important_action_is_valid_but_invisible"
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

        "is_council_task_valid"
        | "is_in_prison_type"
        | "is_leading_faction_type"
        | "is_performing_council_task" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "is_scheming_against" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_is_scheming_against(block, data, scopes);
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

        "number_maa_regiments_of_base_type" | "number_maa_soldiers_of_base_type" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_number_maa_of_base_type(block, data, scopes);
            }
        }

        "number_maa_regiments_of_type" | "number_maa_soldiers_of_type" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_number_maa_of_type(block, data, scopes);
            }
        }

        "number_of_election_votes" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_number_of_election_votes(block, data, scopes);
            }
        }

        "number_of_opposing_personality_traits"
        | "number_of_opposing_traits"
        | "number_of_personality_traits_in_common"
        | "number_of_traits_in_common" => {
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

        "owns_story_of_type" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
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

        "time_to_hook_expiry" | "trait_compatibility" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_target_value(block, data, scopes, Scopes::Character);
            }
        }

        "vassal_contract_has_flag" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
        }

        "yields_alliance" => {
            scopes.expect_scope(key, Scopes::Character);
            if let Some(block) = bv.expect_block() {
                validate_trigger_yields_alliance(block, data, scopes);
            }
        }

        "is_in_family" => {
            scopes.expect_scope(key, Scopes::Character);
            bv.expect_value();
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
