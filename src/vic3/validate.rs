//! Validation functions that are useful for more than one data module in vic3.

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::validator::Validator;

pub fn validate_treaty_article(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_item("article", Item::TreatyArticle);
    // TODO: directed articles _must_ specify this value, mutual articles _must not_ specify this value
    vd.field_target("source_country", sc, Scopes::Country);
    // TODO: directed articles _must_ specify this value, mutual articles _must not_ specify this value
    vd.field_target("target_country", sc, Scopes::Country);
    // TODO: check which inputs the article requires
    vd.field_validated_block("inputs", |block, data| {
        let mut vd = Validator::new(block, data);
        for block in vd.blocks() {
            let mut vd = Validator::new(block, data);
            vd.field_script_value("quantity", sc);
            vd.field_target("goods", sc, Scopes::Goods);
            vd.field_target("state", sc, Scopes::State);
            vd.field_target("strategic_region", sc, Scopes::StrategicRegion);
            vd.field_target("company", sc, Scopes::Company);
            vd.field_target("building_type", sc, Scopes::BuildingType);
            vd.field_target("law_type", sc, Scopes::LawType);
            vd.field_target("country", sc, Scopes::Country);
            if block.num_items() > 1 {
                let msg = "use only 1 input per block";
                err(ErrorKey::Validation).msg(msg).loc(block).push();
            }
        }
    });
}
