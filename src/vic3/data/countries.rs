use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;

#[derive(Clone, Debug)]
pub struct Country {}

impl Country {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Country, key, block, Box::new(Self {}));
    }
}

impl DbKind for Country {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_ADJ");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("country_type", Item::CountryType);
        vd.field_item("tier", Item::CountryTier);
        vd.field_list_items("cultures", Item::Culture);
        vd.field_item("religion", Item::Religion);
        vd.field_item("capital", Item::StateRegion);
        vd.field_bool("is_named_from_capital");

        vd.field_validated_key_block(
            "valid_as_home_country_for_separatists",
            |key, block, data| {
                // TODO: what is the scope type here?
                let mut sc = ScopeContext::new(Scopes::None, key);
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            },
        );
    }
}

#[derive(Clone, Debug)]
pub struct CountryType {}

impl CountryType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CountryType, key, block, Box::new(Self {}));
    }
}

impl DbKind for CountryType {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_bool("is_colonizable");
        vd.field_bool("is_unrecognized");
        vd.field_bool("uses_prestige");
        vd.field_bool("has_events");
        vd.field_bool("has_military");
        vd.field_bool("has_economy");
        vd.field_bool("has_politics");
        vd.field_bool("can_research");

        vd.req_field("default_rank");
        vd.field_item("default_rank", Item::CountryRank);
        vd.field_item("default_subject_type", Item::SubjectType);
    }
}

#[derive(Clone, Debug)]
pub struct CountryRank {}

impl CountryRank {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CountryRank, key, block, Box::new(Self {}));
    }
}

impl DbKind for CountryRank {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_integer("rank_value");
        vd.field_integer("icon_index");
        vd.field_bool("enforce_subject_rank_check");
        vd.field_numeric("diplo_pact_cost");
        vd.field_numeric("prestige_average_threshold");
        vd.field_numeric("prestige_relative_threshold");
        vd.field_numeric("infamy_aggressor_scaling");
        vd.field_numeric("infamy_target_scaling");

        vd.field_list_items("valid_country_types", Item::CountryType);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
    }
}
