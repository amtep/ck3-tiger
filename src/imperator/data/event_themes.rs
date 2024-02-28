use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct EventTheme {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::EventTheme, EventTheme::add)
}

impl EventTheme {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventTheme, key, block, Box::new(Self {}));
    }
}

impl DbKind for EventTheme {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("icon");
        vd.field_item("soundeffect", Item::Sound);
    }
}
