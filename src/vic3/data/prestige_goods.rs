use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PrestigeGoods {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PrestigeGoods, PrestigeGoods::add)
}

impl PrestigeGoods {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PrestigeGoods, key, block, Box::new(Self {}));
    }
}

impl DbKind for PrestigeGoods {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("base_good", Item::Goods);
        vd.field_numeric("prestige_bonus");
        vd.field_item("texture", Item::File);

        vd.field_trigger_rooted("is_possible", Tooltipped::No, Scopes::Country);
    }
}
