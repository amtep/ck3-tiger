use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct CourtPositionCategory {}

impl CourtPositionCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtPositionCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtPositionCategory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
    }
}
