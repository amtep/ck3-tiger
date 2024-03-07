use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CultureGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::CultureGroup, CultureGroup::add)
}

impl CultureGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("culture") {
            for (culture, block) in block.iter_definitions() {
                db.add(Item::Culture, culture.clone(), block.clone(), Box::new(Culture {}));
            }
        }
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
        vd.field_bool("use_latin_name_rules");

        vd.field_validated_block("nickname", validate_name_list);
        vd.field_validated_block("female_order", validate_name_list);
        vd.field_validated_block("male_names", validate_name_list);
        vd.field_validated_block("female_names", validate_name_list);
        vd.field_validated_block("barbarian_names", validate_name_list);
        vd.field_validated_block("family", validate_family_name_list);

        vd.field_block("culture"); // validated by Culture class

        vd.field_validated_block("ethnicities", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|_key, value| {
                data.verify_exists(Item::Ethnicity, value);
            });
        });
    }
}

fn validate_name_list(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.verify_exists(Item::Localization, token);
    }
    for (_, block) in vd.integer_blocks() {
        let mut vd = Validator::new(block, data);
        for token in vd.values() {
            data.verify_exists(Item::Localization, token);
        }
    }
}

fn validate_family_name_list(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        for key in token.split('.') {
            data.verify_exists(Item::Localization, &key);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Culture {}

impl DbKind for Culture {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);

        vd.field_item("levy_template", Item::LevyTemplate);
        vd.field_validated_block("nickname", validate_name_list);
        vd.field_validated_block("male_names", validate_name_list);
        vd.field_validated_block("female_names", validate_name_list);
        vd.field_validated_block("family", validate_family_name_list);
    }
}
