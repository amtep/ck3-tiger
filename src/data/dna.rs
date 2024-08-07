//! Character DNA for portraits. Used in CK3 and Vic3 but not Imperator.

use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Dna {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3.union(GameFlags::Vic3), Item::Dna, Dna::add)
}

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
    vd.field_validated_block("genes", validate_genes);
    if Game::is_vic3() {
        vd.field_block("entity");
    }
}

pub fn validate_genes(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::GeneCategory, key);
        data.validate_use(Item::GeneCategory, key, block);
    });
}
