use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Modifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Modifier, Modifier::add)
}

impl Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}
