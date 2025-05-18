use crate::block::Block;
use crate::ck3::data::activities::validate_tes;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct TravelOption {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::TravelOption, TravelOption::add)
}

impl TravelOption {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TravelOption, key, block, Box::new(Self {}));
    }
}

impl DbKind for TravelOption {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("is_valid", Tooltipped::Yes, &mut sc);

        vd.field_validated_block_sc("cost", &mut sc, validate_cost);

        vd.multi_field_validated_block("travel_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::TravelPlan, vd);
        });

        vd.multi_field_validated_block("owner_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        sc.define_name("travel_speed", Scopes::Value, key);
        sc.define_name("travel_safety", Scopes::Value, key);
        vd.field_effect("on_applied_effect", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_travel_end_effect", Tooltipped::No, &mut sc);

        let mut sc = ScopeContext::new(Scopes::TravelPlan, key);
        sc.define_name("highest_future_danger_value", Scopes::Value, key);
        vd.field_script_value_no_breakdown("ai_will_do", &mut sc);

        vd.field_validated_block("travel_entourage_selection", |block, data| {
            validate_tes(key, block, data, false);
        });
    }
}
