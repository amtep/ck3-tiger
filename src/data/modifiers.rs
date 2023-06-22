use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Modifier {}

impl Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // There are {key} and {key}_desc locas but both are optional
        data.item_used(Item::Localization, key.as_str());
        let loca = format!("{key}_desc");
        data.item_used(Item::Localization, &loca);

        // icon is also optional
        if let Some(icon) = vd.field_value("icon") {
            if let Some(icon_path) =
                data.get_defined_string_warn(icon, "NGameIcons|STATICMODIFIER_ICON_PATH")
            {
                let pathname = format!("{icon_path}/{icon}.dds");
                data.fileset.verify_exists_implied(&pathname, icon);
            }
        }

        vd.field_bool("stacking");
        vd.field_bool("hide_effects");
        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}
