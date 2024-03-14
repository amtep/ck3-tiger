//! Validate effects, which are parts of the script that change the game state.

use crate::block::{Block, Comparator, Eq::*, BV};
use crate::context::{Reason, ScopeContext};
use crate::data::effect_localization::EffectLocalization;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::Game;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, fatal, tips, warn, ErrorKey};
use crate::scopes::{scope_iterator, Scopes};
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
#[cfg(any(feature = "ck3", feature = "imperator"))]
use crate::trigger::validate_target_ok_this;
use crate::trigger::{validate_target, validate_trigger};
#[cfg(not(feature = "imperator"))]
use crate::validate::validate_compare_duration;
#[cfg(any(feature = "ck3", feature = "imperator"))]
use crate::validate::validate_modifiers;
#[cfg(feature = "vic3")]
use crate::validate::validate_vic3_modifiers;
use crate::validate::{
    precheck_iterator_fields, validate_ifelse_sequence, validate_inside_iterator,
    validate_iterator_fields, validate_scope_chain, validate_scripted_modifier_call, ListType,
};
use crate::validator::{Validator, ValueValidator};

/// The standard interface to effect validation. Validates an effect in the given [`ScopeContext`].
///
/// `tooltipped` determines what warnings are emitted related to tooltippability of the effects
/// inside the block.
pub fn validate_effect(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let vd = Validator::new(block, data);
    validate_effect_internal(Lowercase::empty(), ListType::None, block, data, sc, vd, tooltipped);
}

/// The interface to effect validation when [`validate_effect`] is too limited.
///
/// `caller` is the key that opened this effect. It is used to determine which special cases apply.
/// For example, if `caller` is `if` then a `limit` block is expected.
///
/// `list_type` specifies whether this effect is directly in an iterator of the specified
/// [`ListType`]. It is also used to determine which special cases apply. The `list_type` must not
/// be `ListType::Any` because that is only used for triggers.
///
/// `vd` is a `Validator` that may have already been checked for some fields by the caller. This is
/// because some effects are in contexts where other keys than just the effects are valid.
pub fn validate_effect_internal<'a>(
    caller: &Lowercase,
    list_type: ListType,
    block: &Block,
    data: &'a Everything,
    sc: &mut ScopeContext,
    mut vd: Validator<'a>,
    mut tooltipped: Tooltipped,
) {
    if caller == "if"
        || caller == "else_if"
        || caller == "else"
        || caller == "while"
        || list_type != ListType::None
    {
        vd.field_validated_key_block("limit", |key, block, data| {
            if caller == "else" {
                let msg = "`else` with a `limit` does work, but may indicate a mistake";
                let info = "normally you would use `else_if` instead.";
                tips(ErrorKey::IfElse).msg(msg).info(info).loc(key).push();
            }
            validate_trigger(block, data, sc, tooltipped);
        });
    } else {
        vd.ban_field("limit", || "if/else_if or lists");
    }

    #[allow(clippy::if_not_else)] // for symmetry
    if list_type != ListType::None {
        vd.field_validated_block("filter", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("filter", || "lists");
    }

    validate_iterator_fields(caller, list_type, data, sc, &mut vd, &mut tooltipped, false);

    if list_type != ListType::None {
        validate_inside_iterator(caller, list_type, block, data, sc, &mut vd, tooltipped);
    }

    validate_ifelse_sequence(block, "if", "else_if", "else");

    vd.set_allow_questionmark_equals(true);
    vd.unknown_fields_cmp(|key, cmp, bv| {
        validate_effect_field(caller, key, cmp, bv, data, sc, tooltipped);
    });
}

/// Validate a single effect field
pub fn validate_effect_field(
    caller: &Lowercase,
    key: &Token,
    cmp: Comparator,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    if let Some(effect) = data.get_effect(key) {
        match bv {
            BV::Value(token) => {
                if !effect.macro_parms().is_empty() {
                    fatal(ErrorKey::Macro).msg("expected macro arguments").loc(token).push();
                } else if !token.is("yes") {
                    warn(ErrorKey::Validation).msg("expected just effect = yes").loc(token).push();
                }
                effect.validate_call(key, data, sc, tooltipped);
            }
            BV::Block(block) => {
                let parms = effect.macro_parms();
                if parms.is_empty() {
                    err(ErrorKey::Macro)
                        .msg("this scripted effect does not need macro arguments")
                        .info("you can just use it as effect = yes")
                        .loc(block)
                        .push();
                } else {
                    let mut vec = Vec::new();
                    let mut vd = Validator::new(block, data);
                    for parm in &parms {
                        if let Some(token) = vd.field_value(parm) {
                            vec.push(token.clone());
                        } else {
                            let msg = format!("this scripted effect needs parameter {parm}");
                            err(ErrorKey::Macro).msg(msg).loc(block).push();
                            return;
                        }
                    }
                    vd.unknown_value_fields(|key, _value| {
                        let msg = format!("this scripted effect does not need parameter {key}");
                        let info = "supplying an unneeded parameter often causes a crash";
                        fatal(ErrorKey::Macro).msg(msg).info(info).loc(key).push();
                    });
                    let args: Vec<_> = parms.into_iter().zip(vec).collect();
                    effect.validate_macro_expansion(key, &args, data, sc, tooltipped);
                }
            }
        }
        return;
    }

    if let Some(modifier) = data.scripted_modifiers.get(key.as_str()) {
        if caller != "random" && caller != "random_list" && caller != "duel" {
            let msg = "cannot use scripted modifier here";
            err(ErrorKey::Validation).msg(msg).loc(key).push();
            return;
        }
        validate_scripted_modifier_call(key, bv, modifier, data, sc);
        return;
    }

    let scope_effect = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::effects::scope_effect,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::effects::scope_effect,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::effects::scope_effect,
    };

    if let Some((inscopes, effect)) = scope_effect(key, data) {
        sc.expect(inscopes, &Reason::Token(key.clone()));
        match effect {
            Effect::Yes => {
                if let Some(token) = bv.expect_value() {
                    if !token.is("yes") {
                        let msg = format!("expected just `{key} = yes`");
                        warn(ErrorKey::Validation).msg(msg).loc(token).push();
                    }
                }
            }
            Effect::Boolean => {
                if let Some(token) = bv.expect_value() {
                    validate_target(token, data, sc, Scopes::Bool);
                }
            }
            Effect::Integer => {
                if let Some(token) = bv.expect_value() {
                    token.expect_integer();
                }
            }
            Effect::ScriptValue | Effect::NonNegativeValue => {
                if let Some(token) = bv.get_value() {
                    if let Some(number) = token.get_number() {
                        if matches!(effect, Effect::NonNegativeValue) && number < 0.0 {
                            if key.is("add_gold") {
                                let msg = "add_gold does not take negative numbers";
                                let info = "try remove_short_term_gold instead";
                                warn(ErrorKey::Range).msg(msg).info(info).loc(token).push();
                            } else {
                                let msg = format!("{key} does not take negative numbers");
                                warn(ErrorKey::Range).msg(msg).loc(token).push();
                            }
                        }
                    }
                }
                validate_script_value(bv, data, sc);
            }
            #[cfg(feature = "vic3")]
            Effect::Date => {
                if let Some(token) = bv.expect_value() {
                    token.expect_date();
                }
            }
            Effect::Scope(outscopes) => {
                if let Some(token) = bv.expect_value() {
                    validate_target(token, data, sc, outscopes);
                }
            }
            #[cfg(any(feature = "ck3", feature = "imperator"))]
            Effect::ScopeOkThis(outscopes) => {
                if let Some(token) = bv.expect_value() {
                    validate_target_ok_this(token, data, sc, outscopes);
                }
            }
            Effect::Item(itype) => {
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(itype, token);
                }
            }
            Effect::ScopeOrItem(outscopes, itype) => {
                if let Some(token) = bv.expect_value() {
                    if !data.item_exists(itype, token.as_str()) {
                        validate_target(token, data, sc, outscopes);
                    }
                }
            }
            #[cfg(feature = "ck3")]
            Effect::Target(key, outscopes) => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_case_sensitive(false);
                    vd.req_field(key);
                    vd.field_target(key, sc, outscopes);
                }
            }
            #[cfg(not(feature = "imperator"))]
            Effect::TargetValue(key, outscopes, valuekey) => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_case_sensitive(false);
                    vd.req_field(key);
                    vd.req_field(valuekey);
                    vd.field_target(key, sc, outscopes);
                    vd.field_script_value(valuekey, sc);
                }
            }
            #[cfg(feature = "ck3")]
            Effect::ItemTarget(ikey, itype, tkey, outscopes) => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_case_sensitive(false);
                    vd.field_item(ikey, itype);
                    vd.field_target(tkey, sc, outscopes);
                }
            }
            #[cfg(feature = "ck3")]
            Effect::ItemValue(key, itype) => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_case_sensitive(false);
                    vd.req_field(key);
                    vd.req_field("value");
                    vd.field_item(key, itype);
                    vd.field_script_value("value", sc);
                }
            }
            Effect::Choice(choices) => {
                if let Some(token) = bv.expect_value() {
                    if !choices.contains(&token.as_str()) {
                        let msg = format!("expected one of {}", choices.join(", "));
                        err(ErrorKey::Choice).msg(msg).loc(token).push();
                    }
                }
            }
            #[cfg(feature = "ck3")]
            Effect::Desc => validate_desc(bv, data, sc),
            #[cfg(not(feature = "imperator"))]
            Effect::Timespan => {
                if let Some(block) = bv.expect_block() {
                    validate_compare_duration(block, data, sc);
                }
            }
            Effect::Vb(f) => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.set_case_sensitive(false);
                    f(key, block, data, sc, vd, tooltipped);
                }
            }
            Effect::Vv(f) => {
                if let Some(token) = bv.expect_value() {
                    let vd = ValueValidator::new(token, data);
                    f(key, vd, sc, tooltipped);
                }
            }
            Effect::Vbv(f) => {
                f(key, bv, data, sc, tooltipped);
            }
            Effect::ControlOrLabel => match bv {
                BV::Value(t) => {
                    data.verify_exists(Item::Localization, t);
                    data.validate_localization_sc(t.as_str(), sc);
                }
                BV::Block(b) => {
                    validate_effect_control(&Lowercase::new(key.as_str()), b, data, sc, tooltipped);
                }
            },
            Effect::Control => {
                if let Some(block) = bv.expect_block() {
                    validate_effect_control(
                        &Lowercase::new(key.as_str()),
                        block,
                        data,
                        sc,
                        tooltipped,
                    );
                }
            }
            Effect::Removed(version, explanation) => {
                let msg = format!("`{key}` was removed in {version}");
                warn(ErrorKey::Removed).msg(msg).info(explanation).loc(key).push();
            }
            Effect::Unchecked => (),
            #[cfg(not(feature = "imperator"))]
            Effect::UncheckedTodo => (),
        }
        return;
    }

    if let Some((it_type, it_name)) = key.split_once('_') {
        if it_type.is("any") || it_type.is("ordered") || it_type.is("every") || it_type.is("random")
        {
            if let Some((inscopes, outscope)) = scope_iterator(&it_name, data, sc) {
                if it_type.is("any") {
                    let msg = "cannot use `any_` lists in an effect";
                    err(ErrorKey::Validation).msg(msg).loc(key).push();
                    return;
                }
                sc.expect(inscopes, &Reason::Token(key.clone()));
                let ltype = ListType::try_from(it_type.as_str()).unwrap();
                if let Some(b) = bv.expect_block() {
                    precheck_iterator_fields(ltype, it_name.as_str(), b, data, sc);
                }
                sc.open_scope(outscope, key.clone());
                if let Some(b) = bv.get_block() {
                    let vd = Validator::new(b, data);
                    validate_effect_internal(
                        &Lowercase::new(it_name.as_str()),
                        ltype,
                        b,
                        data,
                        sc,
                        vd,
                        tooltipped,
                    );
                }
                sc.close();
                return;
            }
        }
    }

    // Check if it's a target = { target_scope } block.
    sc.open_builder();
    if validate_scope_chain(key, data, sc, matches!(cmp, Comparator::Equals(Question))) {
        sc.finalize_builder();
        if key.starts_with("flag:") {
            let msg = "as of 1.9, flag literals cannot be used on the left-hand side";
            err(ErrorKey::Scopes).msg(msg).loc(key).push();
        }
        if let Some(block) = bv.expect_block() {
            validate_effect(block, data, sc, tooltipped);
        }
    }
    sc.close();
}

/// Validate an effect that has other effects inside its block.
pub fn validate_effect_control(
    caller: &Lowercase,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);

    if caller == "if" || caller == "else_if" {
        vd.req_field_warn("limit");
    }

    if caller == "custom_description"
        || caller == "custom_description_no_bullet"
        || caller == "custom_tooltip"
        || caller == "custom_label"
    {
        vd.req_field("text");
        if caller == "custom_tooltip" || caller == "custom_label" {
            vd.field_item("text", Item::Localization);
            if let Some(value) = block.get_field_value("text") {
                data.validate_localization_sc(value.as_str(), sc);
            }
        } else if let Some(token) = vd.field_value("text") {
            data.verify_exists(Item::EffectLocalization, token);
            if let Some((key, block)) = data.get_key_block(Item::EffectLocalization, token.as_str())
            {
                EffectLocalization::validate_use(key, block, data, token, tooltipped);
            }
        }
        vd.field_target_ok_this("subject", sc, Scopes::non_primitive());
        tooltipped = Tooltipped::No;
    } else {
        vd.ban_field("text", || "`custom_description` or `custom_tooltip`");
        vd.ban_field("subject", || "`custom_description` or `custom_tooltip`");
    }

    if caller == "custom_description" || caller == "custom_description_no_bullet" {
        vd.field_target_ok_this("object", sc, Scopes::non_primitive());
        vd.field_script_value("value", sc);
    } else {
        vd.ban_field("object", || "`custom_description`");
        vd.ban_field("value", || "`custom_description`");
    }

    if caller == "hidden_effect" || caller == "hidden_effect_new_object" {
        tooltipped = Tooltipped::No;
    }

    if caller == "random" {
        vd.req_field("chance");
        vd.field_script_value("chance", sc);
    } else {
        vd.ban_field("chance", || "`random`");
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() && (caller == "send_interface_message" || caller == "send_interface_toast") {
        vd.field_item("type", Item::Message);
        vd.field_validated_sc("title", sc, validate_desc);
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_validated_sc("tooltip", sc, validate_desc);
        let icon_scopes =
            Scopes::Character | Scopes::LandedTitle | Scopes::Artifact | Scopes::Faith;
        if let Some(token) = vd.field_value("left_icon") {
            validate_target_ok_this(token, data, sc, icon_scopes);
        }
        if let Some(token) = vd.field_value("right_icon") {
            validate_target_ok_this(token, data, sc, icon_scopes);
        }
        if let Some(token) = vd.field_value("goto") {
            let msg = "`goto` was removed from interface messages in 1.9";
            warn(ErrorKey::Removed).msg(msg).loc(token).push();
        }
        // These seem to be scopes to set for the message loca.
        vd.field_validated_block("localization_values", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|_key, value| {
                validate_target_ok_this(value, data, sc, Scopes::all());
            });
        });
    }

    if caller == "while" {
        if !(block.has_key("limit") || block.has_key("count")) {
            let msg = "`while` needs one of `limit` or `count`";
            warn(ErrorKey::Validation).msg(msg).loc(block).push();
        }

        vd.field_script_value("count", sc);
    } else {
        vd.ban_field("count", || "`while` and `any_` lists");
    }

    if caller == "random" || caller == "random_list" || caller == "duel" {
        #[cfg(feature = "vic3")]
        if Game::is_vic3() {
            validate_vic3_modifiers(&mut vd, sc);
        }
        #[cfg(feature = "imperator")]
        if Game::is_imperator() {
            validate_modifiers(&mut vd, sc);
        }
        #[cfg(feature = "ck3")]
        if Game::is_ck3() {
            validate_modifiers(&mut vd, sc);
        }
    } else {
        vd.ban_field("modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("compare_modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("opinion_modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("ai_value_modifier", || "`random`, `random_list` or `duel`");
        vd.ban_field("compatibility", || "`random`, `random_list` or `duel`");
    }

    if caller == "random_list" || caller == "duel" {
        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_bool("show_chance");
        vd.field_validated_sc("desc", sc, validate_desc);
        vd.field_script_value("min", sc); // used in vanilla
        vd.field_script_value("max", sc); // used in vanilla
    } else {
        vd.ban_field("trigger", || "`random_list` or `duel`");
        vd.ban_field("show_chance", || "`random_list` or `duel`");
    }

    validate_effect_internal(caller, ListType::None, block, data, sc, vd, tooltipped);
}

/// This `enum` describes what arguments an effect takes, so that they can be validated.
///
/// Since effects are so varied, many of them end up as special cases described by the `VB`, `VBv`,
/// and `VV` variants. Common patterns can be captured here though.
///
/// TODO: adding a "Block" syntax similar to that in triggers may be helpful. It could remove some
/// of the variants that currently have very few users, and it could remove some of the special
/// cases.
#[derive(Copy, Clone)]
pub enum Effect {
    /// No special value, just `effect = yes`.
    Yes,
    /// Yes and no are both meaningful. The difference between this and [`Effect::Yes`] can be hard
    /// to distinguish.
    /// TODO: needs testing.
    Boolean,
    /// The effect takes a literal integer. It's not clear whether effects of this type actually
    /// exist or if they're all secrectly [`Effect::ScriptValue`].
    /// TODO: needs testing.
    Integer,
    /// The effect takes a script value, which can be a literal number or a named script value or an
    /// inline script value block.
    ScriptValue,
    /// Just like [`Effect::ScriptValue`], but warns if the argument is a negative literal number.
    #[allow(dead_code)]
    NonNegativeValue,
    /// The effect takes a literal date.
    #[cfg(feature = "vic3")]
    Date,
    /// The effect takes a target value that must evaluate to a scope type in the given [`Scopes`] value.
    ///
    /// * Example: `set_county_culture = root.culture`
    Scope(Scopes),
    /// Just like [`Effect::Scope`] but it doesn't warn if the target is a literal `this`. The
    /// default behavior for targets is to warn about that, because it's usually a mistake.
    ///
    /// * Example: `destroy_artifact = this`
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    ScopeOkThis(Scopes),
    /// The effect takes a literal string that must exist in the item database for the given [`Item`] type.
    ///
    /// * Example: `add_perk = iron_constitution_perk`
    Item(Item),
    /// A combination of [`Effect::Scope`] and [`Effect::Item`]. The argument is first checked to see
    /// if it's a literal [`Item`], and if not, it's evaluated as a target. This can sometimes
    /// cause strange error messages if the argument was intended to be an item but just doesn't exist.
    ///
    /// * Example: `add_trait = cannibal`
    /// * Example: `add_trait = scope:learned_trait`
    ScopeOrItem(Scopes, Item),
    /// The effect takes a block that contains a single field, named here, which is a target that
    /// must evaluate to a scope type in the given [`Scopes`] value.
    ///
    /// * Only example: `becomes_independent = { change = scope:change }`
    #[cfg(feature = "ck3")]
    Target(&'static str, Scopes),
    /// The effect takes a block with two fields, both named here, where one specifies a target of
    /// the given [`Scopes`] type and the other specifies a script value.
    ///
    /// * Example: `change_de_jure_drift_progress = { target = root.primary_title value = 5 }`
    #[cfg(not(feature = "imperator"))]
    TargetValue(&'static str, Scopes, &'static str),
    /// The effect takes a block with two fields, both named here, where one specifies a key for
    /// the given [`Item`] type and the other specifies a target of the given [`Scopes`] type.
    ///
    /// * Example: `remove_hook = { type = indebted_hook target = scope:old_caliph }`
    #[cfg(feature = "ck3")]
    ItemTarget(&'static str, Item, &'static str, Scopes),
    /// The effect takes a block with two fields, both named here, where one specifies a key for
    /// the given [`Item`] type and the other specifies a script value.
    ///
    /// * Example: `set_amenity_level = { type = court_food_quality value = 3 }`
    #[cfg(feature = "ck3")]
    ItemValue(&'static str, Item),
    /// The effect takes either a localization key or a description block with `first_valid` etc.
    ///
    /// * Example: `set_artifact_name = relic_weapon_name`
    #[cfg(feature = "ck3")]
    Desc,
    /// The effect takes a duration, with a `days`, `weeks`, `months`, or `years` script value.
    ///
    /// * Example: `add_destination_progress = { days = 5 }`
    #[cfg(not(feature = "imperator"))]
    Timespan,
    /// The effect takes a block that contains other effects.
    ///
    /// * Examples: `if`, `while`, `custom_description`
    Control,
    /// The effect takes either a localization key, or a block that contains other effects.
    /// This variant is used by `custom_tooltip`.
    ControlOrLabel,
    /// This variant is for effects that can take any argument and it's not validated.
    /// The effect is too unusual, or not worth checking, or really any argument is fine.
    ///
    /// * Examples: `assert_if`, `debug_log`, `remove_variable`
    Unchecked,
    /// This variant is for effects that we haven't gotten around to validating yet.
    #[cfg(not(feature = "imperator"))]
    UncheckedTodo,
    /// The effect takes a literal string that is one of the options given here.
    ///
    /// * Example: `end_war = white_peace`
    Choice(&'static [&'static str]),
    /// The effect is no longer valid; warn if it's still being used.
    /// The first string is the game version number where it was removed and the second string is
    /// an explanation that suggests a different effect to try. The second string may be empty.
    Removed(&'static str, &'static str),
    /// The effect takes a block that will be validated by this function
    Vb(fn(&Token, &Block, &Everything, &mut ScopeContext, Validator, Tooltipped)),
    /// The effect takes a block or value that will be validated by this function
    Vbv(fn(&Token, &BV, &Everything, &mut ScopeContext, Tooltipped)),
    /// The effect takes a value that will be validated by this function
    Vv(fn(&Token, ValueValidator, &mut ScopeContext, Tooltipped)),
}
