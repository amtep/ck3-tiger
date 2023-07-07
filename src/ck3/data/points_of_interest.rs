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
pub struct PointOfInterest {}

impl PointOfInterest {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PointOfInterest, key, block, Box::new(Self {}));
    }
}

impl DbKind for PointOfInterest {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.field_validated_block("build_province_list", |block, data| {
            let mut sc = sc.clone();
            sc.define_list("provinces", Scopes::Province, key);
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        sc.define_name("province", Scopes::Province, key);
        vd.field_validated_block("on_visit", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}
