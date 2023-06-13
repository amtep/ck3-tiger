use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Nickname {}

impl Nickname {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Nickname, key, block, Box::new(Self {}));
    }
}

impl DbKind for Nickname {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);
        vd.field_bool("is_prefix");
        vd.field_bool("is_bad");
    }
}
