use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct House {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::House, House::add)
}

impl House {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::House, key, block, Box::new(Self {}));
    }

    pub fn get_dynasty<'a>(key: &str, data: &'a Everything) -> Option<&'a Token> {
        data.database
            .get_key_block(Item::House, key)
            .and_then(|(_, block)| block.get_field_value("dynasty"))
    }
}

impl DbKind for House {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("name");
        vd.req_field("dynasty");

        vd.field_item("name", Item::Localization);
        vd.field_item("prefix", Item::Localization);
        vd.field_item("motto", Item::Localization);
        vd.field_item("dynasty", Item::Dynasty);
        vd.field_value("forced_coa_religiongroup"); // TODO
    }
}
