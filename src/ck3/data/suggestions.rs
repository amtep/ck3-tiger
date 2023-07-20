use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;

#[derive(Clone, Debug)]
pub struct Suggestion {}

impl Suggestion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Suggestion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Suggestion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_label");
        data.mark_used(Item::Localization, &loca); // TODO: when is _label needed?
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_click");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("check_create_suggestion", |block, data| {
            // TODO: "only interface effects are allowed"
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("effect", |block, data| {
            let mut sc = sc.clone();
            // TODO: The scope context will contain all scopes passed in the try_create_important_action call
            sc.set_strict_scopes(false);
            // TODO: "only interface effects are allowed"
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("is_valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_item("soundeffect", Item::Sound);

        vd.field_validated_block_sc("weight", &mut sc, validate_modifiers_with_base);

        // TODO: The scope context will contain all scopes passed in the try_create_important_action call
        sc.set_strict_scopes(false);
        vd.field_validated_block_sc("score", &mut sc, validate_modifiers_with_base);
    }
}
