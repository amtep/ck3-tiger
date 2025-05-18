use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PlayableDifficultyInfo {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::PlayableDifficultyInfo, PlayableDifficultyInfo::add)
}

impl PlayableDifficultyInfo {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PlayableDifficultyInfo, key, block, Box::new(Self {}));
    }
}

impl DbKind for PlayableDifficultyInfo {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
    }
}
