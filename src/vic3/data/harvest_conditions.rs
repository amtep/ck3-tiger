use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct HarvestConditionType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::HarvestConditionType, HarvestConditionType::add)
}

impl HarvestConditionType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::HarvestConditionType, key, block, Box::new(Self {}));
    }
}

impl DbKind for HarvestConditionType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_key_block("trigger", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::State, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("time", |key, block, data| {
            // TODO: figure out scope here
            let mut sc = ScopeContext::new(Scopes::all(), key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.multi_field_item("incompatible_with", Item::HarvestConditionType);

        let mut sc = ScopeContext::new(Scopes::State, key);
        vd.field_script_value("range", &mut sc);
        vd.field_script_value("duration", &mut sc);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_script_value("intensity", &mut sc);
        vd.field_script_value("chance", &mut sc);

        // TODO: find out if the color = { R G B } documentation is the only option
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("icon", Item::File);

        vd.field_choice(
            "graphics",
            &[
                "none",
                "drought",
                "flood",
                "frost",
                "wildfire",
                // undocumented
                "hail",
                "extreme_winds",
                "torrential_rains",
            ],
        );

        vd.field_list_items("incompatible_terrain", Item::Terrain);
        vd.field_item("map_texture", Item::File);
    }
}
