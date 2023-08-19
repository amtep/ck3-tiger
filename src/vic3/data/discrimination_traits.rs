use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DiscriminationTrait {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DiscriminationTrait, DiscriminationTrait::add)
}

impl DiscriminationTrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DiscriminationTrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for DiscriminationTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_bool("heritage");
    }
}
