use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;
use crate::vic3::data::production_methods::ProductionMethodGroup;

#[derive(Clone, Debug)]
pub struct BuildingType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::BuildingType, BuildingType::add)
}

impl BuildingType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BuildingType, key, block, Box::new(Self {}));
    }

    #[allow(clippy::unused_self)]
    pub fn validate_production_method(
        &self,
        pm: &Token,
        building: &Token,
        block: &Block,
        data: &Everything,
    ) {
        if let Some(groups) = block.get_field_list("production_method_groups") {
            for group in groups {
                if let Some((_, block, group_item)) = data
                    .get_item::<ProductionMethodGroup>(Item::ProductionMethodGroup, group.as_str())
                {
                    if group_item.contains_production_method(pm, block, data) {
                        return;
                    }
                }
            }
        }
        let msg = format!("production method `{pm}` not valid for `{building}`");
        err(ErrorKey::Validation).msg(msg).loc(pm).push();
    }

    pub fn is_discoverable(block: &Block, data: &Everything) -> bool {
        let mut seen = Vec::new();
        if let Some(group) = block.get_field_value("building_group") {
            let mut group = group.as_str();
            loop {
                seen.push(group);
                if let Some((_, block)) = data.get_key_block(Item::BuildingGroup, group) {
                    if block.get_field_bool("discoverable_resource").unwrap_or(false) {
                        return true;
                    }
                    if let Some(parent) = block.get_field_value("parent_group") {
                        if seen.contains(&parent.as_str()) {
                            let msg = "cycle in building groups";
                            let info =
                                format!("building group `{parent}` ends up being its own parent");
                            err(ErrorKey::Loop).msg(msg).info(info).loc(parent).push();
                            break;
                        }
                        group = parent.as_str();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        false
    }
}

impl DbKind for BuildingType {
    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if property == "max_level" {
            return block.get_field_bool("has_max_level").unwrap_or(false);
        }
        false
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        let mut building_sc = ScopeContext::new(Scopes::Building, key);
        let mut state_sc = ScopeContext::new(Scopes::State, key);

        data.verify_exists(Item::Localization, key);
        if block.get_field_bool("expandable").unwrap_or(true) {
            let loca = format!("{key}_lens_option");
            // TODO: figure out when this is required
            data.mark_used(Item::Localization, &loca);
        }

        if BuildingType::is_discoverable(block, data) {
            let loca = format!("{key}_discovered_resource");
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

        if block.get_field_bool("has_max_level").unwrap_or(false) {
            let modif = format!("state_{key}_max_level_add");
            data.verify_exists_implied(Item::ModifierTypeDefinition, &modif, key);
        }

        vd.replaced_field("recruits_combat_unit", "recruits_combat_units = yes");
        vd.field_bool("recruits_combat_units");

        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_validated_block("can_build", |block, data| {
            validate_trigger(block, data, &mut state_sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_build_government", |block, data| {
            validate_trigger(block, data, &mut state_sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_build_private", |block, data| {
            validate_trigger(block, data, &mut state_sc, Tooltipped::Yes);
        });

        vd.field_integer("construction_points");
        vd.field_validated_block("construction_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Building, vd);
        });
        vd.field_validated("required_construction", validate_non_dynamic_script_value);

        vd.field_item("owners", Item::PopType);
        vd.field_numeric_range("economic_contribution", 0.0..=1.0);
        vd.field_numeric("min_raise_to_hire");

        vd.field_bool("naval");
        vd.field_item("canal", Item::CanalType);

        vd.field_script_value_rooted("ai_value", Scopes::State);
        vd.field_numeric("ai_subsidies_weight");
        vd.advice_field(
            "ai_privatization_desire",
            "docs say ai_privatization_desire but it's ai_nationalization_desire",
        );
        vd.field_script_value("ai_nationalization_desire", &mut sc);

        vd.field_item("slaves_role", Item::PopType);

        // docs say production_methods
        vd.field_list_items("production_method_groups", Item::ProductionMethodGroup);

        vd.field_validated_block("should_auto_expand", |block, data| {
            validate_trigger(block, data, &mut building_sc, Tooltipped::Yes);
        });

        vd.field_choice("city_type", &["none", "city", "farm", "mine", "port", "wood"]);
        vd.field_bool("generates_residences");
        vd.field_item("terrain_manipulator", Item::TerrainManipulator);
        vd.field_integer("levels_per_mesh");
        vd.field_integer("residence_points_per_level");
        vd.field_bool("override_centerpiece_mesh");
        vd.field_bool("statue");
        if block.field_value_is("override_centerpiece_mesh", "yes")
            || block.field_value_is("statue", "yes")
        {
            vd.req_field("centerpiece_mesh_weight");
        }
        if block.field_value_is("override_centerpiece_mesh", "yes")
            && block.field_value_is("statue", "yes")
        {
            let msg = "override_centerpiece_mesh and statue are mutually exclusive";
            err(ErrorKey::Validation).msg(msg).loc(block).push();
        }
        vd.field_integer("centerpiece_mesh_weight");

        vd.field_list("meshes"); // TODO
        vd.field_list("entity_not_constructed"); // TODO
        vd.field_list("entity_under_construction"); // TODO
        vd.field_list("entity_constructed"); // TODO
        vd.field_value("locator"); // TODO
        vd.field_value("lens"); // TODO

        vd.field_choice("ownership_type", &["no_ownership", "self", "other"]);
        vd.field_item("background", Item::File);

        // undocumented

        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut state_sc, Tooltipped::No);
        });

        vd.field_validated_block("city_gfx_interactions", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_bool("clear_collision_mesh_area");
            vd.field_bool("clear_size_area");
            vd.field_integer("size");
        });

        vd.field_bool("cannot_switch_owner");

        vd.field_validated_block("investment_scores", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                let mut vd = Validator::new(block, data);
                vd.field_item("group", Item::BuildingGroup);
                vd.field_script_value_full("score", Scopes::Country, false);
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct BuildingGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::BuildingGroup, BuildingGroup::add)
}

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
        vd.field_bool("always_self_owning"); // undocumented

        // TODO: are category and land_usage really both valid?
        vd.field_choice("category", &["urban", "rural", "development"]);
        vd.field_choice("land_usage", &["urban", "rural"]);

        // TODO: check if it's a building in this group
        vd.field_item("default_building", Item::BuildingType);
        vd.field_value("lens"); // TODO

        vd.field_numeric("cash_reserves_max");
        vd.field_numeric("urbanization");

        vd.field_numeric("hiring_rate");
        vd.field_numeric("min_hiring_rate");
        vd.field_numeric("max_hiring_rate");
        vd.field_numeric("proportionality_limit");
        vd.field_bool("hires_unemployed_only");
        vd.field_bool("ignores_productivity_when_hiring");

        vd.field_validated_block_rooted(
            "should_auto_expand",
            Scopes::Building,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            },
        );

        // undocumented fields

        vd.field_numeric("economy_of_scale_ai_factor");
        vd.field_numeric("foreign_investment_ai_factor");
        vd.field_numeric("infrastructure_usage_per_level");
        vd.field_numeric("min_productivity_to_hire");
        vd.field_bool("fired_pops_become_radical");
        vd.field_bool("is_military");
        vd.field_bool("is_government_funded");
        vd.field_bool("owns_other_buildings");
        vd.field_bool("is_shown_in_outliner");
    }
}
