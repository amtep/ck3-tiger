use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::context::ScopeContext;
use crate::item::{Item, ItemLoader};
use crate::game::GameFlags;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug)]
pub struct LevyTemplate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::LevyTemplate, LevyTemplate::add)
}

impl LevyTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AiPlanGoals, key, block, Box::new(Self {}));
    }
}

impl DbKind for LevyTemplate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.unknown_block_fields(|key, block| {
            vd.field_bool("default");
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|_, value| {
                data.verify_exists(Item::UnitType, value);
            });
        });
    }
}
