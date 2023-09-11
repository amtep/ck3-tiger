use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CombatTactic {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::CombatTactic, CombatTactic::add)
}

impl CombatTactic {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CombatTactic, key, block, Box::new(Self {}));
    }
}

impl DbKind for CombatTactic {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_bool("use_as_default");
        vd.field_bool("navy");
        vd.field_bool("enable");

        vd.field_item("sound", Item::Sound);

        vd.unknown_value_fields(|_, value| {
            data.verify_exists(Item::CombatTactic, key);
            value.expect_number();
        });

        vd.field_validated_block("effective_composition", |block, data| {
            // TODO - this could be better, the structure is similar to levy templates.
            data.verify_exists(Item::CombatTactic, key);
            // value.expect_number();
        });
    }
}
