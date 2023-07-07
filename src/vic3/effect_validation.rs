use std::str::FromStr;

use crate::block::validator::Validator;
use crate::block::{Block, Comparator, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_control, validate_normal_effect};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{error, warn, warn_info, ErrorKey};
use crate::scopes::Scopes;
use crate::scriptvalue::{validate_non_dynamic_scriptvalue, validate_scriptvalue};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target, validate_target_ok_this, validate_trigger_key_bv};
use crate::validate::{
    validate_duration, validate_optional_duration, validate_optional_duration_int,
    validate_random_culture, validate_random_faith, validate_random_traits_list, ListType,
};
use EvB::*;
use EvBv::*;
use EvV::*;

#[derive(Debug, Copy, Clone)]
pub enum EvB {}

#[derive(Debug, Copy, Clone)]
pub enum EvBv {}

#[derive(Debug, Copy, Clone)]
pub enum EvV {}

pub fn validate_effect_block(
    v: EvB,
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    let mut vd = Validator::new(block, data);
    vd.set_case_sensitive(false);
    match v {}
}

pub fn validate_effect_value(
    v: EvV,
    _key: &Token,
    value: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match v {}
}

pub fn validate_effect_bv(
    v: EvBv,
    key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match v {}
}
