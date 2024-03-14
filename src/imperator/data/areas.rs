use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Area {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Area, Area::add)
}

impl Area {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Area, key, block, Box::new(Self {}));
    }
}

impl DbKind for Area {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("color", validate_color);
        vd.field_list("provinces");
    }
}
