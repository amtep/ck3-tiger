use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PortraitType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::PortraitType, PortraitType::add)
}

impl PortraitType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PortraitType, key, block, Box::new(Self {}));
    }
}

impl DbKind for PortraitType {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        for field in &["colors", "properties"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("skin", Item::File);
                vd.field_item("eye", Item::File);
                vd.field_item("hair", Item::File);
            });
        }

        for field in &["male", "female", "boy", "girl"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_choice("sex", &["male", "female"]);
                vd.field_integer("minimum_age");
                vd.field_integer("maximum_age");
                vd.field_item("head", Item::Entity);
                vd.field_item("torso", Item::Entity);
            });
        }

        vd.multi_field_block("attach"); // TODO
    }
}
