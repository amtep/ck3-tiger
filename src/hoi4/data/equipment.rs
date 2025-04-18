use crate::block::{Block, BV};
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::hoi4::data::subunits::validate_equipment_stat;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Equipment {}
#[derive(Clone, Debug)]
pub struct EquipmentSearchFilter {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Equipment, Equipment::add)
}

impl Equipment {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("equipments") || key.is("duplicate_archetypes") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::Equipment, key, block, Box::new(Self {}));
            }
        } else if key.is("search_filters") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::EquipmentSearchFilter, name.clone(), block, Box::new(Self {}));
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

impl DbKind for Equipment {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_integer("year");

        let is_archetype = block.get_field_bool("is_archetype").unwrap_or(false);
        vd.field_bool("is_archetype");
        vd.field_bool("is_buildable");
        vd.field_bool("is_convertable");
        vd.field_bool("can_license");
        vd.field_integer("priority");
        vd.field_item("archetype", Item::Equipment);
        vd.field_item("parent", Item::Equipment);
        vd.field_bool("is_frame");
        vd.field_choice("type", TYPES);

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
        vd.field_trigger_full("can_be_produced", Scopes::Country, Tooltipped::Yes);
        vd.field_trigger_full("can_be_lend_leased", Scopes::Country, Tooltipped::Yes);

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

        vd.field_integer("build_cost_ic");
        vd.field_integer("lend_lease_cost");
        vd.field_numeric("fuel_consumption");

        vd.field_integer("visual_level");

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

        vd.advice_field("max_organization", "it's max_organisation with an s");
        vd.unknown_value_fields(|key, value| {
            validate_equipment_stat(key, value, data);
        });
    }
}

impl DbKind for EquipmentSearchFilter {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_list_items("values", Item::Equipment);
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
