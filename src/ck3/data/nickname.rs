use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Nickname {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Nickname, Nickname::add)
}

impl Nickname {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Nickname, key, block, Box::new(Self {}));
    }
}

impl DbKind for Nickname {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);
        data.localization.suggest(&format!("{key}_desc"), key);
        vd.field_bool("is_prefix");
        vd.field_bool("is_bad");
    }
}
