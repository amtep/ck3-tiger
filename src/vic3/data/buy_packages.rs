use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct BuyPackage {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::BuyPackage, BuyPackage::add)
}

impl BuyPackage {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BuyPackage, key, block, Box::new(Self {}));
    }

    pub fn crosscheck(_data: &Everything) {
        // TODO: check that wealth_1 through wealth_99 are all present.
        // Problem: what would be the loc for the warning about a missing one?
    }
}

impl DbKind for BuyPackage {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // Check that this is one of the expected keys, wealth_1 through wealth_99.
        if let Some(wealth_nr) = key.strip_prefix("wealth_") {
            if let Some(nr) = wealth_nr.expect_integer() {
                if !(1..=99).contains(&nr) {
                    let msg = "expected wealth between 1 and 99";
                    err(ErrorKey::Range).msg(msg).loc(key).push();
                }
            }
        } else {
            let msg = "expected wealth_NN format for buy_package keys";
            err(ErrorKey::Validation).msg(msg).loc(key).push();
        }

        vd.field_numeric("political_strength");
        vd.field_validated_block("goods", validate_goods);
    }
}

fn validate_goods(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, value| {
        data.verify_exists(Item::PopNeed, key);
        value.expect_integer();
    });
}
