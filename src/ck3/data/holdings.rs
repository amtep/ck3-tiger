use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct HoldingType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::HoldingType, HoldingType::add)
}

impl HoldingType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::HoldingType, key, block, Box::new(Self {}));
    }
}

impl DbKind for HoldingType {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("parameters") {
            for token in block.iter_values() {
                db.add_flag(Item::HoldingParameter, token.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let modif = format!("{key}_build_speed");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_build_gold_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_build_piety_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_build_prestige_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_speed");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_gold_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_piety_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_holding_build_prestige_cost");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let mut vd = Validator::new(block, data);
        vd.advice_field("flag", "replaced with parameters block in 1.16");
        vd.req_field("primary_building");
        vd.field_item("primary_building", Item::Building);
        vd.field_list_items("buildings", Item::Building);
        vd.field_bool("can_be_inherited");
        vd.field_bool("counts_toward_domain_limit_if_disabled");
        vd.field_list_items("required_heir_government_types", Item::GovernmentType);
        vd.field_list("parameters");
    }
}
