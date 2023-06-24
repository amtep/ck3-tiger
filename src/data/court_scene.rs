use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct CourtSceneGroup {}

impl CourtSceneGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtSceneGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtSceneGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("order_type", &["random", "ascending", "descending"]);
        vd.field_choice("position_type", &["dynamic", "static"]);
        vd.field_choice("access_type", &["random", "top"]);
        vd.field_value("value"); // TODO
    }
}
