use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::context::ScopeContext;
use crate::item::{Item, ItemLoader};
use crate::game::GameFlags;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_color;

#[derive(Clone, Debug)]
pub struct CultureGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::CultureGroup, CultureGroup::add)
}

impl CultureGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca1 = format!("{key}_malename");
        let loca2 = format!("{key}_femalename");
        let loca3 = format!("ARMY_NAME_{key}");
        let loca4 = format!("NAVY_NAME_{key}");
        let loca5 = format!("COHORT_NAME_{key}");
        data.verify_exists_implied(Item::Localization, &loca1, key);
        data.verify_exists_implied(Item::Localization, &loca2, key);
        data.verify_exists_implied(Item::Localization, &loca3, key);
        data.verify_exists_implied(Item::Localization, &loca4, key);
        data.verify_exists_implied(Item::Localization, &loca5, key);

        vd.field_validated_block("color", validate_color);
        vd.field_item("primary", Item::Unit);
        vd.field_item("second", Item::Unit);
        vd.field_item("flank", Item::Unit);
        vd.field_item("primary_navy", Item::Unit);
        vd.field_item("secondary_navy", Item::Unit);
        vd.field_item("flank_navy", Item::Unit);
        vd.field_item("levy_template", Item::LevyTemplate);
        vd.field_item("graphical_culture", Item::GraphicalCultureType);

        vd.field_list("male_names");
        vd.field_list("female_names");
        vd.field_list("family");
        vd.field_list("barbarian_names");

        // TODO - How do I do the Culture item here? cultures are defined inside of the CultureGroup item...

        // TODO - Not sure what to do with ethnicities...
        vd.field_block("ethnicities")
    }
}
