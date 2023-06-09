use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Holding {}

impl Holding {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::HoldingFlag, token);
        }
        db.add(Item::Holding, key, block, Box::new(Self {}));
    }
}

impl DbKind for Holding {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_values("flag");
        vd.field_item("primary_building", Item::Building);
        vd.field_list_items("buildings", Item::Building);
        vd.field_bool("can_be_inherited");
    }
}
