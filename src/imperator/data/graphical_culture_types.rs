use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct GraphicalCultureType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GraphicalCultureType, GraphicalCultureType::add)
}

impl GraphicalCultureType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::GraphicalCultureType, key, block, Box::new(Self {}));
    }
}

impl DbKind for GraphicalCultureType {
    fn validate(&self, _key: &Token, _block: &Block, _data: &Everything) {
        // Literally nothing
    }
}
