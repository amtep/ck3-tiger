use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
/// A module for validation functions that are useful for more than one data module.
use crate::context::ScopeContext;
use crate::data::scripted_modifiers::ScriptedModifier;
use crate::data::scriptvalues::ScriptValue;
use crate::desc::validate_desc;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::{validate_normal_trigger, validate_target, validate_trigger};

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
            validate_normal_trigger(b, data, sc, false);
        });
        if vd.field_value("event_background").is_some() {
            let msg = "`event_background` now causes a crash. It has been replaced by `reference` since 1.9";
            error(
                block.get_key("event_background").unwrap(),
                ErrorKey::Crash,
                msg,
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
        validate_normal_trigger(b, data, sc, false);
    });
    // TODO: verify the file exists
    vd.field_value("reference"); // file
}

pub fn validate_theme_sound(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false);
    });
    vd.field_value("reference"); // event:/ resource reference
}

pub fn validate_theme_transition(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("trigger", |b, data| {
        validate_normal_trigger(b, data, sc, false);
    });
    vd.field_value("reference"); // TODO: unknown
}

pub fn validate_days_weeks_months_years(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        if let Some(bv) = vd.field_any_cmp(field) {
            ScriptValue::validate_bv(bv, data, sc);
            count += 1;
        }
    }

    if count != 1 {
        let msg = "must have 1 of days, weeks, months, or years";
        error(block, ErrorKey::Validation, msg);
    }
}

// Very similar to validate_days_weeks_months_years, but requires = instead of allowing comparators
// "weeks" is not documented but is used all over vanilla TODO: verify
pub fn validate_cooldown(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        if vd.field_script_value(field, sc) {
            count += 1;
        }
    }

    if count != 1 {
        let msg = "must have 1 of days, weeks, months, or years";
        error(block, ErrorKey::Validation, msg);
    }
}

// Very similar to validate_cooldown, but validates part of a block that may contain a cooldown
// Also does not accept scriptvalues (per the documentation)
pub fn validate_optional_cooldown_int(vd: &mut Validator) {
    let mut count = 0;
    let mut found = None;

    for field in &["days", "weeks", "months", "years"] {
        if vd.field_integer(field) {
            count += 1;
            found = Some(field);
        }
    }

    if count > 1 {
        let msg = "must have at most 1 of days, weeks, months, or years";
        error(vd.key(found.unwrap()).unwrap(), ErrorKey::Validation, msg);
    }
}

// Very similar to validate_days_weeks_months_years, but requires = instead of allowing comparators
pub fn validate_optional_cooldown(vd: &mut Validator, sc: &mut ScopeContext) {
    let mut count = 0;
    let mut found = None;

    for field in &["days", "weeks", "months", "years"] {
        if vd.field_script_value(field, sc) {
            count += 1;
            found = Some(field);
        }
    }

    if count > 1 {
        let msg = "must have at most 1 of days, weeks, months, or years";
        error(vd.key(found.unwrap()).unwrap(), ErrorKey::Validation, msg);
    }
}

// Very similar to validate_cooldown, but validates part of a block that may contain a color
// Also does not accept scriptvalues (per the documentation)
pub fn validate_color(block: &Block, _data: &Everything) {
    let mut count = 0;
    for (k, _, v) in block.iter_items() {
        if let Some(key) = k {
            error(key, ErrorKey::Validation, "expected color value");
        } else {
            match v {
                BlockOrValue::Value(t) => {
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
    // TODO integrate this to the SCOPE_FROM_PREFIX table
    match prefix.as_str() {
        "accolade_type" => data.verify_exists(Item::AccoladeType, arg),
        "activity_type" => data.verify_exists(Item::ActivityType, arg),
        "character" => data.verify_exists(Item::Character, arg),
        "council_task" => data.verify_exists(Item::CouncilTask, arg),
        "aptitude" | "court_position" => data.verify_exists(Item::CourtPosition, arg),
        "cp" => data.verify_exists(Item::CouncilPosition, arg),
        "culture" => data.verify_exists(Item::Culture, arg),
        "culture_pillar" => data.verify_exists(Item::CulturePillar, arg),
        "culture_tradition" => data.verify_exists(Item::CultureTradition, arg),
        "decision" => data.verify_exists(Item::Decision, arg),
        "array_define" | "define" => data.verify_exists(Item::Define, arg),
        "doctrine" => data.verify_exists(Item::Doctrine, arg),
        "dynasty" => data.verify_exists(Item::Dynasty, arg),
        "event_id" => data.verify_exists(Item::Event, arg),
        "faith" => data.verify_exists(Item::Faith, arg),
        "government_type" => data.verify_exists(Item::GovernmentType, arg),
        "house" => data.verify_exists(Item::House, arg),
        "mandate_type_qualification" => data.verify_exists(Item::DiarchyMandate, arg),
        "province" => data.verify_exists(Item::Province, arg),
        "religion" => data.verify_exists(Item::Religion, arg),
        "struggle" => data.verify_exists(Item::Struggle, arg),
        "title" => data.verify_exists(Item::Title, arg),
        "trait" => data.verify_exists(Item::Trait, arg),
        "vassal_contract" => data.verify_exists(Item::VassalObligation, arg),
        "vassal_contract_obligation_level" => data.verify_exists(Item::VassalObligationLevel, arg),
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
    caller: &str,
    list_type: ListType,
    sc: &mut ScopeContext,
    vd: &mut Validator,
    tooltipped: &mut bool,
) {
    // undocumented
    if list_type != ListType::None {
        if vd.field_item("custom", Item::Localization) {
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
        if caller != "random_list" && caller != "duel" {
            vd.ban_field("min", || "`ordered_` lists, `random_list`, and `duel`");
        }
        vd.ban_field("max", || "`ordered_` lists");
        vd.ban_field("check_range_bounds", || "`ordered_` lists");
    }

    if list_type == ListType::Random {
        vd.field_validated_block_sc("weight", sc, validate_modifiers_with_base);
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
            let msg = "must have one of `list =` or `variable =`";
            error(block, ErrorKey::Validation, msg);
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
        vd.field_item("region", Item::Region);
    } else {
        vd.ban_field("region", || format!("`{listtype}_county_in_region`"));
    }

    if name == "court_position_holder" {
        vd.field_item("type", Item::CourtPosition);
    } else if name == "relation" {
        vd.req_field("type");
        for t in vd.field_values("type") {
            data.verify_exists(Item::Relation, t);
        }
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
        vd.field_item("name", Item::GuestSubset);
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
    vd.field_script_value("gold", sc);
    vd.field_script_value("prestige", sc);
    vd.field_script_value("piety", sc);
    vd.field_bool("round");
}

pub fn validate_traits(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    // TODO: parse these. Can be single tokens ("wrathful") or assignments ("wrathful = 3")
    // or even wrathful = { modifier = modifier_key scale = 2 }
    vd.field_block("virtues");
    vd.field_block("sins");
}

pub fn validate_compare_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_target("target", sc, Scopes::Character);
    // I guess that "value" and "factor" are run for both the current scope character
    // and the target character, and then compared.
    vd.field_script_value("value", sc);
    vd.field_script_value("factor", sc);
    vd.fields_script_value("multiplier", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_script_value("step", sc); // What does this do?
    vd.field_script_value("offset", sc); // What does this do?
    vd.field_validated_sc("desc", sc, validate_desc);
    if let Some(b) = vd.field_block("trigger") {
        validate_normal_trigger(b, data, sc, false);
    }
}

pub fn validate_opinion_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target(target, data, sc, Scopes::Character);
    }
    vd.req_field("opinion_target");
    if let Some(target) = vd.field_value("opinion_target") {
        validate_target(target, data, sc, Scopes::Character);
    }
    vd.field_script_value("multiplier", sc);
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    vd.field_script_value("step", sc); // What does this do?
    if let Some(b) = vd.field_block("trigger") {
        validate_normal_trigger(b, data, sc, false);
    }
}

pub fn validate_ai_value_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target(target, data, sc, Scopes::Character);
    }
    vd.field_script_value("ai_boldness", sc);
    vd.field_script_value("ai_compassion", sc);
    vd.field_script_value("ai_energy", sc);
    vd.field_script_value("ai_greed", sc);
    vd.field_script_value("ai_honor", sc);
    vd.field_script_value("ai_rationality", sc);
    vd.field_script_value("ai_sociability", sc);
    vd.field_script_value("ai_vengefulness", sc);
    vd.field_script_value("ai_zeal", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    if let Some(b) = vd.field_block("trigger") {
        validate_normal_trigger(b, data, sc, false);
    }
}

pub fn validate_compatibility_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    if let Some(target) = vd.field_value("who") {
        validate_target(target, data, sc, Scopes::Character);
    }
    if let Some(target) = vd.field_value("compatibility_target") {
        validate_target(target, data, sc, Scopes::Character);
    }
    vd.field_script_value("multiplier", sc);
    //vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
    if let Some(b) = vd.field_block("trigger") {
        validate_normal_trigger(b, data, sc, false);
    }
}

pub fn validate_modifiers_with_base(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_script_value("base", sc);
    vd.field_script_value("add", sc);
    vd.field_script_value("factor", sc);
    validate_modifiers(&mut vd, block, data, sc);
    validate_scripted_modifier_calls(vd, data, sc);
}

pub fn validate_modifiers(
    vd: &mut Validator,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
) {
    vd.field_validated_blocks("first_valid", |b, data| {
        let mut vd = Validator::new(b, data);
        validate_modifiers(&mut vd, block, data, sc);
    });
    vd.field_validated_blocks("modifier", |b, data| {
        validate_trigger("modifier", false, b, data, sc, false);
    });
    vd.field_validated_blocks_sc("compare_modifier", sc, validate_compare_modifier);
    vd.field_validated_blocks_sc("opinion_modifier", sc, validate_opinion_modifier);
    vd.field_validated_blocks_sc("ai_value_modifier", sc, validate_ai_value_modifier);
    vd.field_validated_blocks_sc(
        "compatibility_modifier",
        sc,
        validate_compatibility_modifier,
    );
}

pub fn validate_scripted_modifier_call(
    key: &Token,
    bv: &BlockOrValue,
    modifier: &ScriptedModifier,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    match bv {
        BlockOrValue::Value(token) => {
            if !modifier.macro_parms().is_empty() {
                error(token, ErrorKey::Macro, "expected macro arguments");
            } else if !token.is("yes") {
                warn(token, ErrorKey::Validation, "expected just modifier = yes");
            }
            modifier.validate_call(key, data, sc);
        }
        BlockOrValue::Block(block) => {
            let parms = modifier.macro_parms();
            if parms.is_empty() {
                error_info(
                    block,
                    ErrorKey::Macro,
                    "modifier does not need macro arguments",
                    "you can just use it as modifier = yes",
                );
            } else {
                let mut vec = Vec::new();
                let mut vd = Validator::new(block, data);
                for parm in &parms {
                    vd.req_field(parm.as_str());
                    if let Some(token) = vd.field_value(parm.as_str()) {
                        vec.push(token.clone());
                    } else {
                        return;
                    }
                }
                let args = parms.into_iter().zip(vec.into_iter()).collect();
                modifier.validate_macro_expansion(key, args, data, sc);
            }
        }
    }
}

pub fn validate_scripted_modifier_calls(
    mut vd: Validator,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    for (key, bv) in vd.unknown_keys() {
        if let Some(modifier) = data.scripted_modifiers.get(key.as_str()) {
            validate_scripted_modifier_call(key, bv, modifier, data, sc);
        } else {
            let msg = "unknown field `{key}`";
            warn(key, ErrorKey::Validation, msg);
        }
    }
}

pub fn validate_ai_chance(bv: &BlockOrValue, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BlockOrValue::Value(t) => {
            if f64::from_str(t.as_str()).is_err() {
                let msg = "expected number";
                warn(t, ErrorKey::Validation, msg);
            }
        }
        BlockOrValue::Block(b) => validate_modifiers_with_base(b, data, sc),
    }
}
