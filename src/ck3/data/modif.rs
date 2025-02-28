use crate::block::Block;
use crate::ck3::tables::modifs::modif_loc;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, verify_modif_exists};
use crate::report::Severity;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ModifierFormat {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ModifierFormat, ModifierFormat::add)
}

impl ModifierFormat {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ModifierFormat, key, block, Box::new(Self {}));
    }
}

impl DbKind for ModifierFormat {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if let Some("yes") = vd.field_value("hidden").map(Token::as_str) {
            // hidden hence no localization check needed, e.g. `ai_<value>`
        } else {
            let loca = modif_loc(key);
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        verify_modif_exists(key, data, ModifKinds::all(), Severity::Untidy);

        vd.field_integer("decimals");
        vd.field_choice("color", &["good", "neutral", "bad"]);

        vd.field_item("prefix", Item::Localization);
        vd.field_item("suffix", Item::Localization);
        vd.field_item("negative_suffix", Item::Localization);

        vd.field_item("dlc_feature", Item::DlcFeature);

        // Docs say these are in a `format = { ... }` block, but apparently not.
        vd.field_bool("percent");
        vd.field_bool("already_percent");
        vd.field_bool("hidden");
        vd.field_bool("no_difference_sign");
    }
}
