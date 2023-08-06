use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_optional_duration;
use crate::validator::Validator;

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
    vd.field_item("type", Item::Journalentry);
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
    vd.field_integer("progress");
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
    vd.field_target("region", sc, Scopes::StateRegion);
    vd.field_bool("primary_demand");
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
