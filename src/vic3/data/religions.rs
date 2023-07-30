use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;
use crate::validate::validate_possibly_named_color;

#[derive(Clone, Debug)]
pub struct Vic3Religion {}

impl Vic3Religion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Religion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Vic3Religion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("texture", Item::File);
        vd.field_list("traits"); // TODO
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_list_items("taboos", Item::Goods);
    }
}
