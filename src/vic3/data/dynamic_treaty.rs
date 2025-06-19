use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DynamicTreatyName {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DynamicTreatyName, DynamicTreatyName::add)
}

impl DynamicTreatyName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynamicTreatyName, key, block, Box::new(Self {}));
    }
}

impl DbKind for DynamicTreatyName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::TreatyOptions);
        vd.field_script_value_rooted("weight", Scopes::TreatyOptions);
    }
}
