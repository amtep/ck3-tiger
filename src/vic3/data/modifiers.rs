use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Modifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Modifier, Modifier::add)
}

impl Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // The standard of living defines for cultures are required but don't need localizations
        if !key.as_str().ends_with("_standard_of_living_modifier_positive")
            && !key.as_str().ends_with("standard_of_living_modifier_negative")
        {
            data.verify_exists(Item::Localization, key);
        }
        vd.field_item("icon", Item::File);

        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}
