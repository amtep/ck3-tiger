use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Terrain {
    is_water: bool,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Terrain, Terrain::add)
}

impl Terrain {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("categories") {
            for (key, block) in block.drain_definitions_warn() {
                let is_water = block.get_field_bool("is_water").unwrap_or(false);
                db.add(Item::Terrain, key, block, Box::new(Self { is_water }));
            }
        } else if key.is("terrain") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::GraphicalTerrain, key, block, Box::new(GraphicalTerrain {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `categories`";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(&key).push();
        }
    }
}

impl DbKind for Terrain {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("color", validate_color);
        vd.field_bool("naval_terrain");
        vd.field_numeric("movement_cost");
        vd.field_numeric("attrition");
        vd.field_bool("is_water");
        vd.field_choice("sound_type", &["sea", "forest", "desert", "plains"]); // TODO: other choices
        vd.field_numeric("match_value");
        vd.field_integer("combat_width");
        vd.field_integer("combat_support_width");
        vd.field_numeric("ai_terrain_importance_factor");
        vd.field_numeric("supply_flow_penalty_factor");

        vd.field_validated_block("buildings_max_level", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Building, key);
                value.expect_integer();
            });
        });

        vd.field_validated_block("units", validate_unit);

        vd.unknown_block_fields(|key, block| {
            data.verify_exists(Item::SubUnit, key);

            let mut vd = Validator::new(block, data);
            vd.field_validated_block("units", validate_unit);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        validate_modifs(block, data, ModifKinds::all(), vd);
    }

    fn has_property(
        &self,
        _key: &Token,
        _block: &Block,
        _property: &str,
        _data: &Everything,
    ) -> bool {
        self.is_water
    }
}

fn validate_unit(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_numeric("attack");
    vd.field_numeric("movement");
    vd.field_numeric("defence");
}

#[derive(Clone, Debug)]
pub struct GraphicalTerrain {}

impl DbKind for GraphicalTerrain {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("type", Item::Terrain);
        vd.field_list_integers_exactly("color", 1);
        vd.field_integer("texture");
        vd.field_bool("spawn_city");
        vd.field_bool("perm_snow");
    }
}
