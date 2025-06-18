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
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Country {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Country, Country::add)
}

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
        vd.field_item("social_hierarchy", Item::SocialHierarchy);
        vd.field_list_items("cultures", Item::Culture);
        vd.field_item("religion", Item::Religion);
        vd.field_item("capital", Item::StateRegion);
        vd.field_bool("is_named_from_capital");
        vd.field_bool("dynamic_country_definition");

        vd.field_validated("primary_unit_color", validate_possibly_named_color);
        vd.field_validated("secondary_unit_color", validate_possibly_named_color);
        vd.field_validated("tertiary_unit_color", validate_possibly_named_color);

        // TODO: what is the scope type here?
        vd.field_trigger_rooted(
            "valid_as_home_country_for_separatists",
            Tooltipped::No,
            Scopes::None,
        );
    }
}

#[derive(Clone, Debug)]
pub struct CountryType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CountryType, CountryType::add)
}

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

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CountryRank, CountryRank::add)
}

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

#[derive(Clone, Debug)]
pub struct CountryFormation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CountryFormation, CountryFormation::add)
}

impl CountryFormation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CountryFormation, key, block, Box::new(Self {}));
    }
}

impl DbKind for CountryFormation {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_case_sensitive(false);

        data.verify_exists(Item::Country, key);
        vd.field_bool("is_major_formation");

        vd.field_bool("use_culture_states");
        vd.field_numeric("required_states_fraction");
        vd.field_list_items("states", Item::StateRegion);

        if block.field_value_is("is_major_formation", "yes") {
            vd.field_item("unification_play", Item::DiplomaticPlay);
            vd.field_item("leadership_play", Item::DiplomaticPlay);
        } else {
            vd.ban_field("unification_play", || "major formations");
            vd.ban_field("leadership_play", || "major formations");
        }

        vd.field_trigger_rooted("ai_will_do", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("possible", Tooltipped::Yes, Scopes::Country);
    }
}

#[derive(Clone, Debug)]
pub struct CountryCreation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CountryCreation, CountryCreation::add)
}

impl CountryCreation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CountryCreation, key, block, Box::new(Self {}));
    }
}

impl DbKind for CountryCreation {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_case_sensitive(false);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Country, key);

        // TODO: verify if `use_culture_states = yes` together with a `states` list is an error

        vd.field_bool("use_culture_states");
        vd.field_integer("required_num_states");
        vd.field_list_items("states", Item::StateRegion);
        vd.field_list_items("provinces", Item::Province);

        vd.field_trigger("possible", Tooltipped::Yes, &mut sc);
        vd.field_trigger("ai_will_do", Tooltipped::No, &mut sc);
    }
}
