use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Loyalty {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Loyalty, Loyalty::add)
}

impl Loyalty {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Loyalty, key, block, Box::new(Self {}));
    }
}

impl DbKind for Loyalty {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("value");

        vd.field_numeric("value");
        vd.field_numeric("min");
        vd.field_numeric("max");
        vd.field_numeric("yearly_decay");
        vd.field_numeric("months");
    }
}
