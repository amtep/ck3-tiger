use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct InterestGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::InterestGroup, InterestGroup::add)
}

impl InterestGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::InterestGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for InterestGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_only_icon");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("texture", Item::File);
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("layer", Item::MapLayer);
        vd.field_integer("index"); // TODO: do these have to be consecutive?

        vd.field_list_items("ideologies", Item::Ideology);
        vd.field_list_items("character_ideologies", Item::Ideology);
        // deprecated
        vd.field_list_items("traits", Item::InterestGroupTrait);
        vd.advice_field("traits", "deprecated; use on_enable effect to assign traits instead");

        vd.field_validated_key_block("enable", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_enable", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_disable", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_character_ig_membership", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("interest_group", Scopes::InterestGroup, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key_block("pop_potential", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            sc.define_name("interest_group", Scopes::InterestGroup, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key("pop_weight", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            sc.define_name("interest_group", Scopes::InterestGroup, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_script_value_rooted("monarch_weight", Scopes::InterestGroup);
        vd.field_script_value_rooted("agitator_weight", Scopes::InterestGroup);
        vd.field_script_value_rooted("commander_weight", Scopes::InterestGroup);
        vd.field_script_value_rooted("female_commander_chance", Scopes::InterestGroup);
        vd.field_script_value_rooted("female_politician_chance", Scopes::InterestGroup);
        vd.field_script_value_rooted("female_agitator_chance", Scopes::InterestGroup);

        vd.field_script_value_rooted("noble_chance", Scopes::None);
        vd.field_validated_key("commander_leader_chance", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::InterestGroup, key);
            sc.define_name("character", Scopes::Character, key);
            validate_script_value(bv, data, &mut sc);
        });

        // TODO: figure out these scopes

        vd.field_validated_block("priority_cultures", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_validated_block("rule", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_validated_key_block("trigger", |key, block, data| {
                    let mut sc = ScopeContext::new(Scopes::Country, key);
                    validate_trigger(block, data, &mut sc, Tooltipped::No);
                });
                vd.field_list_items("cultures", Item::Culture);
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct InterestGroupTrait {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::InterestGroupTrait, InterestGroupTrait::add)
}

impl InterestGroupTrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::InterestGroupTrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for InterestGroupTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_item("min_approval", Item::Approval);
        vd.field_item("max_approval", Item::Approval);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
    }
}
