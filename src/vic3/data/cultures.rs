use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Culture {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Culture, Culture::add)
}

impl Culture {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Culture, key, block, Box::new(Self {}));
    }
}

impl DbKind for Culture {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        // The game will log errors if these are not defined.
        let modif = format!("{key}_standard_of_living_modifier_positive");
        data.verify_exists_implied(Item::Modifier, &modif, key);
        let modif = format!("{key}_standard_of_living_modifier_negative");
        data.verify_exists_implied(Item::Modifier, &modif, key);

        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("religion", Item::Religion);
        vd.field_list("traits"); // TODO

        vd.field_list_items("male_common_first_names", Item::Localization);
        vd.field_list_items("female_common_first_names", Item::Localization);
        vd.field_list_items("male_noble_first_names", Item::Localization);
        vd.field_list_items("female_noble_first_names", Item::Localization);
        vd.field_list_items("male_regal_first_names", Item::Localization);
        vd.field_list_items("female_regal_first_names", Item::Localization);
        vd.field_list_items("common_last_names", Item::Localization);
        vd.field_list_items("noble_last_names", Item::Localization);
        vd.field_list_items("regal_last_names", Item::Localization);

        vd.field_list_items("obsessions", Item::Goods);

        vd.field_item("graphics", Item::CultureGraphics);
        vd.field_validated_block("ethnicities", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.numeric_keys(|_, bv| {
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(Item::Ethnicity, token);
                }
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct CultureGraphics {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CultureGraphics, CultureGraphics::add)
}

impl CultureGraphics {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureGraphics, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureGraphics {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        // When it is dropped, this validator will check that the block is empty
        Validator::new(block, data);
    }
}
