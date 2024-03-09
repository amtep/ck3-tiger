use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct EventEffect2d {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::EventEffect2d, EventEffect2d::add)
}

impl EventEffect2d {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventEffect2d, key, block, Box::new(Self {}));
    }
}

impl DbKind for EventEffect2d {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("effect_2d");
        vd.multi_field_validated_block("effect_2d", |block, data| {
            let mut vd = Validator::new(block, data);
            // TODO: use `validate_call` for event scope
            vd.field_block("trigger");
            vd.req_field("reference");
            vd.field_item("reference", Item::File);
            vd.field_item("mask", Item::File);
            vd.field_choice("mask_type", &["texture", "video"]);
            vd.field_numeric_range("duration", 0.0..);
        });
    }
}
