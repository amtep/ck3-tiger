use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::data::activities::validate_tes;
use crate::db::{Db, DbKind};
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_cost;

#[derive(Clone, Debug)]
pub struct TravelOption {}

impl TravelOption {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TravelOption, key, block, Box::new(Self {}));
    }
}

impl DbKind for TravelOption {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.field_validated_block("is_shown", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("is_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block_sc("cost", &mut sc, validate_cost);

        vd.field_validated_blocks("travel_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::TravelPlan, vd);
        });

        vd.field_validated_blocks("owner_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        sc.define_name("travel_speed", Scopes::Value, key);
        sc.define_name("travel_safety", Scopes::Value, key);
        vd.field_validated_block("on_applied_effect", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_travel_end_effect", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        let mut sc = ScopeContext::new(Scopes::TravelPlan, key);
        sc.define_name("highest_future_danger_value", Scopes::Value, key);
        vd.field_script_value_no_breakdown("ai_will_do", &mut sc);

        vd.field_validated_block("travel_entourage_selection", |block, data| {
            validate_tes(key, block, data, false);
        });
    }
}
