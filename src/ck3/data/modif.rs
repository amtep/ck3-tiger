use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::fileset::FileKind;
use crate::item::Item;
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct ModifierFormat {}

impl ModifierFormat {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ModifierFormat, key, block, Box::new(Self {}));
    }
}

impl DbKind for ModifierFormat {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // TODO: figure out exactly when a localization is needed
        if block.loc.kind == FileKind::Vanilla {
            data.localization.mark_used(key.as_str());
        } else {
            data.localization.verify_exists(key);
        }

        verify_modif_exists(key, data, ModifKinds::all());

        vd.field_integer("decimals");
        vd.field_choice("color", &["good", "neutral", "bad"]);

        vd.field_item("prefix", Item::Localization);
        vd.field_item("suffix", Item::Localization);

        vd.field_item("dlc_feature", Item::DlcFeature);

        // Docs say these are in a `format = { ... }` block, but apparently not.
        vd.field_bool("percent");
        vd.field_bool("already_percent");
        vd.field_bool("hidden");
        vd.field_bool("no_difference_sign");
    }
}
