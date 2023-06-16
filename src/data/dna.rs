use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Dna {}

impl Dna {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Dna, key, block, Box::new(Self {}));
    }
}

impl DbKind for Dna {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("portrait_info", validate_portrait_info);
        vd.field_bool("enabled");
    }
}

fn validate_portrait_info(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("genes", |block, data| {
        let mut vd = Validator::new(block, data);
        for (key, bv) in vd.unknown_keys() {
            data.verify_exists(Item::GeneCategory, key);
            if let Some(block) = bv.expect_block() {
                data.validate_use(Item::GeneCategory, key, block);
            }
        }
    });
}
