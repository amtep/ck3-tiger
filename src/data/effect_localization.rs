use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::errorkey::ErrorKey;
use crate::errors::warn2;
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug)]
pub struct EffectLocalization {}

impl EffectLocalization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EffectLocalization, key, block, Box::new(Self {}));
    }

    // TODO: when are the _neg effect tooltips used?
    pub fn validate_use(
        key: &Token,
        block: &Block,
        data: &Everything,
        caller: &Token,
        tooltipped: Tooltipped,
    ) {
        match tooltipped {
            Tooltipped::No => return,
            Tooltipped::Yes | Tooltipped::Negated => {
                for field in &["global", "first", "third"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                let msg = format!("missing present perspective");
                warn2(caller, ErrorKey::MissingPerspective, &msg, key, "here");
            }
            Tooltipped::Past => {
                for field in &["global_past", "first_past", "third_past"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                for field in &["global", "first", "third"] {
                    if let Some(token) = block.get_field_value(field) {
                        // TODO: check if these are auto-guessed if _past key is missing
                        let loca = format!("{token}_PAST");
                        if data.item_exists(Item::Localization, &loca) {
                            return;
                        }
                    }
                }
                let msg = format!("missing `_past` perspective");
                warn2(caller, ErrorKey::MissingPerspective, &msg, key, "here");
            }
        }
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
