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
pub struct Decision {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Decision, Decision::add)
}

impl Decision {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Decision, key, block, Box::new(Self {}));
    }
}

impl DbKind for Decision {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("possible", Tooltipped::Yes, &mut sc);
        vd.field_effect("when_taken", Tooltipped::Yes, &mut sc);

        vd.field_script_value_no_breakdown("ai_chance", &mut sc);
    }
}
