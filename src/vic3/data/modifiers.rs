use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Vic3Modifier {}

impl Vic3Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Vic3Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        vd.field_item("icon", Item::File);

        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}
