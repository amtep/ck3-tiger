use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct PlayableDifficultyInfo {}

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
        vd.field_validated_block("is_shown", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}
