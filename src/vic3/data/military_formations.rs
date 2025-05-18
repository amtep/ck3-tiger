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
pub struct MilitaryFormationFlags {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MilitaryFormationFlags, MilitaryFormationFlags::add)
}

impl MilitaryFormationFlags {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MilitaryFormationFlags, key, block, Box::new(Self {}));
    }
}

impl DbKind for MilitaryFormationFlags {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_item("icon", Item::File);
        vd.field_choice("type", &["army", "navy"]);
    }
}
