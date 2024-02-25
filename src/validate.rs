//! Validation functions that are useful for more than one data module.

use std::fmt::{Display, Formatter};

use crate::block::{Block, BV};
#[cfg(feature = "ck3")]
use crate::ck3::validate::{
    validate_activity_modifier, validate_ai_value_modifier, validate_compare_modifier,
    validate_compatibility_modifier, validate_opinion_modifier, validate_scheme_modifier,
};
use crate::context::ScopeContext;
use crate::data::scripted_modifiers::ScriptedModifier;
use crate::everything::Everything;
use crate::game::Game;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, fatal, report, warn, Confidence, ErrorKey, Severity};
#[cfg(feature = "ck3")]
use crate::scopes::Scopes;
use crate::scopes::{scope_prefix, scope_to_scope};
use crate::script_value::{validate_non_dynamic_script_value, validate_script_value};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
#[cfg(feature = "ck3")]
use crate::trigger::validate_target_ok_this;
use crate::trigger::{
    partition, validate_argument, validate_argument_scope, validate_inscopes, validate_trigger,
    validate_trigger_internal, warn_not_first, Part, PartFlags,
};
use crate::validator::Validator;

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
            _ => Err(std::fmt::Error),
        }
    }
}

pub fn validate_compare_duration(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        if let Some(bv) = vd.field_any_cmp(field) {
            validate_script_value(bv, data, sc);
            count += 1;
        }
    }

    if count != 1 {
        let msg = "must have 1 of days, weeks, months, or years";
        let key = if count == 0 { ErrorKey::FieldMissing } else { ErrorKey::Validation };
        err(key).msg(msg).loc(block).push();
    }
}

// Very similar to validate_days_weeks_months_years, but requires = instead of allowing comparators
// "weeks" is not documented but is used all over vanilla TODO: verify
pub fn validate_mandatory_duration(block: &Block, vd: &mut Validator, sc: &mut ScopeContext) {
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        if vd.field_script_value(field, sc) {
            count += 1;
        }
    }

    if count != 1 {
        let msg = "must have 1 of days, weeks, months, or years";
        let key = if count == 0 { ErrorKey::FieldMissing } else { ErrorKey::Validation };
        err(key).msg(msg).loc(block).push();
    }
}

pub fn validate_duration(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    validate_mandatory_duration(block, &mut vd, sc);
}

// Very similar to validate_duration, but validates part of a block that may contain a duration
// Also does not accept script values (per the documentation)
#[cfg(feature = "ck3")]
pub fn validate_optional_duration_int(vd: &mut Validator) {
    let mut count = 0;

    for field in &["days", "weeks", "months", "years"] {
        vd.field_validated_value(field, |key, mut vd| {
            vd.integer();
            count += 1;
            if count > 1 {
                let msg = "must have at most 1 of days, weeks, months, or years";
                err(ErrorKey::Validation).msg(msg).loc(key).push();
            }
        });
    }
}

// Very similar to validate_days_weeks_months_years, but requires = instead of allowing comparators
pub fn validate_optional_duration(vd: &mut Validator, sc: &mut ScopeContext) {
    let mut count = 0;

    #[cfg(not(feature = "imperator"))]
    let options = &["days", "weeks", "months", "years"];

    // Imperator does not allow a "weeks" field and does allow a "duration" field for modifiers.
    #[cfg(feature = "imperator")]
    let options = &["days", "months", "years", "duration"];

    for field in options {
        vd.field_validated_key(field, |key, bv, data| {
            validate_script_value(bv, data, sc);
            count += 1;
            if count > 1 {
                let msg = "must have at most 1 of days, weeks, months, or years";
                err(ErrorKey::Validation).msg(msg).loc(key).push();
            }
        });
    }
}

// Does not accept script values (per the documentation)
pub fn validate_color(block: &Block, _data: &Everything) {
    // Reports in this function are `warn` level because a bad color is just an UI issue,
    // and normal confidence level because I'm not 100% sure of the color formats.
    let mut count = 0;
    // Get the color tag, as in color = hsv { 0.5 1.0 1.0 }
    let tag = block.tag.as_deref().map_or("rgb", Token::as_str);
    for item in block.iter_items() {
        if let Some(t) = item.get_value() {
            if tag == "hsv" {
                t.check_number();
                if let Some(f) = t.get_number() {
                    if !(0.0..=1.0).contains(&f) {
                        // TODO: check if integer color values actually work in hsv,
                        // then adjust the report.
                        let msg = "hsv values should be between 0.0 and 1.0";
                        let mut info = "";
                        if t.is_integer() {
                            info = "did you mean `hsv360`?";
                        }
                        warn(ErrorKey::Colors).weak().msg(msg).info(info).loc(t).push();
                    }
                } else {
                    warn(ErrorKey::Colors).msg("expected hsv value").loc(t).push();
                }
            } else if tag == "hsv360" {
                if let Some(i) = t.get_integer() {
                    if count == 0 && !(0..=360).contains(&i) {
                        let msg = "hsv360 h values should be between 0 and 360";
                        warn(ErrorKey::Colors).msg(msg).loc(t).push();
                    } else if count > 0 && !(0..=100).contains(&i) {
                        let msg = "hsv360 s and v values should be between 0 and 100";
                        warn(ErrorKey::Colors).msg(msg).loc(t).push();
                    }
                } else {
                    warn(ErrorKey::Colors).msg("expected hsv360 value").loc(t).push();
                }
            } else if let Some(i) = t.get_integer() {
                if !(0..=255).contains(&i) {
                    let msg = "color values should be between 0 and 255";
                    warn(ErrorKey::Colors).msg(msg).loc(t).push();
                }
            } else if let Some(f) = t.get_number() {
                t.check_number();
                if !(0.0..=1.0).contains(&f) {
                    let msg = "color values should be between 0.0 and 1.0";
                    warn(ErrorKey::Colors).msg(msg).loc(t).push();
                }
            } else {
                warn(ErrorKey::Colors).msg("expected color value").loc(t).push();
            }
            count += 1;
        }
    }
    if count != 3 && count != 4 {
        warn(ErrorKey::Colors).msg("expected 3 or 4 color values").loc(block).push();
    }
}

pub fn validate_possibly_named_color(bv: &BV, data: &Everything) {
    match bv {
        BV::Value(token) => data.verify_exists(Item::NamedColor, token),
        BV::Block(block) => validate_color(block, data),
    }
}

/// Check some iterator fields *before* the list scope has opened.
#[allow(unused_variables)] // `name` is only used for ck3
pub fn precheck_iterator_fields(
    ltype: ListType,
    name: &str,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    match ltype {
        ListType::Any => {
            if let Some(bv) = block.get_field("percent") {
                if let Some(token) = bv.get_value() {
                    if let Some(num) = token.get_number() {
                        token.check_number();
                        if num > 1.0 {
                            let msg = "'percent' here needs to be between 0 and 1";
                            warn(ErrorKey::Range).msg(msg).loc(token).push();
                        }
                    }
                }
                validate_script_value(bv, data, sc);
            }
            if let Some(bv) = block.get_field("count") {
                match bv {
                    BV::Value(token) if token.is("all") => (),
                    bv => validate_script_value(bv, data, sc),
                }
            };
        }
        ListType::Ordered => {
            for field in &["position", "min", "max"] {
                if let Some(bv) = block.get_field(field) {
                    validate_script_value(bv, data, sc);
                }
            }
        }
        ListType::Random | ListType::Every | ListType::None => (),
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() && name == "county_in_region" {
        for region in block.get_field_values("region") {
            if !data.item_exists(Item::Region, region.as_str()) {
                validate_target_ok_this(region, data, sc, Scopes::GeographicalRegion);
            }
        }
    }
}

/// This checks the fields that are only used in iterators.
/// It does not check "limit" because that is shared with the if/else blocks.
/// Returns true iff the iterator took care of its own tooltips
pub fn validate_iterator_fields(
    caller: &Lowercase,
    list_type: ListType,
    _data: &Everything,
    sc: &mut ScopeContext,
    vd: &mut Validator,
    tooltipped: &mut Tooltipped,
) {
    // undocumented
    if list_type == ListType::None {
        vd.ban_field("custom", || "lists");
    } else if vd.field_item("custom", Item::Localization) {
        *tooltipped = Tooltipped::No;
    }

    // undocumented
    if list_type != ListType::None && list_type != ListType::Any {
        vd.multi_field_validated_block("alternative_limit", |b, data| {
            validate_trigger(b, data, sc, *tooltipped);
        });
    } else {
        vd.ban_field("alternative_limit", || "`every_`, `ordered_`, and `random_` lists");
    }

    if list_type == ListType::Any {
        vd.field_any_cmp("percent"); // prechecked
        vd.field_any_cmp("count"); // prechecked
    } else {
        vd.ban_field("percent", || "`any_` lists");
        if caller != "while" {
            vd.ban_field("count", || "`while` and `any_` lists");
        }
    }

    if list_type == ListType::Ordered {
        vd.field_script_value("order_by", sc);
        vd.field("position"); // prechecked
        vd.field("min"); // prechecked
        vd.field("max"); // prechecked
        vd.field_bool("check_range_bounds");
    } else {
        vd.ban_field("order_by", || "`ordered_` lists");
        vd.ban_field("position", || "`ordered_` lists");
        if caller != "random_list" && caller != "duel" {
            vd.ban_field("min", || "`ordered_` lists, `random_list`, and `duel`");
            vd.ban_field("max", || "`ordered_` lists, `random_list`, and `duel`");
        }
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
#[allow(unused_variables)] // vic3 does not use `tooltipped`
pub fn validate_inside_iterator(
    name: &Lowercase,
    listtype: ListType,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    vd: &mut Validator,
    tooltipped: Tooltipped,
) {
    // Docs say that all three can take either list or variable, but global and local lists must be variable lists.
    if name == "in_list" {
        vd.req_field_one_of(&["list", "variable"]);
        if let Some(token) = vd.field_value("list") {
            sc.expect_list(token);
            sc.replace_list_entry(token.as_str(), token);
        }
    } else if name == "in_local_list" || name == "in_global_list" {
        vd.req_field("variable");
        vd.ban_field("list", || format!("{listtype}_in_list"));
    } else {
        vd.ban_field("list", || format!("{listtype}_in_list"));
        vd.ban_field("variable", || {
            format!(
                "`{listtype}_in_list`, `{listtype}_in_local_list`, or `{listtype}_in_global_list`"
            )
        });
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "in_de_facto_hierarchy" || name == "in_de_jure_hierarchy" {
            vd.field_validated_block("filter", |block, data| {
                validate_trigger(block, data, sc, tooltipped);
            });
            vd.field_validated_block("continue", |block, data| {
                validate_trigger(block, data, sc, tooltipped);
            });
        } else {
            let only_for = || {
                format!("`{listtype}_in_de_facto_hierarchy` or `{listtype}_in_de_jure_hierarchy`")
            };
            vd.ban_field("filter", only_for);
            vd.ban_field("continue", only_for);
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "county_in_region" {
            vd.req_field("region");
            vd.multi_field_value("region"); // prechecked
        } else {
            vd.ban_field("region", || format!("`{listtype}_county_in_region`"));
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "court_position_holder" {
            vd.field_item("type", Item::CourtPosition);
        } else if name == "relation" {
            if !block.has_key("type") {
                let msg = "required field `type` missing";
                let info = format!(
                    "Verified for 1.9.2: with no type, {listtype}_relation will do nothing."
                );
                err(ErrorKey::FieldMissing).strong().msg(msg).info(info).loc(block).push();
            }
            vd.multi_field_item("type", Item::Relation);
        } else {
            vd.ban_field("type", || {
                format!("`{listtype}_court_position_holder` or `{listtype}_relation`")
            });
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "claim" {
            vd.field_choice("explicit", &["yes", "no", "all"]);
            vd.field_choice("pressed", &["yes", "no", "all"]);
        } else {
            vd.ban_field("explicit", || format!("`{listtype}_claim`"));
            vd.ban_field("pressed", || format!("`{listtype}_claim`"));
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "pool_character" {
            vd.req_field("province");
            if let Some(token) = vd.field_value("province") {
                validate_target_ok_this(token, data, sc, Scopes::Province);
            }
        } else {
            vd.ban_field("province", || format!("`{listtype}_pool_character`"));
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if sc.can_be(Scopes::Character) {
            vd.field_bool("only_if_dead");
            vd.field_bool("even_if_dead");
        } else {
            vd.ban_field("only_if_dead", || "lists of characters");
            vd.ban_field("even_if_dead", || "lists of characters");
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "character_struggle" {
            vd.field_choice("involvement", &["involved", "interloper"]);
        } else {
            vd.ban_field("involvement", || format!("`{listtype}_character_struggle`"));
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
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
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "activity_phase_location"
            || name == "activity_phase_location_future"
            || name == "activity_phase_location_past"
        {
            vd.field_bool("unique");
        } else {
            let only_for =
                || format!("the `{listtype}_activity_phase_location` family of iterators");
            vd.ban_field("unique", only_for);
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "guest_subset" || name == "guest_subset_current_phase" {
            vd.field_item("name", Item::GuestSubset);
        } else {
            vd.ban_field("name", || {
                format!("`{listtype}_guest_subset` and `{listtype}_guest_subset_current_phase`")
            });
        }
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if name == "guest_subset" {
            vd.field_value("phase"); // TODO
        } else {
            vd.ban_field("phase", || format!("`{listtype}_guest_subset`"));
        }
    }

    if Game::is_ck3() {
        #[cfg(feature = "ck3")]
        if name == "trait_in_category" {
            vd.field_value("category"); // TODO
        } else {
            // Don't ban, because it's a valid trigger
        }
    }
}

pub fn validate_modifiers_with_base(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_validated("base", validate_non_dynamic_script_value);
    vd.fields_script_value("add", sc);
    vd.fields_script_value("factor", sc);
    vd.fields_script_value("min", sc);
    vd.fields_script_value("max", sc);
    validate_modifiers(&mut vd, sc);
    validate_scripted_modifier_calls(vd, data, sc);
}

pub fn validate_modifiers(vd: &mut Validator, sc: &mut ScopeContext) {
    vd.multi_field_validated_block("first_valid", |b, data| {
        let mut vd = Validator::new(b, data);
        validate_modifiers(&mut vd, sc);
    });
    vd.multi_field_validated_block("modifier", |b, data| {
        validate_trigger_internal(
            &Lowercase::new_unchecked("modifier"),
            false,
            b,
            data,
            sc,
            Tooltipped::No,
            false,
            Severity::Error,
        );
    });
    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        vd.multi_field_validated_block_sc("compare_modifier", sc, validate_compare_modifier);
        vd.multi_field_validated_block_sc("opinion_modifier", sc, validate_opinion_modifier);
        vd.multi_field_validated_block_sc("ai_value_modifier", sc, validate_ai_value_modifier);
        vd.multi_field_validated_block_sc(
            "compatibility_modifier",
            sc,
            validate_compatibility_modifier,
        );

        // These are special single-use modifiers
        vd.multi_field_validated_block_sc("scheme_modifier", sc, validate_scheme_modifier);
        vd.multi_field_validated_block_sc("activity_modifier", sc, validate_activity_modifier);
    }
}

#[cfg(feature = "vic3")]
pub fn validate_vic3_modifiers(vd: &mut Validator, sc: &mut ScopeContext) {
    vd.multi_field_validated("modifier", |bv, data| {
        validate_script_value(bv, data, sc);
    });
}

#[cfg(feature = "imperator")]
pub fn validate_imperator_modifiers(vd: &mut Validator, sc: &mut ScopeContext) {
    vd.multi_field_validated("modifier", |bv, data| {
        validate_script_value(bv, data, sc);
    });
}

pub fn validate_scripted_modifier_call(
    key: &Token,
    bv: &BV,
    modifier: &ScriptedModifier,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    match bv {
        BV::Value(token) => {
            if !modifier.macro_parms().is_empty() {
                fatal(ErrorKey::Macro).msg("expected macro arguments").loc(token).push();
            } else if !token.is("yes") {
                warn(ErrorKey::Validation).msg("expected just modifier = yes").loc(token).push();
            }
            modifier.validate_call(key, data, sc);
        }
        BV::Block(block) => {
            let parms = modifier.macro_parms();
            if parms.is_empty() {
                fatal(ErrorKey::Macro)
                    .msg("this scripted modifier does not need macro arguments")
                    .info("you can just use it as modifier = yes")
                    .loc(block)
                    .push();
            } else {
                let mut vec = Vec::new();
                let mut vd = Validator::new(block, data);
                for parm in &parms {
                    if let Some(token) = vd.field_value(parm) {
                        vec.push(token.clone());
                    } else {
                        let msg = format!("this scripted modifier needs parameter {parm}");
                        err(ErrorKey::Macro).msg(msg).loc(block).push();
                        return;
                    }
                }
                vd.unknown_value_fields(|key, _value| {
                    let msg = format!("this scripted modifier does not need parameter {key}");
                    let info = "supplying an unneeded parameter often causes a crash";
                    fatal(ErrorKey::Macro).msg(msg).info(info).loc(key).push();
                });
                let args: Vec<_> = parms.into_iter().zip(vec).collect();
                modifier.validate_macro_expansion(key, &args, data, sc);
            }
        }
    }
}

pub fn validate_scripted_modifier_calls(
    mut vd: Validator,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    vd.unknown_fields(|key, bv| {
        if let Some(modifier) = data.scripted_modifiers.get(key.as_str()) {
            validate_scripted_modifier_call(key, bv, modifier, data, sc);
        } else {
            let msg = format!("unknown field `{key}`");
            warn(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    });
}

pub fn validate_ai_chance(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BV::Value(t) => _ = t.expect_number(),
        BV::Block(b) => validate_modifiers_with_base(b, data, sc),
    }
}

/// Validate the left-hand part of a `target = { target_scope }` block.
///
/// The caller is expected to have done `sc.open_builder()` before calling and then do `sc.close()` after calling.
/// Returns true iff validation was complete.
/// `qeq` is true if the scope chain is to the left of a ?= operator.
pub fn validate_scope_chain(
    token: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    qeq: bool,
) -> bool {
    let part_vec = partition(token);
    for i in 0..part_vec.len() {
        let mut part_flags = PartFlags::empty();
        if i == 0 {
            part_flags |= PartFlags::First;
        }
        if i + 1 == part_vec.len() {
            part_flags |= PartFlags::Last;
        }
        if qeq {
            part_flags |= PartFlags::Question;
        }
        let part = &part_vec[i];

        match part {
            Part::TokenArgument(func, arg) => validate_argument(part_flags, func, arg, data, sc),
            Part::Token(part) => {
                let part_lc = Lowercase::new(part.as_str());
                // prefixed scope transition, e.g. cp:councillor_steward
                if let Some((prefix, arg)) = part.split_once(':') {
                    // known prefix
                    if let Some(entry) = scope_prefix(&prefix) {
                        validate_argument_scope(part_flags, entry, &prefix, &arg, data, sc);
                    } else {
                        let msg = format!("unknown prefix `{prefix}:`");
                        err(ErrorKey::Validation).msg(msg).loc(prefix).push();
                        return false;
                    }
                } else if part_lc == "root" {
                    sc.replace_root();
                } else if part_lc == "prev" {
                    if !part_flags.contains(PartFlags::First) && !Game::is_imperator() {
                        warn_not_first(part);
                    }
                    sc.replace_prev();
                } else if part_lc == "this" {
                    sc.replace_this();
                } else if let Some((inscopes, outscope)) = scope_to_scope(part, sc.scopes()) {
                    validate_inscopes(part_flags, part, inscopes, sc);
                    sc.replace(outscope, part.clone());
                } else {
                    let msg = format!("unknown token `{part}`");
                    err(ErrorKey::UnknownField).msg(msg).loc(part).push();
                    return false;
                }
            }
        }
    }
    true
}

pub fn validate_ifelse_sequence(block: &Block, key_if: &str, key_elseif: &str, key_else: &str) {
    let mut seen_if = false;
    for (key, block) in block.iter_definitions() {
        if key.is(key_if) {
            seen_if = true;
            continue;
        } else if key.is(key_elseif) {
            if !seen_if {
                let msg = format!("`{key_elseif} without preceding `{key_if}`");
                warn(ErrorKey::IfElse).msg(msg).loc(key).push();
            }
            seen_if = true;
            continue;
        } else if key.is(key_else) {
            if !seen_if {
                let msg = format!("`{key_else} without preceding `{key_if}`");
                warn(ErrorKey::IfElse).msg(msg).loc(key).push();
            }
            if block.has_key("limit") {
                // `else` with a `limit`, followed by another `else`, does work.
                seen_if = true;
                continue;
            }
        }
        seen_if = false;
    }
}

pub fn validate_numeric_range(
    block: &Block,
    data: &Everything,
    min: f64,
    max: f64,
    sev: Severity,
    conf: Confidence,
) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;
    let mut prev = 0.0;

    for token in vd.values() {
        if let Some(n) = token.expect_number() {
            count += 1;
            if !(min..=max).contains(&n) {
                let msg = format!("expected number between {min} and {max}");
                report(ErrorKey::Range, sev).conf(conf).msg(msg).loc(token).push();
            }
            if count == 1 {
                prev = n;
            } else if count == 2 && n < prev {
                let msg = "expected second number to be bigger than first number";
                report(ErrorKey::Range, sev).conf(conf).msg(msg).loc(token).push();
            } else if count == 3 {
                let msg = "expected exactly 2 numbers";
                report(ErrorKey::Range, sev).strong().msg(msg).loc(block).push();
            }
        }
    }
}
