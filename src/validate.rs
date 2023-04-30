use std::fmt::{Display, Formatter};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ListType {
    None,
    Any,
    Every,
    Ordered,
    Random,
}

impl Display for ListType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ListType::None => write!(f, ""),
            ListType::Any => write!(f, "any"),
            ListType::Every => write!(f, "every"),
            ListType::Ordered => write!(f, "ordered"),
            ListType::Random => write!(f, "random"),
        }
    }
}

impl TryFrom<&str> for ListType {
    type Error = std::fmt::Error;

    fn try_from(from: &str) -> Result<Self, Self::Error> {
        match from {
            "" => Ok(ListType::None),
            "any" => Ok(ListType::Any),
            "every" => Ok(ListType::Every),
            "ordered" => Ok(ListType::Ordered),
            "random" => Ok(ListType::Random),
            _ => Err(Self::Error::default()),
        }
    }
}

pub fn validate_theme_background(bv: &BlockOrValue, data: &Everything) {
    if let Some(block) = bv.get_block() {
        let mut vd = Validator::new(block, data);

        vd.field_block("trigger");
        // TODO: verify the background is defined
        vd.field_value("event_background");
        // TODO: check if `reference` actually works or is a mistake in vanilla
        vd.field_value("reference");
    } else {
        // TODO: verify the background is defined
    }
}

pub fn validate_theme_icon(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    // TODO: verify the file exists
    vd.field_value("reference"); // file
}

pub fn validate_theme_sound(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_block("trigger");
    vd.field_value("reference"); // event:/ resource reference
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
    let msg = format!("should start with `{wanted}:` here");
    error(token, ErrorKey::Validation, &msg);
}

/// This checks the fields that are only used in iterators.
/// It does not check "limit" because that is shared with the if/else blocks.
/// Returns true iff the iterator took care of its own tooltips
pub fn validate_iterator_fields(
    list_type: ListType,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    vd: &mut Validator,
) -> bool {
    let mut has_tooltip = false;
    // undocumented
    if let Some(key) = block.get_key("custom") {
        vd.field_value_item("custom", Item::Localization);
        if list_type == ListType::None {
            warn(
                key,
                ErrorKey::Validation,
                "`custom` can only be used in lists",
            );
        }
        has_tooltip = true;
    }

    vd.field_validated_blocks("alternative_limit", |b, data| {
        if list_type != ListType::None {
            validate_normal_trigger(b, data, sc, false);
        } else {
            warn(
                b,
                ErrorKey::Validation,
                "`alternative_limit` can only be used in lists",
            );
        }
    });

    if let Some(bv) = vd.field("order_by") {
        if list_type == ListType::Ordered {
            ScriptValue::validate_bv(bv, data, sc);
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
            ScriptValue::validate_bv(bv, data, sc);
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
    has_tooltip
}

/// This checks the special fields for certain iterators, like `type =` in `every_relation`.
/// It doesn't check the generic ones like `limit` or the ordering ones for `ordered_*`.
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
                "`list =` is only for `{listtype}_in_list`, `{listtype}_in_local_list`, or `{listtype}_in_global_list`",
            );
            error(token, ErrorKey::Validation, &msg);
        }
        if let Some(token) = vd.field_value("variable") {
            let msg = format!(
                "`variable =` is only for `{listtype}_in_list`, `{listtype}_in_local_list`, or `{listtype}_in_global_list`",
            );
            error(token, ErrorKey::Validation, &msg);
        }
    }

    if let Some(block) = vd.field_block("filter") {
        if name == "in_de_facto_hierarchy" || name == "in_de_jure_hierarchy" {
            validate_normal_trigger(block, data, sc, tooltipped);
        } else {
            let msg = format!(
                "`filter` is only for `{listtype}_in_de_facto_hierarchy` or `{listtype}_in_de_jure_hierarchy`",
            );
            error(block.get_key("filter").unwrap(), ErrorKey::Validation, &msg);
        }
    }
    if let Some(block) = vd.field_block("continue") {
        if name == "in_de_facto_hierarchy" || name == "in_de_jure_hierarchy" {
            validate_normal_trigger(block, data, sc, tooltipped);
        } else {
            let msg = format!(
                "`continue` is only for `{listtype}_in_de_facto_hierarchy` or `{listtype}_in_de_jure_hierarchy`",
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
        let msg = format!("`region` is only for `{listtype}_county_in_region`");
        error(token, ErrorKey::Validation, &msg);
    }

    if name == "court_position_holder" {
        vd.field_value_item("type", Item::CourtPosition);
    } else if name == "relation" {
        vd.req_field("type");
        vd.field_value_item("type", Item::Relation);
    } else if let Some(token) = block.get_key("type") {
        let msg = format!(
            "`type` is only for `{listtype}_court_position_holder` or `{listtype}_relation`",
        );
        error(token, ErrorKey::Validation, &msg);
    }

    if vd.field_choice("explicit", &["yes", "no", "all"]) {
        if name != "claim" {
            let msg = format!("`explicit` is only for `{listtype}_claim`");
            error(
                block.get_key("explicit").unwrap(),
                ErrorKey::Validation,
                &msg,
            );
        }
    }
    if vd.field_choice("pressed", &["yes", "no", "all"]) {
        if name != "claim" {
            let msg = format!("`pressed` is only for `{listtype}_claim`");
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
        let msg = format!("`province` is only for `{listtype}_pool_character`");
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

    if vd.field_choice("involvement", &["involved", "interloper"]) {
        if name != "character_struggle" {
            let msg = format!("`involvement` is only for `{listtype}_character_struggle`",);
            error(
                block.get_key("involvement").unwrap(),
                ErrorKey::Validation,
                &msg,
            );
        }
    }

    // Undocumented
    if vd.field_bool("invert") {
        if name != "connected_county" {
            let msg = format!("`invert` is only for `{listtype}_connected_county`");
            error(block.get_key("invert").unwrap(), ErrorKey::Validation, &msg);
        }
    }
    if vd.field_numeric("max_naval_distance") {
        if name != "connected_county" {
            let msg = format!("`max_naval_distance` is only for `{listtype}_connected_county`",);
            error(
                block.get_key("max_naval_distance").unwrap(),
                ErrorKey::Validation,
                &msg,
            );
        }
    }
    if vd.field_bool("allow_one_county_land_gap") {
        if name != "connected_county" {
            let msg =
                format!("`allow_one_county_land_gap` is only for `{listtype}_connected_county`",);
            error(
                block.get_key("allow_one_county_land_gap").unwrap(),
                ErrorKey::Validation,
                &msg,
            );
        }
    }
}

pub fn validate_cost(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    // These can all be script values
    vd.field_validated_bv("gold", |bv, data| {
        ScriptValue::validate_bv(bv, data, sc);
    });
    vd.field_validated_bv("prestige", |bv, data| {
        ScriptValue::validate_bv(bv, data, sc);
    });
    vd.field_validated_bv("piety", |bv, data| {
        ScriptValue::validate_bv(bv, data, sc);
    });
    vd.field_bool("round");
}

pub fn validate_traits(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    // TODO: parse these. Can be single tokens ("wrathful") or assignments ("wrathful = 3")
    // or even wrathful = { modifier = modifier_key scale = 2 }
    vd.field_block("virtues");
    vd.field_block("sins");
}
