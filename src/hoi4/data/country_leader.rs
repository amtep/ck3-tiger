use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::hoi4::validate::validate_equipment_bonus;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CountryLeaderTrait {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::CountryLeaderTrait, CountryLeaderTrait::add)
}

impl CountryLeaderTrait {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("leader_traits") {
            for (item, block) in block.drain_definitions_warn() {
                db.add(Item::CountryLeaderTrait, item, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `leader_traits` here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl DbKind for CountryLeaderTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        if let Some(name) = vd.field_value("name") {
            data.verify_exists(Item::Localization, name);
        } else {
            data.verify_exists(Item::Localization, key);
        }

        vd.field_bool("random");
        vd.field_integer("sprite");
        vd.field_numeric("command_cap_increase");
        vd.field_numeric("command_power");
        vd.field_item("custom_modifier_tooltip", Item::Localization);
        vd.multi_field_validated_block("ai_strategy", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("type", Item::AiStrategyType);
            vd.field_item("id", Item::CountryTag);
            vd.field_numeric("value");
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_numeric("command_cap");
        vd.multi_field_validated_block("targeted_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("tag", Item::CountryTag);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.multi_field_validated_block("equipment_bonus", validate_equipment_bonus);
        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}
