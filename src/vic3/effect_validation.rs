use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target;
use crate::validate::{validate_color, validate_optional_duration};
use crate::validator::{Validator, ValueValidator};
use crate::vic3::data::buildings::BuildingType;
use crate::vic3::tables::misc::LOBBY_FORMATION_REASON;

pub fn validate_activate_production_method(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("building_type");
    vd.req_field("production_method");
    vd.field_item("building_type", Item::BuildingType);
    // TODO: check that the production method belongs to the building type
    vd.field_item("production_method", Item::ProductionMethod);
}

pub fn validate_add_culture_sol_modifier(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("culture");
    vd.field_target("culture", sc, Scopes::Culture);
    validate_optional_duration(&mut vd, sc);
    vd.field_script_value("multiplier", sc); // seems to be actually an adder
}

pub fn validate_add_religion_sol_modifier(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("religion");
    vd.field_target("religion", sc, Scopes::Religion);
    validate_optional_duration(&mut vd, sc);
    vd.field_script_value("multiplier", sc); // seems to be actually an adder
}

pub fn validate_add_enactment_modifier(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_item("name", Item::Modifier);
    vd.field_script_value("multiplier", sc);
}

pub fn validate_add_modifier(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(value) => {
            data.verify_exists(Item::Modifier, value);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("name");
            vd.field_item("name", Item::Modifier);
            vd.field_script_value("multiplier", sc);
            validate_optional_duration(&mut vd, sc);
            vd.field_bool("is_decaying");
        }
    }
}

pub fn validate_add_journalentry(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_item("type", Item::JournalEntry);
    vd.field_item("objective_subgoal", Item::ObjectiveSubgoal); // undocumented
    vd.field_target("target", sc, Scopes::all());
}

pub fn validate_add_loyalists(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("value");
    vd.field_script_value("value", sc);
    vd.field_item_or_target("interest_group", sc, Item::InterestGroup, Scopes::InterestGroup);
    vd.field_item_or_target("pop_type", sc, Item::PopType, Scopes::PopType);
    vd.field_choice("strata", &["poor", "middle", "rich"]);
    vd.field_item_or_target("culture", sc, Item::Culture, Scopes::Culture);
    vd.field_item_or_target("religion", sc, Item::Religion, Scopes::Religion);
}

pub fn validate_add_technology_progress(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("progress");
    vd.field_numeric("progress");
    vd.req_field("technology");
    vd.field_item("technology", Item::Technology);
}

pub fn validate_add_war_goal(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("holder");
    vd.field_item_or_target("holder", sc, Item::Country, Scopes::Country);
    vd.req_field("type");
    vd.field_item("type", Item::Wargoal);
    vd.field_target("state", sc, Scopes::State);
    // TODO: verify this; there's only one example in vanilla
    vd.advice_field("country", "docs say `country` but it's `target_country`");
    vd.field_target("target_country", sc, Scopes::Country);
    vd.field_target("target_state", sc, Scopes::State);
    vd.field_target("region", sc, Scopes::StateRegion);
    vd.field_bool("primary_demand");
}

pub fn validate_remove_war_goal(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("who");
    vd.field_item_or_target("who", sc, Item::Country, Scopes::Country);
    vd.req_field("war_goal");
    vd.field_item("war_goal", Item::Wargoal);
}

pub fn validate_addremove_backers(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    for value in vd.values() {
        if !data.item_exists(Item::Country, value.as_str()) {
            validate_target(value, data, sc, Scopes::Country);
        }
    }
}

pub fn validate_call_election(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("months");
    vd.field_script_value("months", sc);
}

pub fn validate_change_institution_investment_level(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("institution");
    vd.field_item("institution", Item::Institution);
    vd.req_field("investment");
    vd.field_integer("investment");
}

pub fn validate_diplomatic_pact(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.req_field("type");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.field_item_or_target("country", sc, Item::Country, Scopes::Country);
    vd.field_item_or_target("first_state", sc, Item::StateRegion, Scopes::State);
    vd.field_item_or_target("second_state", sc, Item::StateRegion, Scopes::State);
    vd.field_item("type", Item::DiplomaticAction);
}

pub fn validate_country_value(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.req_field("value");
    vd.field_item_or_target("country", sc, Item::Country, Scopes::Country);
    vd.field_script_value("value", sc);
}

pub fn validate_create_building(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("building");
    vd.field_item("building", Item::BuildingType);
    let building = block.get_field_value("building");
    vd.field_validated_list("activate_production_methods", |token, data| {
        data.verify_exists(Item::ProductionMethod, token);
        if let Some(building) = building {
            if let Some((_, block, building_item)) =
                data.get_item::<BuildingType>(Item::BuildingType, building.as_str())
            {
                building_item.validate_production_method(token, building, block, data);
            }
        }
    });
    vd.field_bool("subsidized");
    vd.field_numeric_range("reserves", 0.0..=1.0);
    vd.field_validated_value("level", |_, mut vd| {
        vd.maybe_is("arable_land");
        vd.integer();
    });
    vd.field_validated_block("add_ownership", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.multi_field_validated_block("country", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("country");
            vd.field_target("country", sc, Scopes::Country);
            vd.req_field("levels");
            vd.field_integer("levels");
        });
        // Docs say "country" for both, but vanilla uses "building".
        vd.multi_field_validated_block("building", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("country");
            vd.field_target("country", sc, Scopes::Country);
            vd.req_field("levels");
            vd.field_integer("levels");
            vd.req_field("type");
            vd.field_item("type", Item::BuildingType);
            vd.req_field("region");
            vd.field_item("region", Item::StateRegion);
        });
    });
}

pub fn validate_create_character(
    key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_localization("name", sc);
    vd.field_localization("first_name", sc);
    vd.field_localization("last_name", sc);
    if block.has_key("name") {
        vd.ban_field("first_name", || "characters without `name`");
        vd.ban_field("last_name", || "characters without `name`");
    } else if block.has_key("first_name") {
        if !block.has_key("last_name") {
            let msg = "character has `first_name` but no `last_name`";
            warn(ErrorKey::Validation).msg(msg).loc(key).push();
        }
    } else if block.has_key("last_name") {
        let msg = "character has `last_name` but no `first_name`";
        warn(ErrorKey::Validation).msg(msg).loc(key).push();
    }
    // NOTE: docs say this is an Item, but vanilla files consistently pass a scope.
    vd.field_target("culture", sc, Scopes::Culture);
    // TODO: vanilla files pass religion as an item in several places. Figure out if that's a bug.
    vd.field_target("religion", sc, Scopes::Religion);
    vd.field_validated_value("female", |_, mut vd| {
        vd.maybe_bool();
        vd.target(sc, Scopes::Character);
    });
    vd.field_validated_value("noble", |_, mut vd| {
        vd.maybe_bool();
        vd.target(sc, Scopes::Character);
    });
    vd.field_bool("ruler");
    vd.field_bool("heir");
    vd.field_bool("historical");
    vd.field_validated("age", |bv, data| {
        match bv {
            BV::Value(value) => {
                // age = integer or character scope
                let mut vd = ValueValidator::new(value, data);
                vd.maybe_integer();
                vd.target(sc, Scopes::Character);
            }
            BV::Block(block) => {
                // age = { min max }
                let mut vd = Validator::new(block, data);
                vd.req_tokens_integers_exactly(2);
            }
        }
    });
    vd.field_item_or_target("ideology", sc, Item::Ideology, Scopes::Ideology);
    vd.field_item_or_target("interest_group", sc, Item::InterestGroup, Scopes::InterestGroup);
    vd.field_item("template", Item::CharacterTemplate);
    vd.field_validated_block("on_created", |block, data| {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        validate_effect(block, data, &mut sc, Tooltipped::No);
    });
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Character, name);
    }
    vd.field_validated_key_block("trait_generation", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        validate_effect(block, data, &mut sc, Tooltipped::No);
    });
    // The item option is undocumented
    vd.field_item_or_target("hq", sc, Item::StrategicRegion, Scopes::Hq | Scopes::StrategicRegion);

    // undocumented fields

    // TODO: not known how age and birth_date interact
    vd.field_date("birth_date");
    vd.field_list_items("traits", Item::CharacterTrait);
    vd.field_item("dna", Item::Dna);
    vd.field_bool("is_general");
    vd.field_bool("is_admiral");
    vd.field_bool("is_agitator");
    vd.field_bool("ig_leader");
    vd.field_item("commander_rank", Item::CommanderRank);
}

pub fn validate_form_government(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_script_value("value", sc);
    vd.multi_field_item("interest_group_type", Item::InterestGroup);
}

pub fn validate_set_secret_goal(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.req_field("secret_goal");
    vd.field_item_or_target("country", sc, Item::Country, Scopes::Country);
    vd.field_item("secret_goal", Item::SecretGoal);
}

pub fn validate_post_notification(
    _key: &Token,
    mut vd: ValueValidator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.item(Item::Message);
    vd.implied_localization_sc("notification_", "_name", sc);
    vd.implied_localization_sc("notification_", "_desc", sc);
    vd.implied_localization_sc("notification_", "_tooltip", sc);
}

pub fn validate_progress(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("value");
    vd.req_field("name");
    vd.field_script_value("value", sc);
    vd.field_item("name", Item::ScriptedProgressBar);
}

pub fn validate_join_war(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("side");
    vd.field_target("target", sc, Scopes::Country);
    vd.field_target("side", sc, Scopes::Country);
}

pub fn validate_create_truce(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.req_field("months");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.field_target("country", sc, Scopes::Country);
    // TODO: docs say integer, but check if script value is allowed
    vd.field_integer("months");
}

pub fn validate_create_power_bloc(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("map_color");
    vd.req_field("identity");
    // TODO: see if a full desc is allowed here. Docs just say loc key.
    vd.field_validated_sc("name", sc, validate_desc);
    // TODO: check if named colors are allowed
    vd.field_validated_block("map_color", validate_color);
    vd.field_item("identity", Item::PowerBlocIdentity);
    vd.multi_field_item("principle", Item::Principle);
    vd.multi_field_target("member", sc, Scopes::Country);

    // undocumented

    vd.field_date("founding_date");
}

pub fn validate_create_lobby(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("target");
    vd.field_item("type", Item::PoliticalLobby);
    vd.field_target("target", sc, Scopes::Country);
    vd.multi_field_target("add_interest_group", sc, Scopes::InterestGroup);
    // undocumented
    vd.field_choice("lobby_formation_reason", LOBBY_FORMATION_REASON);
}

pub fn validate_create_catalyst(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("target");
    vd.field_item("type", Item::DiplomaticCatalyst);
    vd.field_target("target", sc, Scopes::Country);
}

pub fn validate_change_appeasement(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("amount");
    vd.req_field("factor");
    vd.field_item("factor", Item::PoliticalLobbyAppeasement);
    vd.field_script_value("amount", sc);
}

/// Validate `set_pop_wealth` and `add_pop_wealth`
pub fn validate_pop_wealth(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("wealth_distribution");
    vd.field_script_value("wealth_distribution", sc);
    vd.field_bool("update_loyalties");
}

pub fn validate_kill_character(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(value) => {
            // kill_character = yes
            let mut vd = ValueValidator::new(value, data);
            vd.bool();
        }
        BV::Block(block) => {
            // kill_character = { hidden = yes value = yes }
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.field_bool("value");
            vd.field_bool("hidden");
        }
    }
}
