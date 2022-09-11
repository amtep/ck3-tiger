use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
/// A module for validation functions that are useful for more than one data module.
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::{validate_normal_trigger, validate_target};

pub fn validate_theme_background(bv: &BlockOrValue, data: &Everything) {
    if let Some(block) = bv.get_block() {
        let mut vd = Validator::new(block, data);

        vd.field_block("trigger");
        // TODO: verify the background is defined
        vd.field_value("event_background");
        // TODO: check if `reference` actually works or is a mistake in vanilla
        vd.field_value("reference");
        vd.warn_remaining();
    } else {
        // TODO: verify the background is defined
    }
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

pub fn validate_days_weeks_months_years(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    if let Some(bv) = vd.field_any_cmp("days") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }
    if let Some(bv) = vd.field_any_cmp("weeks") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }
    if let Some(bv) = vd.field_any_cmp("months") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }
    if let Some(bv) = vd.field_any_cmp("years") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }

    if count != 1 {
        error(
            block,
            ErrorKey::Validation,
            "must have 1 of days, weeks, months, or years",
        );
    }

    vd.warn_remaining();
}

// Very similar to validate_years_months_days, but requires = instead of allowing comparators
pub fn validate_cooldown(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    if let Some(bv) = vd.field("days") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }
    if let Some(bv) = vd.field("months") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }
    if let Some(bv) = vd.field("years") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }

    if count != 1 {
        error(
            block,
            ErrorKey::Validation,
            "must have 1 of days, months, or years",
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

pub fn validate_prefix_reference(prefix: &Token, arg: &Token, data: &Everything) {
    // TODO there are more to match
    match prefix.as_str() {
        "character" => data.verify_exists(Item::Character, arg),
        "dynasty" => data.verify_exists(Item::Dynasty, arg),
        "event_id" => data.verify_exists(Item::Event, arg),
        "faith" => data.verify_exists(Item::Faith, arg),
        "house" => data.verify_exists(Item::House, arg),
        "province" => data.verify_exists(Item::Province, arg),
        "religion" => data.verify_exists(Item::Religion, arg),
        "title" => data.verify_exists(Item::Title, arg),
        &_ => (),
    }
}

pub fn validate_prefix_reference_token(token: &Token, data: &Everything, wanted: &str) {
    if let Some((prefix, arg)) = token.split_once(':') {
        validate_prefix_reference(&prefix, &arg, data);
        if prefix.is(wanted) {
            return;
        }
    }
    let msg = format!("should start with `{}:` here", wanted);
    error(token, ErrorKey::Validation, &msg);
}

// This checks the special fields for certain iterators, like type = in every_relation.
// It doesn't check the generic ones like "limit" or the ordering ones for ordered_*.
pub fn validate_inside_iterator(
    name: &str,
    listtype: &str,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    vd: &mut Validator,
    tooltipped: bool,
) {
    if name == "in_list" || name == "in_local_list" || name == "in_global_list" {
        let have_list = vd.field_value("list").is_some();
        let have_var = vd.field_value("variable").is_some();
        if have_list == have_var {
            error(
                block,
                ErrorKey::Validation,
                "must have one of `list =` or `variable =`",
            );
        }
    } else {
        if let Some(token) = vd.field_value("list") {
            let msg = format!(
                "`list =` is only for `{}_in_list`, `{}_in_local_list`, or `{}_in_global_list`",
                listtype, listtype, listtype
            );
            error(token, ErrorKey::Validation, &msg);
        }
        if let Some(token) = vd.field_value("variable") {
            let msg = format!(
                "`variable =` is only for `{}_in_list`, `{}_in_local_list`, or `{}_in_global_list`",
                listtype, listtype, listtype
            );
            error(token, ErrorKey::Validation, &msg);
        }
    }

    if let Some(block) = vd.field_block("filter") {
        if name == "in_de_facto_hierarchy" || name == "in_de_jure_hierarchy" {
            validate_normal_trigger(block, data, sc, tooltipped);
        } else {
            let msg = format!(
                "`filter` is only for `{}_in_de_facto_hierarchy` or `{}_in_de_jure_hierarchy`",
                listtype, listtype
            );
            error(block.get_key("filter").unwrap(), ErrorKey::Validation, &msg);
        }
    }
    if let Some(block) = vd.field_block("continue") {
        if name == "in_de_facto_hierarchy" || name == "in_de_jure_hierarchy" {
            validate_normal_trigger(block, data, sc, tooltipped);
        } else {
            let msg = format!(
                "`continue` is only for `{}_in_de_facto_hierarchy` or `{}_in_de_jure_hierarchy`",
                listtype, listtype
            );
            error(
                block.get_key("continue").unwrap(),
                ErrorKey::Validation,
                &msg,
            );
        }
    }

    if name == "county_in_region" {
        vd.req_field("region");
        vd.field_value_item("region", Item::Region);
    } else if let Some(token) = block.get_key("region") {
        let msg = format!("`region` is only for `{}_county_in_region`", listtype);
        error(token, ErrorKey::Validation, &msg);
    }

    if name == "court_position_holder" {
        vd.field_value_item("type", Item::CourtPosition);
    } else if name == "relation" {
        vd.req_field("type");
        vd.field_value_item("type", Item::Relation);
    } else if let Some(token) = block.get_key("type") {
        let msg = format!(
            "`type` is only for `{}_court_position_holder` or `{}_relation`",
            listtype, listtype
        );
        error(token, ErrorKey::Validation, &msg);
    }

    if vd.field_choice("explicit", &["yes", "no", "all"]) {
        if name != "claim" {
            let msg = format!("`explicit` is only for `{}_claim`", listtype);
            error(
                block.get_key("explicit").unwrap(),
                ErrorKey::Validation,
                &msg,
            );
        }
    }
    if vd.field_choice("pressed", &["yes", "no", "all"]) {
        if name != "claim" {
            let msg = format!("`pressed` is only for `{}_claim`", listtype);
            error(
                block.get_key("pressed").unwrap(),
                ErrorKey::Validation,
                &msg,
            );
        }
    }

    if name == "pool_character" {
        vd.req_field("province");
        if let Some(token) = block.get_field_value("province") {
            validate_target(token, data, sc, Scopes::Province);
        }
    } else if let Some(token) = block.get_key("province") {
        let msg = format!("`province` is only for `{}_pool_character`", listtype);
        error(token, ErrorKey::Validation, &msg);
    }

    if vd.field_bool("only_if_dead") {
        if !sc.can_be(Scopes::Character) {
            warn(
                block.get_key("only_if_dead").unwrap(),
                ErrorKey::Validation,
                "`only_if_dead` is only for lists of characters",
            );
        }
    }
    if vd.field_bool("even_if_dead") {
        if !sc.can_be(Scopes::Character) {
            warn(
                block.get_key("even_if_dead").unwrap(),
                ErrorKey::Validation,
                "`even_if_dead` is only for lists of characters",
            );
        }
    }
}
