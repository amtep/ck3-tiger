use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Unit {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Unit, Unit::add)
}

impl Unit {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Unit, key, block, Box::new(Self {}));
    }
}

impl DbKind for Unit {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::State, key);

        vd.field_bool("default");
        vd.field_bool("army");
        vd.field_bool("assault");
        vd.field_bool("support");
        vd.field_bool("reduces_road_building_cost");
        vd.field_bool("is_flank");
        vd.field_bool("legions");
        vd.field_bool("enable");

        vd.field_validated_block("allow", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_choice("levy_tier", &["advanced", "basic", "none"]);
        vd.field_choice("category", &["light", "medium", "heavy"]);
        
        vd.field_numeric("watercrossing_negation");
        vd.field_numeric("siege_impact");
        vd.field_numeric("build_time");
        vd.field_numeric("maneuver");
        vd.field_numeric("movement_speed");
        vd.field_numeric("attrition_weight");
        vd.field_numeric("morale_damage_taken");
        vd.field_numeric("attrition_loss");
        vd.field_numeric("ai_max_percentage");
        vd.field_numeric("food_consumption");
        vd.field_numeric("food_storage");
        vd.field_numeric("merc_cohorts_required");
        vd.field_numeric("setup_fraction");

        vd.field_numeric("morale_damage_done");
        vd.field_numeric("port_level");
        vd.field_numeric("outside_of_naval_range_attrition");
        vd.field_numeric("strength_damage_done");
        vd.field_numeric("strength_damage_taken");

        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Unit, key);
            value.expect_number();
        });

        vd.field_validated_block("build_cost", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("gold");
            vd.field_numeric("manpower");
        });
    }
}
