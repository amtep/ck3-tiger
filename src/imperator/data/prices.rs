use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Price {}

impl Price {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Price, key, block, Box::new(Self {}));
    }
}

impl DbKind for Price {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_numeric("political_influence");
        vd.field_numeric("stability");
        vd.field_numeric("gold");
        vd.field_numeric("tyranny");
        vd.field_numeric("scaled_gold");
        vd.field_numeric("scaled_manpower");
        vd.field_numeric("aggressive_expansion");
        vd.field_numeric("innovations");
        vd.field_numeric("military_experience");
        vd.field_numeric("war_exhaustion");
    }
}