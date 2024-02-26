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
pub struct DeathReason {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::DeathReason, DeathReason::add)
}

impl DeathReason {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DeathReason, key, block, Box::new(Self {}));
    }
}

impl DbKind for DeathReason {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::State, key);

        vd.field_bool("default");
        vd.field_bool("natural");
        vd.field_numeric("priority");

        vd.field_validated_block("natural_death_trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
    }
}
