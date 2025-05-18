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
pub struct PointOfInterest {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::PointOfInterest, PointOfInterest::add)
}

impl PointOfInterest {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PointOfInterest, key, block, Box::new(Self {}));
    }
}

impl DbKind for PointOfInterest {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_effect_builder("build_province_list", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_list("provinces", Scopes::Province, key);
            sc
        });

        vd.field_effect_builder("on_visit", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("province", Scopes::Province, key);
            sc
        });
    }
}
