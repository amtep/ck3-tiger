use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct AcceptanceStatus {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::AcceptanceStatus, AcceptanceStatus::add)
}

impl AcceptanceStatus {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AcceptanceStatus, key, block, Box::new(Self {}));
    }
}

impl DbKind for AcceptanceStatus {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_only_icon");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_numeric("threshold");
        vd.field_numeric("base_migration_desire");
        vd.field_numeric("war_exhaustion_impact_own_side");
        vd.field_numeric("war_exhaustion_impact_other_side");

        // undocumented

        // TODO: find out if color = { R G B } is the only option
        vd.field_validated("color", validate_possibly_named_color);

        vd.field_item("small_icon", Item::File);
        vd.field_item("large_icon", Item::File);
        vd.field_item("background", Item::File);
    }
}
