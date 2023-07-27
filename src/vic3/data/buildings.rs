use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::scriptvalue::validate_non_dynamic_scriptvalue;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct BuildingType {}

impl BuildingType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BuildingType, key, block, Box::new(Self {}));
    }
}

impl DbKind for BuildingType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_item("building_group", Item::BuildingGroup);
        vd.field_item("texture", Item::File);

        vd.field_bool("buildable");
        vd.field_bool("expandable");
        vd.field_bool("downsizeable");
        vd.field_bool("unique");
        vd.field_bool("has_max_level");
        vd.field_bool("ignore_stateregion_max_level");
        vd.field_bool("enable_air_connection");
        vd.field_bool("port");

        vd.field_item("recruits_combat_unit", Item::CombatUnit); // undocumented

        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_validated_block("can_build", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_build_government", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_build_private", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_integer("construction_points");
        vd.field_validated_block("construction_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Building, vd);
        });
        vd.field_validated("required_construction", validate_non_dynamic_scriptvalue);

        vd.field_item("owners", Item::PopType);
        vd.field_numeric_range("economic_contribution", 0.0, 1.0);
        vd.field_numeric("min_raise_to_hire");

        vd.field_bool("naval");
        vd.field_item("canal", Item::CanalType);

        vd.field_numeric("ai_value");
        vd.field_numeric("ai_subsidies_weight");

        vd.field_item("slaves_role", Item::PopType);

        // docs say production_methods
        vd.field_list_items("production_method_groups", Item::ProductionMethodGroup);

        vd.field_validated_block("should_auto_expand", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_choice("city_type", &["none", "city", "farm", "mine", "port", "wood"]);
        vd.field_bool("generates_residences");
        vd.field_item("terrain_manipulator", Item::TerrainManipulator);
        vd.field_integer("levels_per_mesh");
        vd.field_integer("residence_points_per_level");
        vd.field_bool("override_centerpiece_mesh");
        vd.field_integer("centerpiece_mesh_weight");

        vd.field_list("meshes"); // TODO
        vd.field_list("entity_not_constructed"); // TODO
        vd.field_list("entity_under_construction"); // TODO
        vd.field_list("entity_constructed"); // TODO
        vd.field_value("locator"); // TODO
        vd.field_value("lens"); // TODO

        // undocumented

        vd.field_validated_key_block("possible", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::State, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("city_gfx_interactions", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_bool("clear_collision_mesh_area");
            vd.field_bool("clear_size_area");
            vd.field_integer("size");
        });
    }
}

#[derive(Clone, Debug)]
pub struct BuildingGroup {}

impl BuildingGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BuildingGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for BuildingGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("parent_group", Item::BuildingGroup);
        vd.field_item("texture", Item::File);

        vd.field_bool("always_possible");
        vd.field_bool("economy_of_scale");
        vd.field_bool("is_subsistence");
        vd.field_bool("auto_place_buildings");
        vd.field_bool("capped_by_resources");
        vd.field_bool("discoverable_resource");
        vd.field_bool("depletable_resource");
        vd.field_bool("can_use_slaves");
        vd.field_bool("inheritable_construction");
        vd.field_bool("stateregion_max_level");
        vd.field_bool("pays_taxes"); // undocumented
        vd.field_bool("created_by_trade_routes"); // undocumented
        vd.field_bool("subsidized"); // undocumented

        // TODO: are category and land_usage really both valid?
        vd.field_choice("category", &["urban", "rural", "development"]);
        vd.field_choice("land_usage", &["urban", "rural"]);

        // TODO: check if it's a building in this group
        vd.field_item("default_building", Item::BuildingType);
        vd.field_value("lens"); // TODO

        vd.field_numeric("cash_reserves_max");
        vd.field_numeric("urbanization");

        vd.field_numeric("hiring_rate");
        vd.field_numeric("proportionality_limit");
        vd.field_bool("hires_unemployed_only");

        vd.field_validated_block_rooted(
            "should_auto_expand",
            Scopes::Building,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            },
        );

        // undocumented fields

        vd.field_numeric("economy_of_scale_ai_factor");
        vd.field_numeric("infrastructure_usage_per_level");
        vd.field_bool("fired_pops_become_radical");
        vd.field_bool("is_military");
        vd.field_bool("is_government_funded");
    }
}
