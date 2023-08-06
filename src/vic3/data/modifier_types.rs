use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::report::Severity;
use crate::token::Token;
use crate::validator::Validator;

/// Equivalent to CK3's `Item::ModifierFormat` in the `ck3::data::modif` module.

#[derive(Clone, Debug)]
pub struct ModifierType {}

impl ModifierType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ModifierType, key, block, Box::new(Self {}));
    }
}

impl DbKind for ModifierType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("modifier_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("modifier_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        verify_modif_exists(key, data, ModifKinds::all(), Severity::Untidy);

        vd.field_bool("neutral");
        vd.field_bool("good");
        vd.field_bool("percent");
        vd.field_bool("boolean");
        vd.field_integer("num_decimals");
        vd.field_numeric("ai_value");

        vd.field_item("prefix", Item::Localization);
        vd.field_item("postfix", Item::Localization);

        vd.field_item("translate", Item::ModifierType);
    }
}
