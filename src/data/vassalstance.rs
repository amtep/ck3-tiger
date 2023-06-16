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
pub struct VassalStance {}

impl VassalStance {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::VassalStance, key, block, Box::new(Self {}));
    }
}

impl DbKind for VassalStance {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
        sc.define_name("liege", key.clone(), Scopes::Character);

        vd.field_validated_blocks("is_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_script_value("score", &mut sc);
        vd.field_script_value("heir_score", &mut sc);
    }
}
