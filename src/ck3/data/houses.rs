use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct House {}

impl House {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::House, key, block, Box::new(Self {}));
    }
}

impl DbKind for House {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("name");
        vd.req_field("dynasty");

        vd.field_item("name", Item::Localization);
        vd.field_item("prefix", Item::Localization);
        vd.field_item("motto", Item::Localization);
        vd.field_item("dynasty", Item::Dynasty);
        vd.field_value("forced_coa_religiongroup"); // TODO
    }
}
