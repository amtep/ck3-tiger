use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

/*
Example:
<combat_tactic> = {
    use_as_default = yes

    enable = yes

    sound = "event:/SFX/UI/Unit/sfx_ui_unit_tactic_set_offensive"

    <combat_tactic> = 0.20
    <combat_tactic> = 0.20
    <combat_tactic> = -0.1
    <combat_tactic> = -0.1

    casualties = 0.1

    effective_composition = {
        <unit_type> = 0
        <unit_type> = 0
        <unit_type> = 0
        <unit_type> = 1.0
        <unit_type> = 1.0
        <unit_type> = 0
        <unit_type> = 0.0
        <unit_type> = 0.75
        <unit_type> = 2.0
    }
}
*/

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

        vd.field_numeric("casualties");

        vd.field_item("sound", Item::Sound);

        vd.field_validated_block("effective_composition", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Unit, key);
                value.expect_number();
            });
        });
    }
}
