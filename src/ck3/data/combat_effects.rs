use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CombatEffect {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CombatEffect, CombatEffect::add)
}

impl CombatEffect {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CombatEffect, key, block, Box::new(Self {}));
    }
}

impl DbKind for CombatEffect {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let visible = !block.field_value_is("visible", "no");

        let name = vd.field_value("name").unwrap_or(key);
        if visible {
            data.verify_exists(Item::Localization, name);
        }

        let icon = vd.field_value("image").unwrap_or(key);
        if visible {
            data.mark_used_icon("NGameIcons|COMBAT_EFFECT_ICON_PATH", icon, ".dds");
        }

        vd.field_numeric("advantage");
        vd.field_bool("adjacency");
        vd.advice_field("fortification", "removed in 1.13");
        vd.field_bool("visible");
    }
}
