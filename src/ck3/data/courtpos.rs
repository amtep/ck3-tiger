use crate::block::Block;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CourtPositionCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CourtPositionCategory, CourtPositionCategory::add)
}

impl CourtPositionCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtPositionCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtPositionCategory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
    }
}

#[derive(Clone, Debug)]
pub struct CourtPosition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CourtPosition, CourtPosition::add)
}

impl CourtPosition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtPosition, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtPosition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        vd.field_item("skill", Item::Skill);
        vd.field_integer("max_available_positions");
        vd.field_item("category", Item::CourtPositionCategory);
        vd.field_choice("minimum_rank", &["county", "duchy", "kingdom", "empire"]);
        vd.field_bool("is_travel_related");

        let mut sc = ScopeContext::new(Scopes::None, key);
        sc.define_name("liege", Scopes::Character, key);
        sc.define_name("employee", Scopes::Character, key);
        vd.field_script_value("opinion", &mut sc);

        vd.field_validated_block("aptitude_level_breakpoints", validate_breakpoints);
        vd.field_script_value_rooted("aptitude", Scopes::Character);
        vd.field_validated_block_rooted("is_shown", Scopes::Character, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });

        vd.field_validated_block_rooted("valid_position", Scopes::Character, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_block("is_shown_character", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("valid_character", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        // guessing that root is the liege here
        vd.field_validated_block_rooted("revoke_cost", Scopes::Character, |block, data, sc| {
            validate_cost(block, data, sc);
        });

        vd.field_validated_key_block("salary", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("liege", Scopes::Character, key);
            validate_cost(block, data, &mut sc);
        });

        vd.field_validated_block("base_employer_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_validated_block("scaling_employer_modifiers", |block, data| {
            validate_scaling_employer_modifiers(block, data);
        });
        vd.field_validated_block("custom_scaling_employer_modifier_description", |block, data| {
            let mut vd = Validator::new(block, data);
            for field in ["terrible", "poor", "average", "good", "excellent", "range"] {
                vd.field_localization(field, &mut sc);
            }
        });

        vd.field_validated_block("base_employer_court_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_validated_block("scaling_employer_court_modifiers", |block, data| {
            validate_scaling_employer_modifiers(block, data);
        });

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_item("custom_employer_modifier_description", Item::Localization);
        vd.field_item("custom_employee_modifier_description", Item::Localization);

        vd.field_validated_block_rooted(
            "search_for_courtier",
            Scopes::Character,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );

        for field in &[
            "on_court_position_received",
            "on_court_position_revoked",
            "on_court_position_invalidated",
        ] {
            vd.field_validated_block(field, |block, data| {
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
        }

        vd.field_validated_key("candidate_score", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("employee", Scopes::Character, key);
            sc.define_name("base_value", Scopes::Value, key);
            sc.define_name("firing_court_position", Scopes::Bool, key);
            sc.define_name("percent_of_monthly_gold_income", Scopes::Value, key);
            sc.define_name("percent_of_positive_monthly_prestige_balance", Scopes::Value, key);
            sc.define_name("percent_of_positive_monthly_piety_balance", Scopes::Value, key);
            sc.define_name("percent_of_monthly_gold_income_all_positions", Scopes::Value, key);
            sc.define_name(
                "percent_of_positive_monthly_prestige_balance_all_positions",
                Scopes::Value,
                key,
            );
            sc.define_name(
                "percent_of_positive_monthly_piety_balance_all_positions",
                Scopes::Value,
                key,
            );
            sc.define_name("highest_available_aptitude", Scopes::Value, key); // undocumented
            sc.define_name("employee_aptitude", Scopes::Value, key); // undocumented
            validate_script_value(bv, data, &mut sc);
        });

        vd.field_script_value("sort_order", &mut sc);

        vd.field_bool("is_powerful_agent"); // undocumented
    }
}

fn validate_breakpoints(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_tokens_integers_exactly(4);
}

fn validate_scaling_employer_modifiers(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    for field in ["terrible", "poor", "average", "good", "excellent"] {
        vd.field_validated_block(field, |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
    }

    for field in [
        "aptitude_level_1",
        "aptitude_level_2",
        "aptitude_level_3",
        "aptitude_level_4",
        "aptitude_level_5",
    ] {
        if let Some(key) = block.get_key(field) {
            err(ErrorKey::Removed).msg("the aptitude_level_N fields have been replaced in 1.11 with terrible, poor, average, good, and excellent").loc(key).push();
        }
    }
}
