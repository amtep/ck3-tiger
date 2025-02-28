use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PopType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::PopType, PopType::add)
}

impl PopType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PopType, key, block, Box::new(Self {}));
    }
}

impl DbKind for PopType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_validated_block("output_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });

        vd.field_validated_block("count_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });

        vd.field_choice("levy_tier", &["advanced", "basic"]);

        vd.field_item("can_promote_to", Item::PopType);
        vd.field_item("demotes_to", Item::PopType);

        vd.field_bool("is_citizen");
        vd.field_bool("is_slaves");
        vd.field_bool("tribal");
        vd.field_bool("score");
        vd.field_bool("integrated_pop_type_right");
        vd.field_bool("default_pop_right");
        vd.field_bool("block_colonization");
        vd.field_bool("is_linked_with_holdings");

        vd.field_numeric("conquest_demote_chance");
        vd.field_numeric("base_happyness");
        vd.field_numeric("political_weight");
        vd.field_numeric("growing_pop");
        vd.field_numeric("convert");
        vd.field_numeric("assimilate");
        vd.field_numeric("promote");
        vd.field_numeric("demote");
        vd.field_numeric("migrant");
        vd.field_numeric("ui_tier");

        vd.field_validated_block("color", validate_color);

        vd.field_block("modification_display"); // TODO
    }
}
