use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

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
        let loca = format!("{key}_modifier");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_effect_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

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

        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_validated_block("can_build", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_integer("construction_points");
        vd.field_validated_block("construction_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Building, vd);
        });

        vd.field_item("owners", Item::PopType);
        vd.field_numeric_range("economic_contribution", 0.0, 1.0);
        vd.field_numeric("min_raise_to_hire");

        vd.field_bool("naval");
        vd.field_item("canal", Item::CanalType);

        vd.field_numeric("ai_value");
        vd.field_numeric("ai_subsidies_weight");

        vd.field_item("slaves_role", Item::PopType);

        vd.field_list_items("production_methods", Item::ProductionMethodGroup);

        vd.field_validated_block("should_auto_expand", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_choice(
            "city_type",
            &["none", "city", "farm", "mine", "port", "wood"],
        );
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
    }
}
