use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::effect::{
    validate_add_to_variable_list, validate_change_variable, validate_clamp_variable,
    validate_random_list, validate_round_variable, validate_save_scope_value,
    validate_set_variable, validate_switch,
};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

use EvB::*;
use EvBv::*;
use EvV::*;

#[derive(Debug, Copy, Clone)]
pub enum EvB {
    ActivateProductionMethod,
    AddToVariableList,
    ChangeVariable,
    ClampVariable,
    RandomList,
    RoundVariable,
    SaveScopeValue,
    Switch,
}

#[derive(Debug, Copy, Clone)]
pub enum EvBv {
    SetVariable,
}

#[derive(Debug, Copy, Clone)]
pub enum EvV {
    AddToList,
    RemoveFromList,
    SaveScope,
}

pub fn validate_effect_block(
    v: EvB,
    _key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);
    vd.set_case_sensitive(false);
    match v {
        ActivateProductionMethod => {
            vd.req_field("building_type");
            vd.req_field("production_method");
            vd.field_item("building_type", Item::BuildingType);
            // TODO: check that the production method belongs to the building type
            vd.field_item("production_method", Item::ProductionMethod);
        }
        AddToVariableList => {
            validate_add_to_variable_list(vd, sc);
        }
        ChangeVariable => {
            validate_change_variable(vd, sc);
        }
        ClampVariable => {
            validate_clamp_variable(vd, sc);
        }
        RandomList => {
            validate_random_list("random_list", block, data, vd, sc, tooltipped);
        }
        RoundVariable => {
            validate_round_variable(vd, sc);
        }
        SaveScopeValue => {
            validate_save_scope_value(vd, sc);
        }
        Switch => {
            validate_switch(vd, data, sc, tooltipped);
        }
    }
}

pub fn validate_effect_value(
    v: EvV,
    _key: &Token,
    value: &Token,
    _data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match v {
        AddToList => {
            sc.define_or_expect_list(value);
        }
        RemoveFromList => {
            sc.expect_list(value);
        }
        SaveScope => {
            sc.save_current_scope(value.as_str());
        }
    }
}

pub fn validate_effect_bv(
    v: EvBv,
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match v {
        SetVariable => {
            validate_set_variable(bv, data, sc);
        }
    }
}
