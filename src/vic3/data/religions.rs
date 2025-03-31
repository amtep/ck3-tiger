use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Religion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Religion, Religion::add)
}

impl Religion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Religion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Religion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        // The game will log errors if these are not defined.
        let modif = format!("{key}_standard_of_living_modifier_positive");
        data.verify_exists_implied(Item::Modifier, &modif, key);
        let modif = format!("{key}_standard_of_living_modifier_negative");
        data.verify_exists_implied(Item::Modifier, &modif, key);

        vd.field_item("texture", Item::File);
        vd.field_list_items("traits", Item::DiscriminationTrait);
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_list_items("taboos", Item::Goods);
    }
}
