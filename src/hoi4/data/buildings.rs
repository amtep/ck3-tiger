use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Building {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Building, Building::add)
}

impl Building {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("buildings") {
            for (item, block) in block.drain_definitions_warn() {
                db.add(Item::Building, item, block, Box::new(Self {}));
            }
        } else if key.is("spawn_points") {
            for (item, block) in block.drain_definitions_warn() {
                db.add(Item::SpawnPoint, item, block, Box::new(SpawnPoint {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `buildings` or `spawn_points` here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl DbKind for Building {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !key.is("infrastructure") {
            data.verify_exists(Item::Localization, key);
        }

        vd.field_item("special_icon", Item::Sprite);

        vd.field_integer("base_cost");
        vd.field_integer("base_cost_conversion");
        vd.field_integer("per_level_extra_cost");
        vd.field_integer("per_controlled_building_extra_cost");
        vd.field_integer("military_production");
        vd.field_integer("general_production");
        vd.field_integer("naval_production");
        vd.field_integer("rocket_production");
        vd.field_integer("rocket_launch_capacity");
        vd.field_integer("air_defence");
        vd.field_integer("land_fort");
        vd.field_integer("naval_fort");
        vd.field_integer("show_on_map");
        vd.field_integer("show_on_map_meshes");
        vd.field_integer("icon_frame");
        vd.field_numeric("value");

        vd.field_bool("show_modifier");
        vd.field_bool("centered");
        vd.field_bool("disable_grow_animation");
        vd.field_bool("has_destroyed_mesh");
        vd.field_bool("need_supply");
        vd.field_bool("need_detection");
        vd.field_bool("allied_build");
        vd.field_bool("always_shown");
        vd.field_bool("infrastructure_construction_effect");
        vd.field_bool("disabled_in_dmz");
        vd.field_bool("only_display_if_exists");
        vd.field_bool("only_costal"); // sic
        vd.field_bool("hide_if_missing_tech");
        vd.field_bool("is_buildable");

        vd.field_bool("air_base");
        vd.field_bool("supply_node");
        vd.field_bool("anti_air");
        vd.field_bool("is_port");
        vd.field_bool("infrastructure");
        vd.field_bool("refinery");
        vd.field_bool("fuel_silo");
        vd.field_bool("radar");
        vd.field_bool("gun_emplacement");
        vd.field_bool("nuclear_reactor");

        vd.field_numeric("damage_factor");
        vd.field_numeric("repair_speed_factor");

        vd.field_item("spawn_point", Item::SpawnPoint);
        vd.field_list_items("specialization", Item::Specialization);
        vd.field_list("tags");
        vd.field_list_items("province_damage_modifiers", Item::Modifier);
        vd.field_list_items("state_damage_modifier", Item::DynamicModifier);

        vd.field_choice("detecting_intel_type", &["navy", "airforce", "army", "civilian"]);

        vd.field_validated_block("level_cap", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("province_max");
            vd.field_integer("state_max");
            vd.field_bool("shares_slots");
            vd.field_choice("group_by", &["special_project_facility", "reactors"]);
            vd.field_item("exclusive_with", Item::Building);
        });

        vd.field_validated_block("country_modifiers", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("enable_for_controllers", Item::CountryTag);
            vd.field_validated_block("modifiers", |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Country, vd);
            });
        });
        vd.field_validated_block("state_modifiers", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::State, vd);
        });

        vd.field_validated_block("missing_tech_loc", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("localization_key", Item::Localization);
            vd.field_item("PROJECT", Item::SpecialProject);
        });

        let mut sc = ScopeContext::new(Scopes::None, key);
        vd.field_trigger("dlc_allowed", Tooltipped::No, &mut sc);
    }
}

#[derive(Clone, Debug)]
pub struct SpawnPoint {}

impl DbKind for SpawnPoint {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("type", &["province", "state"]);
        vd.field_integer("max");
        vd.field_bool("only_costal");
        vd.field_bool("disable_auto_nudging");
    }
}
