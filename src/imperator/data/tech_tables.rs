use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;
use crate::modif::{validate_modifs, ModifKinds};

#[derive(Clone, Debug)]
pub struct TechnologyTable {}

impl TechnologyTable {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TechnologyTable, key, block, Box::new(Self {}));
    }
}

impl DbKind for TechnologyTable {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("icon");
        vd.field_choice("skill", &["martial", "charisma", "zeal", "finesse"]);

        validate_modifs(block, data, ModifKinds::Country, vd);
    }
}