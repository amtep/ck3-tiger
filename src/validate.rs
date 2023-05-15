use std::fmt::{Display, Formatter};

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
/// A module for validation functions that are useful for more than one data module.
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::error;
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

pub fn validate_theme_background(bv: &BlockOrValue, data: &Everything, sc: &mut ScopeContext) {
    if let Some(block) = bv.get_block() {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("trigger", |b, data| {
            validate_normal_trigger(b, data, sc, false)
        });
        if vd.field_value("event_background").is_some() {
            let msg = "`event_background` now causes a crash. It has been replaced by `reference` since 1.9";
            error(
                block.get_key("event_background").unwrap(),
                ErrorKey::Crash,
                &msg,
            );
        }
        vd.field_value("reference");
    } else {
        // TODO: verify the background is defined
    }
}

pub fn validate_theme_icon(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false)
    });
    // TODO: verify the file exists
    vd.field_value("reference"); // file
}

pub fn validate_theme_sound(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false)
    });
    vd.field_value("reference"); // event:/ resource reference
}

pub fn validate_theme_transition(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false)
    });
    vd.field_value("reference"); // TODO: unknown
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

// Very similar to validate_days_weeks_months_years, but requires = instead of allowing comparators
// "weeks" is not documented but is used all over vanilla TODO: verify
pub fn validate_cooldown(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    if let Some(bv) = vd.field("days") {
        ScriptValue::validate_bv(bv, data, sc);
        count += 1;
    }
    if let Some(bv) = vd.field("weeks") {
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
            "must have 1 of days, weeks, months, or years",
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
    sc: &mut ScopeContext,
    vd: &mut Validator,
    tooltipped: &mut bool,
) {
    // undocumented
    if list_type != ListType::None {
        if vd.field_value_item("custom", Item::Localization) {
            *tooltipped = false;
        }
        vd.field_validated_blocks("alternative_limit", |b, data| {
            validate_normal_trigger(b, data, sc, false);
        });
    } else {
        vd.ban_field("custom", || "lists");
        vd.ban_field("alternative_limit", || "lists");
    }

    if list_type == ListType::Ordered {
        vd.field_script_value("order_by", sc);
        vd.field_integer("position");
        vd.field_integer("min");
        vd.field_integer("max");
        vd.field_bool("check_range_bounds");
    } else {
        vd.ban_field("order_by", || "`ordered_` lists");
        vd.ban_field("position", || "`ordered_` lists");
        vd.ban_field("min", || "`ordered_` lists");
        vd.ban_field("max", || "`ordered_` lists");
        vd.ban_field("check_range_bounds", || "`ordered_` lists");
    }

    if list_type == ListType::Random {
        vd.field_block("weight"); // TODO
    } else {
        vd.ban_field("weight", || "`random_` lists");
    }
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
        let only_for = || {
            format!(
                "`{listtype}_in_list`, `{listtype}_in_local_list`, or `{listtype}_in_global_list`"
            )
        };
        vd.ban_field("list", only_for);
        vd.ban_field("variable", only_for);
    }

    if name == "in_de_facto_hierarchy" || name == "in_de_jure_hierarchy" {
        if let Some(block) = vd.field_block("filter") {
            validate_normal_trigger(block, data, sc, tooltipped);
        }
        if let Some(block) = vd.field_block("continue") {
            validate_normal_trigger(block, data, sc, tooltipped);
        }
    } else {
        let only_for =
            || format!("`{listtype}_in_de_facto_hierarchy` or `{listtype}_in_de_jure_hierarchy`");
        vd.ban_field("filter", only_for);
        vd.ban_field("continue", only_for);
    }

    if name == "county_in_region" {
        vd.req_field("region");
        vd.field_value_item("region", Item::Region);
    } else {
        vd.ban_field("region", || format!("`{listtype}_county_in_region`"));
    }

    if name == "court_position_holder" {
        vd.field_value_item("type", Item::CourtPosition);
    } else if name == "relation" {
        vd.req_field("type");
        vd.field_value_item("type", Item::Relation);
    } else {
        vd.ban_field("type", || {
            format!("`{listtype}_court_position_holder` or `{listtype}_relation`")
        });
    }

    if name == "claim" {
        vd.field_choice("explicit", &["yes", "no", "all"]);
        vd.field_choice("pressed", &["yes", "no", "all"]);
    } else {
        vd.ban_field("explicit", || format!("`{listtype}_claim`"));
        vd.ban_field("pressed", || format!("`{listtype}_claim`"));
    }

    if name == "pool_character" {
        vd.req_field("province");
        if let Some(token) = block.get_field_value("province") {
            validate_target(token, data, sc, Scopes::Province);
        }
    } else {
        vd.ban_field("province", || format!("`{listtype}_pool_character`"));
    }

    if sc.can_be(Scopes::Character) {
        vd.field_bool("only_if_dead");
        vd.field_bool("even_if_dead");
    } else {
        vd.ban_field("only_if_dead", || "lists of characters");
        vd.ban_field("even_if_dead", || "lists of characters");
    }

    if name == "character_struggle" {
        vd.field_choice("involvement", &["involved", "interloper"]);
    } else {
        vd.ban_field("involvement", || format!("`{listtype}_character_struggle`"));
    }

    if name == "connected_county" {
        // Undocumented
        vd.field_bool("invert");
        vd.field_numeric("max_naval_distance");
        vd.field_bool("allow_one_county_land_gap");
    } else {
        let only_for = || format!("`{listtype}_connected_county`");
        vd.ban_field("invert", only_for);
        vd.ban_field("max_naval_distance", only_for);
        vd.ban_field("allow_one_county_land_gap", only_for);
    }

    if name == "activity_phase_location"
        || name == "activity_phase_location_future"
        || name == "activity_phase_location_past"
    {
        vd.field_bool("unique");
    } else {
        let only_for = || format!("the `{listtype}_activity_phase_location` family of iterators");
        vd.ban_field("unique", only_for);
    }

    if name == "guest_subset" || name == "guest_subset_current_phase" {
        vd.field_value_item("name", Item::GuestSubset);
    } else {
        vd.ban_field("name", || {
            format!("`{listtype}_guest_subset` and `{listtype}_guest_subset_current_phase`")
        });
    }
    if name == "guest_subset" {
        vd.field_value("phase"); // TODO
    } else {
        vd.ban_field("phase", || format!("`{listtype}_guest_subset`"));
    }

    if name == "trait_in_category" {
        vd.field_value("category"); // TODO
    } else {
        vd.ban_field("category", || format!("`{listtype}_trait_in_category`"));
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
