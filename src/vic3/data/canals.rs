use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CanalType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CanalType, CanalType::add)
}

impl CanalType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CanalType, key, block, Box::new(Self {}));
    }
}

impl DbKind for CanalType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_item("texture", Item::File);
        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_item("state_region", Item::StateRegion);
    }
}
