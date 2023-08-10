use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct EconomicPolicy {}

impl EconomicPolicy {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EconomicPolicy, key, block, Box::new(Self {}));
    }
}

impl DbKind for EconomicPolicy {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.field_validated_block("low", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block("default", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block("high", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block_sc("ai_will_do_low", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block_sc("ai_will_do_high", &mut sc, validate_modifiers_with_base);
        vd.field_choice("war_minimum", &["low", "default", "high"]);
    }
}
