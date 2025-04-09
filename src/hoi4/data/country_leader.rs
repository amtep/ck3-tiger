use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
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

        data.verify_exists(Item::Localization, key);

        vd.field_bool("random");
        vd.field_integer("sprite");
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_numeric("command_cap");
        vd.multi_field_validated_block("targeted_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("tag", Item::CountryTag);
            validate_modifs(block, data, ModifKinds::Country | ModifKinds::Army, vd);
        });
        vd.multi_field_validated_block("equipment_bonus", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                data.verify_exists(Item::EquipmentBonusType, key);
                let mut vd = Validator::new(block, data);
                vd.field_bool("instant");
                vd.unknown_value_fields(|key, value| {
                    data.verify_exists(Item::EquipmentStat, key);
                    value.expect_number();
                });
            });
        });
        validate_modifs(block, data, ModifKinds::Country, vd);
    }
}
