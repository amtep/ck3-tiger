use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MapMode {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::MapMode, MapMode::add)
}

impl MapMode {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MapMode, key, block, Box::new(Self {}));
    }
}

impl DbKind for MapMode {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // TODO: whether these are actually used depends on the gui code
        data.mark_used(Item::Localization, key.as_str());
        let loca = format!("{key}_desc");
        data.mark_used(Item::Localization, &loca);

        // These are all chosen from hardcoded options
        vd.field_value("color_mode");
        vd.field_value("small_map_names");
        vd.field_value("large_map_names");
        vd.field_value("selection");
        vd.field_choice("fill_in_impassable", &["yes", "no", "no_small_names"]);

        vd.field_bool("select_holdings_on_close_zoom");

        vd.field_blocks("gradient_parameters");

        let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
        vd.field_validated_sc("barony_description", &mut sc, validate_desc);
        vd.field_validated_sc("selection_description", &mut sc, validate_desc);
    }
}
