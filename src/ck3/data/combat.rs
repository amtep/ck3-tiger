use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CombatPhaseEvent {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CombatPhaseEvent, CombatPhaseEvent::add)
}

impl CombatPhaseEvent {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CombatPhaseEvent, key, block, Box::new(Self {}));
    }
}

impl DbKind for CombatPhaseEvent {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("combat_side", Scopes::CombatSide, key);

        vd.field_choice("type", &["commander", "knight"]);
        vd.field_validated_block("is_valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block_sc("chance", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block("effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}
