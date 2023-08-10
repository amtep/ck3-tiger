use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct EventTheme {}

impl EventTheme {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventTheme, key, block, Box::new(Self {}));
    }
}

impl DbKind for EventTheme {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("icon", Item::File);
        vd.field_item("soundeffect", Item::Sound);
    }
}