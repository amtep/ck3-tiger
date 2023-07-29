use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::validate::validate_color;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct DeityCategory {}

impl DeityCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DeityCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for DeityCategory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}").to_uppercase();
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
    }
}