use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GovernmentType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::GovernmentType, GovernmentType::add)
}

impl GovernmentType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GovernmentType, key, block, Box::new(Self {}));
    }
}

impl DbKind for GovernmentType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("transfer_of_power", Item::TransferOfPower);
        vd.field_bool("new_leader_on_reform_government");

        vd.field_item("male_ruler", Item::Localization);
        vd.field_item("female_ruler", Item::Localization);
        vd.field_item("male_heir", Item::Localization);
        vd.field_item("female_heir", Item::Localization);

        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_government_type_change", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}
