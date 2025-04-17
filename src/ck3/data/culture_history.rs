use crate::block::Block;
use crate::date::Date;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile};
use crate::pdxfile::PdxEncoding;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CultureHistory {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::CultureHistory, PdxEncoding::Detect, ".txt", LoadAsFile::Yes, CultureHistory::add)
}

impl CultureHistory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureHistory, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureHistory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        if key.starts_with("heritage_") {
            data.verify_exists(Item::CultureHeritage, key);
        } else {
            data.verify_exists(Item::Culture, key);
        }

        let mut vd = Validator::new(block, data);
        vd.validate_history_blocks(validate_history);
    }
}

fn validate_history(_date: Date, _key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.multi_field_item("discover_innovation", Item::Innovation);
    vd.multi_field_validated_block("add_innovation_progress", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("culture_innovation", Item::Innovation);
        vd.field_numeric_range("progress", 0.0..=100.0);
    });
    vd.field_item("join_era", Item::CultureEra);
    vd.field_numeric_range("progress_era", 0.0..=100.0);
}
