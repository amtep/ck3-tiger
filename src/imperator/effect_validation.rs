use crate::block::{BV, Block};
use crate::context::ScopeContext;
use crate::effect::validate_effect_internal;
use crate::everything::Everything;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target;
use crate::validate::{ListType, validate_optional_duration};
use crate::validator::{Validator, ValueValidator};

pub fn validate_trade_route(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("target", sc, Scopes::State);
    vd.field_item("goods", Item::TradeGood);
}

pub fn validate_add_deity_to_pantheon(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target("deity", sc, Scopes::Deity);
}

pub fn validate_remove_subunit_loyalty(
    _key: &Token,
    mut vd: ValueValidator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.maybe_is("yes");
    vd.target_ok_this(sc, Scopes::SubUnit);
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
            vd.field_choice("mode", &["add", "add_and_extend", "extend"]);
            validate_optional_duration(&mut vd, sc);
        }
    }
}

pub fn validate_add_party_conviction_or_approval(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("value");
    vd.field_item_or_target("party", sc, Item::PartyType, Scopes::Party);
    vd.field_item_or_target("party_type", sc, Item::PartyType, Scopes::Party);
    vd.field_script_value("value", sc);
}

pub fn validate_death(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("death_reason", Item::DeathReason);
    vd.field_target("killer", sc, Scopes::Character);
    vd.field_bool("silent");
}

pub fn validate_deify_character(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("deity");
    vd.req_field("country");
    vd.field_target("deity", sc, Scopes::Deity);
    vd.field_item_or_target("country", sc, Item::Localization, Scopes::Country);
}

pub fn validate_legion_history(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("key");
    vd.req_field("commander");
    vd.req_field("province");
    vd.field_item("key", Item::Localization);
    vd.field_target("commander", sc, Scopes::Character);
    vd.field_item_or_target("province", sc, Item::Province, Scopes::Province);
    vd.field_date("date");
}

pub fn validate_make_pregnant(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("father");
    vd.field_target("father", sc, Scopes::Character);
    vd.field_bool("known_bastard");
    vd.field_integer("number_of_children");
}

pub fn validate_change_opinion(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("modifier");
    vd.req_field("target");
    vd.field_item("modifier", Item::Opinion);
    vd.field_item_or_target("target", sc, Item::Localization, Scopes::Country);
}

pub fn validate_add_research(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("technology");
    vd.req_field("value");
    vd.field_item("technology", Item::TechnologyTable);
    vd.field_script_value("value", sc);
}

pub fn validate_add_to_war(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("attacker");
    vd.field_target("target", sc, Scopes::War);
    vd.field_bool("attacker");
    vd.field_bool("leader");
}

pub fn validate_add_truce(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("duration");
    vd.field_item_or_target("target", sc, Item::Localization, Scopes::Country);
    vd.field_integer("duration");
}

pub fn validate_declare_war(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("war_goal");
    vd.req_field("target");
    vd.field_item("war_goal", Item::Wargoal);
    vd.field_item_or_target("target", sc, Item::Localization, Scopes::Country);
    vd.field_item_or_target("province", sc, Item::Province, Scopes::Province);
}

pub fn validate_imprison(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.field_target("target", sc, Scopes::Character);
}

pub fn validate_make_subject(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("type");
    vd.field_item_or_target("target", sc, Item::Localization, Scopes::Country);
    vd.field_item("type", Item::SubjectType);
}

pub fn validate_release_prisoner(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            validate_target(token, data, sc, Scopes::Character);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("target");
            vd.field_target("target", sc, Scopes::Character);
        }
    }
}

pub fn validate_define_pop(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("religion");
    vd.req_field("culture");
    vd.field_item("type", Item::PopType);
    vd.field_item_or_target("religion", sc, Item::Religion, Scopes::Religion);
    vd.field_item_or_target("culture", sc, Item::Culture, Scopes::Culture);
}

pub fn validate_create_treasure(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("key");
    vd.req_field("icon");
    vd.field_validated_block("modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Country | ModifKinds::Province, vd);
    });
    vd.field_validated_block("character_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
}

pub fn validate_raise_legion(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    sc.open_scope(Scopes::Legion, key.clone());
    vd.req_field_warn("create_unit");
    validate_effect_internal(&caller, ListType::None, block, data, sc, &mut vd, tooltipped);
    sc.close();
}

pub fn validate_create_character(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    sc.open_scope(Scopes::Character, key.clone());
    vd.field_item("first_name", Item::Localization);
    vd.field_item("family_name", Item::Localization);
    vd.field_item("nickname", Item::Localization);
    vd.field_value("dna");
    vd.field_item_or_target("culture", sc, Item::Culture, Scopes::Culture);
    vd.field_item_or_target("religion", sc, Item::Religion, Scopes::Religion);
    vd.field_target("family", sc, Scopes::Family);
    vd.field_target("father", sc, Scopes::Character);
    vd.field_target("mother", sc, Scopes::Character);
    vd.field_bool("female");
    vd.field_bool("no_stats");
    vd.field_bool("no_traits");
    vd.field_script_value("age", sc);
    vd.field_date("birth_date");
    vd.field_date("death_date");
    vd.field_item_or_target("birth_province", sc, Item::Province, Scopes::Province);
    validate_effect_internal(&caller, ListType::None, block, data, sc, &mut vd, tooltipped);
    sc.close();
}

pub fn validate_create_unit(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    sc.open_scope(Scopes::Unit, key.clone());
    vd.field_item("name", Item::Localization);
    vd.field_item_or_target("location", sc, Item::Province, Scopes::Province);
    vd.field_target("commander", sc, Scopes::Character);
    vd.field_bool("navy");
    vd.field_item("sub_unit", Item::Unit);
    validate_effect_internal(&caller, ListType::None, block, data, sc, &mut vd, tooltipped);
    sc.close();
}

pub fn validate_create_country(
    key: &Token,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    sc.open_scope(Scopes::Country, key.clone());
    vd.field_validated_block("name", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("name", Item::Localization);
        vd.field_item("adjective", Item::Localization);
    });
    validate_effect_internal(&caller, ListType::None, block, data, sc, &mut vd, tooltipped);
    sc.close();
}

pub fn validate_pay_gold(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("gold");
    vd.field_item_or_target("target", sc, Item::Localization, Scopes::Country);
    vd.field_numeric("gold");
}

pub fn validate_great_work_construction(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("great_work", Item::GreatWorkTemplate);
    vd.field_choice("locator", &["primary_great_work", "secondary_great_work", "great_work"]);
    vd.field_bool("is_ancient");
}
