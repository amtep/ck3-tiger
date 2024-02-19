use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct LegionDistinction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::LegionDistinction, LegionDistinction::add)
}

impl LegionDistinction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LegionDistinction, key, block, Box::new(Self {}));
    }
}

impl DbKind for LegionDistinction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("icon");
        vd.req_field("commander");
        vd.req_field("unit");

        vd.field_item("icon", Item::File);

        vd.field_validated_block("commander", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("unit", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block("legion", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
    }
}
