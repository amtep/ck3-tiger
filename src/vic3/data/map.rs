use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{untidy, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MapLayer {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MapLayer, MapLayer::add)
}

impl MapLayer {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        // This function warns at `untidy` level because if map layers are actually missing,
        // they will be warned about by their users.
        if key.is("layer") {
            if let Some(name) = block.get_field_value("name") {
                db.add_exact_dup_ok(Item::MapLayer, name.clone(), block, Box::new(Self {}));
            } else {
                untidy(ErrorKey::FieldMissing).msg("unnamed map layer").loc(key).push();
            }
        } else {
            untidy(ErrorKey::UnknownField).weak().msg("unknown map layer item").loc(key).push();
        }
    }
}

impl DbKind for MapLayer {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name"); // this is the key

        vd.field_integer("fade_in");
        vd.field_integer("fade_out");

        // None of the following have examples in vanilla
        vd.field_value("category");
        vd.field_value("masks");
        vd.field_value("visibility_tags");
    }
}

#[derive(Clone, Debug)]
pub struct MapMode {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MapMode, MapMode::add)
}

// LAST UPDATED VIC3 VERSION 1.6.0
// Taken from gfx/map/map_modes/map_modes.txt
const MAP_PAINTING_MODES: &[&str] = &[
    "clout_nationally",
    "colonizable_provinces",
    "countries",
    "country_attitude",
    "country_infamy",
    "country_relations",
    "culture_overview",
    "culture_population",
    "gdp",
    "gdp_nationally",
    "goods_consumption",
    "goods_local_prices",
    "goods_production",
    "ig_strength",
    "literacy",
    "loyalists",
    "market_access",
    "mass_migration_pull",
    "migration",
    "migration_pull",
    "none",
    "player_alliances",
    "player_culture_population",
    "player_goods_potentials",
    "player_military_access",
    "player_population",
    "player_religion_population",
    "player_theaters",
    "pollution",
    "population",
    "province_terrain",
    "radicals",
    "religion_population",
    "standard_of_living",
    "state_gradient",
    "state_mass_migration_pull",
    "states",
    "strategic_regions",
    "technology_progress",
];

// LAST UPDATED VIC3 VERSION 1.5.13
// Taken from gfx/map/map_modes/map_modes.txt
const MAP_NAMES: &[&str] =
    &["countries", "cultures", "markets", "states", "strategic_regions", "theaters"];

impl MapMode {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MapMode, key, block, Box::new(Self {}));
    }
}

impl DbKind for MapMode {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        // This entire item type is undocumented
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_choice("map_painting_mode", MAP_PAINTING_MODES);
        vd.field_choice("map_painting_mode_secondary", MAP_PAINTING_MODES);
        vd.field_choice("map_painting_mode_alternate", MAP_PAINTING_MODES);
        vd.field_choice("map_names", MAP_NAMES);
        vd.field_list("map_markers"); // TODO widget names from gui/map_markers.gui
        vd.field_choice("map_tooltip_offset", &["state", "strategic_region", "theater"]);

        vd.field_bool("has_fog_of_war");
        vd.field_bool("show_occupation");
        vd.field_bool("show_formation_movement_arrows");
        vd.field_bool("is_visible");
        vd.field_bool("is_visible_to_countryless_observer");
        vd.field_item("gradient_border_settings", Item::GradientBorderSettings);

        vd.field_item("soundeffect", Item::Sound);

        vd.field_bool("use_mapmode_textures");
        vd.field_bool("primary_red_as_gradient_border");
        // TODO: not sure whether alternate_color blue and alpha are valid
        for field in &[
            "primary_color_red",
            "primary_color_green",
            "primary_color_blue",
            "primary_color_alpha",
            "secondary_color_red",
            "secondary_color_green",
            "secondary_color_blue",
            "secondary_color_alpha",
            "alternate_color_red",
            "alternate_color_green",
            "alternate_color_blue",
            "alternate_color_alpha",
        ] {
            if let Some(token) = vd.field_value(field) {
                let pathname = format!("gfx/map/map_painting/{token}");
                data.verify_exists_implied(Item::File, &pathname, token);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MapInteractionType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MapInteractionType, MapInteractionType::add)
}

impl MapInteractionType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MapInteractionType, key, block, Box::new(Self {}));
    }
}

impl DbKind for MapInteractionType {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        // This entire item type is undocumented
        let mut vd = Validator::new(block, data);

        vd.field_item("mapmode", Item::MapMode);
        vd.field_item("clicksound", Item::Sound);
        vd.field_bool("show_interaction_text_on_click");
    }
}

#[derive(Clone, Debug)]
pub struct GradientBorderSettings {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::GradientBorderSettings, GradientBorderSettings::add)
}

impl GradientBorderSettings {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GradientBorderSettings, key, block, Box::new(Self {}));
    }
}

impl DbKind for GradientBorderSettings {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        // This entire item type is undocumented.
        let mut vd = Validator::new(block, data);

        vd.field_numeric("gradient_water_borders_zoom");
        vd.multi_field_validated_block("gradient_parameters", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("zoom_step");
            vd.field_numeric("gradient_alpha_inside");
            vd.field_numeric("gradient_alpha_outside");
            vd.field_numeric("gradient_width");
            vd.field_numeric("gradient_color_mult");
            vd.field_numeric("edge_width");
            vd.field_numeric("edge_sharpness");
            vd.field_numeric("edge_alpha");
            vd.field_numeric("edge_color_mult");
            vd.field_numeric("before_lighting_blend");
            vd.field_numeric("after_lighting_blend");
        });
    }
}

#[derive(Clone, Debug)]
pub struct MapNotificationType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MapNotificationType, MapNotificationType::add)
}

impl MapNotificationType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MapNotificationType, key, block, Box::new(Self {}));
    }
}

impl DbKind for MapNotificationType {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("message", Item::Localization);
        vd.field_item("widget", Item::WidgetName);
        vd.field_integer("max_height");
        vd.field_item("sound", Item::Sound);
    }
}