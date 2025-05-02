use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ProvinceEffect {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ProvinceEffect, ProvinceEffect::add)
}

impl ProvinceEffect {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ProvinceEffect, key, block, Box::new(Self {}));
    }
}

impl DbKind for ProvinceEffect {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_integer("effect_index");
        vd.field_bool("is_winter_effect");
    }
}
