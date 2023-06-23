use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug)]
pub struct Hook {}

impl Hook {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Hook, key, block, Box::new(Self {}));
    }
}

impl DbKind for Hook {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
        sc.define_name("target", Scopes::Character, key.clone());

        data.verify_exists(Item::Localization, key);

        vd.field_integer("expiration_days");
        vd.field_bool("strong");
        vd.field_bool("requires_secret");

        vd.field_validated_block("on_used", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}
