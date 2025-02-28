use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, verify_modif_exists};
use crate::report::Severity;
use crate::token::Token;
use crate::validator::Validator;
use crate::vic3::tables::modifs::modif_loc;

/// Equivalent to CK3's `Item::ModifierFormat` in the `ck3::data::modif` module.

#[derive(Clone, Debug)]
pub struct ModifierTypeDefinition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ModifierTypeDefinition, ModifierTypeDefinition::add)
}

impl ModifierTypeDefinition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ModifierTypeDefinition, key, block, Box::new(Self {}));
    }
}

impl DbKind for ModifierTypeDefinition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let (loca, loca_desc) = modif_loc(key, data);
        data.verify_exists_implied(Item::Localization, &loca, key);
        data.verify_exists_implied(Item::Localization, &loca_desc, key);

        verify_modif_exists(key, data, ModifKinds::all(), Severity::Untidy);

        vd.field_integer("decimals");
        vd.field_choice("color", &["neutral", "good", "bad"]);
        vd.field_bool("percent");
        vd.field_bool("boolean");
        vd.field_validated_block("game_data", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("ai_value");
            vd.field_item("translate", Item::ModifierTypeDefinition);
            vd.field_list("tags");
            vd.advice_field("type_set", "docs say type_set but it's tags");
        });

        vd.field_item("prefix", Item::Localization);
        vd.replaced_field("postfix", "suffix");
        vd.field_item("suffix", Item::Localization);
    }
}
