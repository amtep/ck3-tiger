use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct NameList {}

impl NameList {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::NameList, key, block, Box::new(Self {}));
    }
}

impl DbKind for NameList {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("mercenary_names", validate_mercenary_names);
        vd.field_validated_block("male_names", validate_name_list);
        vd.field_validated_block("female_names", validate_name_list);
        vd.field_validated_block("dynasty_names", validate_dynasty_names);
        vd.field_validated_block("cadet_dynasty_names", validate_dynasty_names);
        vd.field_item("dynasty_of_location_prefix", Item::Localization);

        vd.field_bool("always_use_patronym");
        vd.field_item("patronym_prefix_male", Item::Localization);
        vd.field_item("patronym_prefix_male_vowel", Item::Localization);
        vd.field_item("patronym_suffix_male", Item::Localization);
        vd.field_item("patronym_prefix_female", Item::Localization);
        vd.field_item("patronym_prefix_female_vowel", Item::Localization);
        vd.field_item("patronym_suffix_female", Item::Localization);

        vd.field_bool("founder_named_dynasties");
        vd.field_item("bastard_dynasty_prefix", Item::Localization);

        // TODO: these should sum to <= 100
        vd.field_integer("father_name_chance");
        vd.field_integer("mat_grf_name_chance");
        vd.field_integer("pat_grf_name_chance");

        vd.field_integer("mother_name_chance");
        vd.field_integer("mat_grm_name_chance");
        vd.field_integer("pat_grm_name_chance");

        vd.field_choice("grammar_transform", &["french"]);
        vd.field_bool("dynasty_name_first");
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

fn validate_mercenary_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
        vd.field_value("coat_of_arms");
    }
}

fn validate_dynasty_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.verify_exists(Item::Localization, token);
    }
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        for token in vd.values() {
            data.verify_exists(Item::Localization, token);
        }
    }
}
