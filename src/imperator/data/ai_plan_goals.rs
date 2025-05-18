use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct AiPlanGoals {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::AiPlanGoals, AiPlanGoals::add)
}

impl AiPlanGoals {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AiPlanGoals, key, block, Box::new(Self {}));
    }
}

impl DbKind for AiPlanGoals {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        // Nothing else can be validated other than the country scoped trigger here
        // This is because all other fields are "ai factors" and there are hundreds of them
        vd.no_warn_remaining();

        vd.field_integer("migration_chance");
        vd.field_integer("aggressive");
        vd.field_integer("trustworthy");
        vd.field_integer("ae_ceiling");
        vd.field_integer("economy_exponent");

        vd.field_trigger("trigger", Tooltipped::No, &mut sc);
    }
}
