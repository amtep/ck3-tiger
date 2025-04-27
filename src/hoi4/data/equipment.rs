use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::hoi4::data::subunits::validate_equipment_stat;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Equipment {}
#[derive(Clone, Debug)]
pub struct EquipmentGroup {}
#[derive(Clone, Debug)]
pub struct EquipmentModule {}
#[derive(Clone, Debug)]
pub struct EquipmentSearchFilter {}
#[derive(Clone, Debug)]
pub struct EquipmentUpgrade {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Equipment, Equipment::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::EquipmentGroup, EquipmentGroup::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::EquipmentModule, EquipmentModule::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::EquipmentUpgrade, EquipmentUpgrade::add)
}

impl Equipment {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("equipments") || key.is("duplicate_archetypes") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::Equipment, key, block, Box::new(Self {}));
            }
        } else if key.is("search_filters") {
            if let Some(name) = block.get_field_value("name") {
                db.add(
                    Item::EquipmentSearchFilter,
                    name.clone(),
                    block,
                    Box::new(EquipmentSearchFilter {}),
                );
            } else {
                let msg = "search filter without `name` field";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl EquipmentGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EquipmentGroup, key, block, Box::new(Self {}));
    }
}

impl EquipmentModule {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("equipment_modules") {
            for (key, block) in block.drain_definitions_warn() {
                // TODO: validate these limit clauses somehow. Where/how to store them?
                if !key.is("limit") {
                    db.add(Item::EquipmentModule, key, block, Box::new(Self {}));
                }
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl EquipmentUpgrade {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("upgrades") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::EquipmentUpgrade, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for Equipment {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("abbreviation");
        vd.field_item("derived_variant_name", Item::Equipment);
        vd.field_integer("year");

        let is_archetype = block.get_field_bool("is_archetype").unwrap_or(false);
        vd.field_bool("is_archetype");
        vd.field_bool("is_buildable");
        vd.field_bool("is_convertable");
        vd.field_bool("can_license");
        vd.field_bool("carrier_capable");
        vd.field_bool("mountaineers");
        vd.field_numeric("default_carrier_composition_weight");
        vd.field_integer("priority");
        vd.field_item("archetype", Item::Equipment);
        vd.field_item("parent", Item::Equipment);
        vd.field_bool("is_frame");
        vd.field_choice("type", TYPES);
        vd.field_value("type_override"); // TODO: where are these defined?
        vd.field_value("model"); // TODO: what is this?
        vd.field_item("alias", Item::Equipment);
        vd.field_item("derived_variant_name", Item::Equipment);

        if is_archetype {
            if let Some(picture) = vd.field_value("picture") {
                let pathname = format!("gfx/interface/archetypes/{picture}.dds");
                data.verify_exists_implied(Item::File, &pathname, picture);
            }
        } else {
            vd.ban_field("picture", || "archetypes");
        }
        // TODO: what is this?
        vd.field_value("sprite");

        vd.field_choice("group_by", &["type", "archetype"]);
        vd.field_choice("interface_category", INTERFACE_CATEGORIES);
        vd.field_bool("active");
        vd.field_bool("one_use_only");

        vd.field_integer("max_military_factories");
        vd.field_integer("max_dockyard_factories");
        vd.field_choice("ai_type", AI_TYPES);

        vd.field_list_items("upgrades", Item::EquipmentUpgrade);
        vd.field_trigger("can_be_produced", Scopes::Country, Tooltipped::Yes);
        vd.field_trigger("can_be_lend_leased", Scopes::Country, Tooltipped::Yes);

        // TODO: validate these when equipment modules are in
        vd.field("module_slots");
        vd.field_block("module_count_limit");
        vd.field_block("default_modules");

        vd.field_validated_block("resources", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::Resource, |_, mut vd| {
                vd.integer();
            });
        });
        vd.field_integer("manpower");

        vd.field_validated_block("variant_names", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::EquipmentBonusType, |_, mut vd| {
                vd.item(Item::Equipment);
            });
        });
        vd.field_integer("air_map_icon_frame");
        vd.field_integer("interface_overview_category_index");
        vd.field_item("substitute", Item::Equipment);

        vd.field_bool("offensive_weapons");
        vd.field_integer("build_cost_ic");
        vd.field_integer("lend_lease_cost");
        vd.field_numeric("fuel_consumption");
        vd.field_numeric("naval_supremacy_factor");
        vd.field_numeric("naval_weather_penalty_factor");

        vd.field_integer("visual_level");
        vd.field_integer("visual_tech_level_addition");

        // These can be either a list or a single value
        for field in &["allow_mission_type", "forbid_mission_type"] {
            vd.field_validated(field, |bv, _| match bv {
                BV::Value(value) => validate_mission_type(value),
                BV::Block(block) => {
                    for value in block.iter_values_warn() {
                        validate_mission_type(value);
                    }
                }
            });
        }

        vd.field_bool("supply_truck");

        vd.advice_field("max_organization", "it's max_organisation with an s");
        vd.unknown_value_fields(|key, value| {
            validate_equipment_stat(key, value, data);
        });
    }
}

impl DbKind for EquipmentGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        vd.field_item("description", Item::Localization);

        vd.field_validated_list("equipment_type", |value, _| {
            if !data.item_exists(Item::Equipment, value.as_str())
                && !data.item_exists(Item::EquipmentCategory, value.as_str())
                && !data.item_exists(Item::EquipmentGroup, value.as_str())
            {
                let msg = format!(
                    "item {value} not found as equipment, equipment category, or equipment group",
                );
                err(ErrorKey::MissingItem).msg(msg).loc(value).push();
            }
        });
    }
}

impl DbKind for EquipmentModule {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(category) = block.get_field_value("category") {
            db.add_flag(Item::EquipmentModuleCategory, category.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("abbreviation");
        vd.field_value("category");

        vd.field_item("sfx", Item::SoundEffect);

        vd.field_choice("allow_equipment_type", TYPES);
        vd.field_choice("forbid_equipment_type", TYPES);
        vd.field_choice("forbid_equipment_type_exact_match", TYPES);
        vd.field_validated_block(
            "forbid_equipment_type_exact_match_for_category",
            |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validate_item_key_values(Item::EquipmentModuleCategory, |_, mut vd| {
                    vd.choice(TYPES);
                });
            },
        );

        vd.field_item("parent", Item::EquipmentModule);

        vd.field_validated_block("allowed_module_categories", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                // TODO: validate the keys
                let mut vd = Validator::new(block, data);
                for value in vd.values() {
                    data.verify_exists(Item::EquipmentModuleCategory, value);
                }
            });
        });

        for field in &["add_stats", "multiply_stats", "add_average_stats"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
                    vd.numeric();
                });
            });
        }

        vd.multi_field_validated_block("can_convert_from", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("module", Item::EquipmentModule);
            vd.field_numeric("convert_cost_ic");
        });

        vd.field_validated_block("build_cost_resources", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::Resource, |_, mut vd| {
                vd.integer();
            });
        });

        vd.field_list("critical_parts");

        vd.field_numeric("dismantle_cost_ic");
        vd.field_integer("xp_cost");
    }
}

impl DbKind for EquipmentSearchFilter {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_list_items("values", Item::Equipment);
    }
}

impl DbKind for EquipmentUpgrade {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        // TODO: is the _desc required?
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("abbreviation");

        vd.field_integer("max_level");
        vd.field_choice("cost", &["air", "land", "naval"]);
        vd.field_validated_block("linear_cost", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("cost_by_level");
            vd.field_integer("cost_by_level_for_licensed_equipment");
        });
        vd.field_validated_key_block("level_requirements", |key, block, data| {
            let mut vd = Validator::new(block, data);
            vd.integer_keys(|_, bv| {
                if let Some(block) = bv.expect_block() {
                    let mut sc = ScopeContext::new(Scopes::Country, key);
                    validate_trigger(block, data, &mut sc, Tooltipped::Yes);
                }
            });
        });
        vd.field_validated_block("resource_cost_thresholds", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.integer_keys(|_, bv| {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    vd.validate_item_key_values(Item::Resource, |_, mut vd| {
                        vd.integer();
                    });
                }
            });
        });

        vd.field_validated_block("add_stats", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
                vd.numeric();
            });
        });

        vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
            vd.numeric();
        });
    }
}

fn validate_mission_type(token: &Token) {
    if !MISSION_TYPES.contains(&token.as_str()) {
        let msg = format!("expected one of {}", MISSION_TYPES.join(", "));
        err(ErrorKey::Choice).msg(msg).loc(token).push();
    }
}

const TYPES: &[&str] = &[
    "armor",
    "infantry",
    "motorized",
    "mechanized",
    "anti_air",
    "artillery",
    "anti_tank",
    "rocket",
    "support",
    "capital_ship",
    "submarine",
    "screen_ship",
    "carrier",
    "convoy",
    "naval_transport",
    "air_transport",
    "figher",
    "cas",
    "interceptor",
    "tactical_bomber",
    "strategic_bomber",
    "naval_bomber",
    "missile",
    "suicide",
];

// TODO: does this really allow all SubUnit items?
const AI_TYPES: &[&str] =
    &["cv_fighter", "cv_interceptor", "cv_cas", "cv_naval_bomber", "cv_suicide", "heavy_fighter"];

const INTERFACE_CATEGORIES: &[&str] = &[
    "interface_category_air",
    "interface_category_air",
    "interface_category_armor",
    "interface_category_armor",
    "interface_category_armor",
    "interface_category_armor",
    "interface_category_capital_ships",
    "interface_category_land",
    "interface_category_land",
    "interface_category_other_ships",
    "interface_category_other_ships",
    "interface_category_screen_ships",
];

const MISSION_TYPES: &[&str] = &[
    "barrage_mission",
    "strategic_bomber",
    "training",
    "air_superiority",
    "interception",
    "cas",
    "attack_logistics",
    "naval_bomber",
    "port_strike",
    "naval_patrol",
    "recon",
    "naval_mines_planting",
    "naval_mines_sweeping",
    "naval_kamikaze",
    "drop_nuke",
    "paradrop",
    "air_supply",
    "sam_mission",
];
