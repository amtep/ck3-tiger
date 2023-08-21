use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::context::ScopeContext;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;

#[derive(Clone, Debug)]
pub struct PartyType {}

impl PartyType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PartyType, key, block, Box::new(Self {}));
    }
}

impl DbKind for PartyType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("can_character_belong", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("province", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_value("description");
    }
}