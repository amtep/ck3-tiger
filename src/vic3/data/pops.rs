use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PopType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PopType, PopType::add)
}

impl PopType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PopType, key, block, Box::new(Self {}));
    }
}

impl DbKind for PopType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_no_icon");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_only_icon");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("texture", Item::File);
        vd.field_validated("color", validate_possibly_named_color);
        vd.replaced_field("strata", "social class definitions");

        vd.field_integer("start_quality_of_life");
        vd.field_numeric("wage_weight");
        vd.field_bool("paid_private_wage");
        vd.field_numeric("literacy_target");
        vd.field_numeric("consumption_mult");
        vd.field_numeric("dependent_wage");

        vd.field_bool("unemployment");
        vd.field_numeric("unemployment_wealth");

        vd.field_numeric_range("political_engagement_base", 0.0..=1.0);
        vd.field_numeric("political_engagement_literacy_factor");
        vd.field_script_value_rooted("political_engagement_mult", Scopes::Pop);

        vd.field_item("qualifications_growth_desc", Item::Localization);
        vd.field_script_value_rooted("qualifications", Scopes::Pop);

        vd.field_script_value_rooted("portrait_age", Scopes::Pop);
        vd.field_script_value_rooted("portrait_pose", Scopes::Pop);
        vd.field_validated_key_block("portrait_is_female", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Pop, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        // undocumented

        vd.field_numeric("education_access");
        vd.field_numeric_range("working_adult_ratio", 0.0..=1.0);
        vd.field_bool("can_always_hire");
        vd.field_bool("subsistence_income");
        vd.field_bool("ignores_employment_proportionality");
        vd.field_bool("is_slave");
        vd.field_bool("military");
    }
}
