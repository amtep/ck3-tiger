use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct TriggerLocalization {}

impl TriggerLocalization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TriggerLocalization, key, block, Box::new(Self {}));
    }
}

impl DbKind for TriggerLocalization {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_item("global", Item::Localization);
        vd.field_item("global_not", Item::Localization);
        vd.field_item("first", Item::Localization);
        vd.field_item("first_not", Item::Localization);
        vd.field_item("third", Item::Localization);
        vd.field_item("third_not", Item::Localization);
    }
}
