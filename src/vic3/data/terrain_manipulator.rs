use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{warn, ErrorKey};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct TerrainManipulator {}

impl TerrainManipulator {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        // Skip the common/terrain_manipulators/provinces/ files.
        // TODO: should be a way to tell fileset to skip subdirectories
        if key.loc.pathname.components().count() > 3 {
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
