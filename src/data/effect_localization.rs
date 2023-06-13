use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct EffectLocalization {}

impl EffectLocalization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EffectLocalization, key, block, Box::new(Self {}));
    }
}

impl DbKind for EffectLocalization {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_item("global", Item::Localization);
        vd.field_item("global_past", Item::Localization);
        vd.field_item("global_neg", Item::Localization);
        vd.field_item("global_past_neg", Item::Localization);
        vd.field_item("first", Item::Localization);
        vd.field_item("first_past", Item::Localization);
        vd.field_item("first_neg", Item::Localization);
        vd.field_item("first_past_neg", Item::Localization);
        vd.field_item("third", Item::Localization);
        vd.field_item("third_past", Item::Localization);
        vd.field_item("third_neg", Item::Localization);
        vd.field_item("third_past_neg", Item::Localization);
    }
}
