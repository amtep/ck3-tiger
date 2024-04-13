use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Goods {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Goods, Goods::add)
}

impl Goods {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Goods, key, block, Box::new(Self {}));
    }
}

impl DbKind for Goods {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("texture", Item::File);
        vd.field_numeric("cost");
        vd.field_choice("category", &["industrial", "luxury", "military", "staple"]);

        vd.field_bool("local");
        vd.field_bool("tradeable");
        vd.field_bool("fixed_price");

        vd.field_numeric("prestige_factor");
        vd.field_numeric("traded_quantity");
        vd.field_numeric("convoy_cost_multiplier");

        vd.field_numeric("obsession_chance");
        vd.field_numeric("consumption_tax_cost");

        vd.field_bool("pop_consumption_can_add_infrastructure"); // undocumented
    }
}
