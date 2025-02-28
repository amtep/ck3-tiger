use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct TriggerLocalization {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::TriggerLocalization, TriggerLocalization::add)
}

impl TriggerLocalization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TriggerLocalization, key, block, Box::new(Self {}));
    }

    pub fn validate_use(
        key: &Token,
        block: &Block,
        data: &Everything,
        caller: &Token,
        tooltipped: Tooltipped,
        negated: bool,
    ) {
        if tooltipped.is_tooltipped() {
            if negated {
                for field in &["global_not", "first_not", "third_not", "none_not"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                for field in &["global", "first", "third"] {
                    if let Some(token) = block.get_field_value(field) {
                        let loca = format!("NOT_{token}");
                        if data.item_exists(Item::Localization, &loca) {
                            return;
                        }
                    }
                }
                let msg = format!("missing `NOT_` perspective for {key}");
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).loc_msg(key, "here").push();
            } else {
                for field in &["global", "first", "third", "none"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                let msg = format!("missing positive perspective for {key}");
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).loc_msg(key, "here").push();
            }
        }
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
        vd.field_item("none", Item::Localization);
        vd.field_item("none_not", Item::Localization);
    }
}
