use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CharacterTemplate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CharacterTemplate, CharacterTemplate::add)
}

impl CharacterTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterTemplate, key, block, Box::new(Self {}));
    }
}

// TODO: check that the "default" template exists

impl DbKind for CharacterTemplate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        if key.is("default") {
            // The default template must have all these fields filled out.
            vd.req_field("first_name");
            vd.req_field("last_name");
            vd.req_field("historical");
            vd.req_field("culture");
            vd.req_field("female");
            vd.req_field("dna");
            vd.req_field("age");
            vd.req_field("interest_group");
            vd.req_field("commander_rank");
            vd.req_field("trait_generation");
        }

        vd.field_validated_value("first_name", |_, mut vd| {
            vd.maybe_is("culture");
            vd.item(Item::Localization);
        });
        vd.field_validated_value("last_name", |_, mut vd| {
            vd.maybe_is("culture");
            vd.item(Item::Localization);
        });

        vd.field_bool("historical");
        vd.field_bool("noble");
        vd.field_bool("female");

        vd.field_validated_value("culture", |_, mut vd| {
            vd.maybe_is("primary_culture");
            vd.maybe_is("ig_before_primary_culture");
            vd.target(&mut sc, Scopes::Culture);
        });
        vd.field_validated_value("religion", |_, mut vd| {
            vd.maybe_is("random");
            vd.item_or_target(&mut sc, Item::Religion, Scopes::Religion);
        });

        vd.field_validated_value("dna", |_, mut vd| {
            vd.maybe_is("random");
            vd.item(Item::Dna);
        });
        vd.field_validated_value("age", |_, mut vd| {
            vd.maybe_is("default");
            vd.integer();
        });
        vd.field_validated_value("interest_group", |_, mut vd| {
            vd.maybe_is("random");
            vd.item(Item::InterestGroup);
        });
        vd.field_item("ideology", Item::Ideology);

        vd.field_bool("is_general");
        vd.field_bool("is_admiral");
        vd.field_item("hq", Item::StrategicRegion);
        vd.field_validated_value("commander_rank", |_, mut vd| {
            vd.maybe_is("default");
            vd.item(Item::CommanderRank);
        });
        vd.field_bool("ruler");
        vd.field_bool("heir");
        vd.field_bool("ig_leader");
        vd.field_bool("is_agitator");

        vd.field_date("birth_date");
        vd.field_list_items("traits", Item::CharacterTrait);

        // TODO: "should only be used for traits"
        vd.field_effect_rooted("trait_generation", Tooltipped::No, Scopes::Character);
        vd.field_effect_rooted("on_created", Tooltipped::No, Scopes::Character);

        vd.field_validated_block("commander_usage", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger_rooted("country_trigger", Tooltipped::No, Scopes::Country);
            vd.field_choice("role", &["general", "admiral"]);
            vd.field_date("earliest_usage_date");
            vd.field_date("latest_usage_date");
            vd.field_numeric_range("chance", 0.0..=100.0);
        });

        for field in &["interest_group_leader_usage", "agitator_usage"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_trigger_rooted("country_trigger", Tooltipped::No, Scopes::Country);
                vd.field_trigger_rooted(
                    "interest_group_trigger",
                    Tooltipped::No,
                    Scopes::InterestGroup,
                );
                vd.field_date("earliest_usage_date");
                vd.field_date("latest_usage_date");
                vd.field_numeric_range("chance", 0.0..=100.0);
            });
        }

        vd.field_validated_block("executive_usage", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger_rooted("country_trigger", Tooltipped::No, Scopes::Country);
            vd.field_trigger_rooted("company_trigger", Tooltipped::No, Scopes::Company);
            vd.field_date("earliest_usage_date");
            vd.field_date("latest_usage_date");
            vd.field_numeric_range("chance", 0.0..=100.0);
        });
    }
}
