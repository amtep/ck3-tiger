use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::report::{untidy, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::util::SmartJoin;

#[derive(Clone, Debug)]
pub struct Terrain {}

impl Terrain {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Terrain, key, block, Box::new(Self {}));
    }
}

impl DbKind for Terrain {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        vd.field_items("label", Item::TerrainLabel);

        vd.field_script_value_rooted("weight", Scopes::Province);
        vd.field_validated_key_blocks("textures", |key, block, data| {
            let mut vd = Validator::new(block, data);
            vd.validated_blocks(|block, data| {
                let mut vd = Validator::new(block, data);
                let mut sc = ScopeContext::new(Scopes::Province, key);
                sc.define_name("state", Scopes::State, key);
                sc.define_name("region", Scopes::StateRegion, key);
                vd.field_script_value("weight", &mut sc);
                vd.field_item("path", Item::File);
                vd.field_item("effect", Item::Entity);
            });
        });

        vd.field_numeric("combat_width");
        vd.field_numeric("risk");

        vd.field_validated_block("materials", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::TerrainMaterial, key);
                value.expect_number();
            });
        });
        vd.field_numeric("pollution_mask_strength");
        vd.field_numeric("devastation_mask_strength");

        // deliberately not validated because it's only for debug
        vd.field("debug_color");

        // undocumented
        vd.field_item("created_material", Item::TerrainMaterial);
    }
}

#[derive(Clone, Debug)]
pub struct TerrainLabel {}

impl TerrainLabel {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TerrainLabel, key, block, Box::new(Self {}));
    }
}

impl DbKind for TerrainLabel {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        // TerrainLabel might have to be renamed if there are more options here
        vd.field_choice("type", &["terrain"]);
        vd.field_validated_list("modifiers", |token, data| {
            verify_modif_exists(token, data, ModifKinds::all());
        });
    }
}

#[derive(Clone, Debug)]
pub struct TerrainManipulator {}

impl TerrainManipulator {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        // Skip the common/terrain_manipulators/provinces/ files.
        // TODO: should be a way to tell fileset to skip subdirectories
        if key.loc.pathname().components().count() > 3 {
            return;
        }
        db.add(Item::TerrainManipulator, key, block, Box::new(Self {}));
    }
}

impl DbKind for TerrainManipulator {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("created_terrain", Item::Terrain);
        vd.field_item("terrain_mask", Item::TerrainMask);
        vd.field_item("preferred_terrain", Item::Terrain);
        // Same as in BuildingType
        vd.field_choice("city_type", &["none", "city", "farm", "mine", "port", "wood"]);
        vd.field_bool("coastal");

        vd.field_validated_block("toggle_map_object_layers", |block, data| {
            let mut vd = Validator::new(block, data);
            // TODO: this list comes from the define NGraphics|DYNAMIC_MAP_OBJECT_LAYERS
            for layer in &[
                "semidynamic",
                "semidynamic_medium",
                "semidynamic_high",
                "mines_dynamic",
                "farms_dynamic",
                "forestry_dynamic",
            ] {
                vd.field_validated(layer, validate_layer);
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct TerrainMaterial {}

impl TerrainMaterial {
    // This gets the whole file as a Block
    pub fn add(db: &mut Db, _key: Token, block: Block) {
        // Structure is { { material } { material } ... } { ... }
        for block in block.iter_blocks_warn() {
            for block in block.iter_blocks_warn() {
                // docs say that the id field uniquely identifies a material,
                // but the name is the one actually used to look them up.
                if let Some(name) = block.get_field_value("name") {
                    db.add(Item::TerrainMaterial, name.clone(), block.clone(), Box::new(Self {}));
                } else {
                    untidy(ErrorKey::FieldMissing).msg("texture with no name").loc(block).push();
                }
            }
        }
    }
}

impl DbKind for TerrainMaterial {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_value("id");

        for field in &["diffuse", "normal", "material"] {
            vd.req_field(field);
            if let Some(token) = vd.field_value(field) {
                let path = block.loc.pathname().smart_join_parent(token.as_str());
                data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
            }
        }

        vd.req_field("mask");
        vd.field_value("mask");
    }
}

#[derive(Clone, Debug)]
pub struct TerrainMask {}

impl TerrainMask {
    pub fn add_json(db: &mut Db, block: Block) {
        // The masks are deeply nested in a json that looks like this:
        // { "masks": [ { ... }, { ... } ] }
        let mut count = 0;
        for block in block.iter_blocks_warn() {
            count += 1;
            if count == 2 {
                warn(ErrorKey::Validation).msg("expected only one block").loc(block).push();
            }
            if let Some(block) = block.get_field_block("masks") {
                for block in block.iter_blocks_warn() {
                    if let Some(token) = block.get_field_value("key") {
                        db.add(Item::TerrainMask, token.clone(), block.clone(), Box::new(Self {}));
                    } else {
                        warn(ErrorKey::FieldMissing).msg("mask with no key").loc(block).push();
                    }
                }
            }
        }
    }
}

impl DbKind for TerrainMask {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_value("key");
        vd.req_field("filename");
        if let Some(token) = vd.field_value("filename") {
            let path = key.loc.pathname().smart_join_parent(token.as_str());
            data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
        }
    }
}

fn validate_layer(bv: &BV, data: &Everything) {
    match bv {
        BV::Value(token) => {
            if !token.is("show_above_default") && !token.is("show_below_default") {
                let msg = "unknown layer position `{token}`";
                warn(ErrorKey::UnknownField).msg(msg).loc(token).push();
            }
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.req_field_one_of(&["show_above", "show_below"]);
            vd.field_numeric("show_above");
            vd.field_numeric("show_below");
        }
    }
}
