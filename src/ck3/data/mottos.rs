use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct MottoInsert {}

impl MottoInsert {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MottoInsert, key, block, Box::new(Self {}));
    }
}

impl DbKind for MottoInsert {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        for (key, block) in vd.unknown_block_fields() {
            let mut vd = Validator::new(block, data);
            let loca = format!("motto_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
            vd.field_validated_block_rooted("trigger", Scopes::Character, |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            });
            vd.field_script_value_rooted("weight", Scopes::Character);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Motto {}

impl Motto {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Motto, key, block, Box::new(Self {}));
    }
}

impl DbKind for Motto {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let loca = format!("motto_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let n = block.count_keys("insert") as i64;
        data.localization.verify_key_has_options(&loca, key, n, "");
        vd.field_items("insert", Item::MottoInsert);
        vd.field_validated_block_rooted("trigger", Scopes::Character, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_script_value_rooted("weight", Scopes::Character);
    }
}
