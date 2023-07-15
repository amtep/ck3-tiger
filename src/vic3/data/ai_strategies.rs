use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::scriptvalue::validate_scriptvalue;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct AiStrategy {}

impl AiStrategy {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AiStrategy, key, block, Box::new(Self {}));
    }
}

impl DbKind for AiStrategy {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !key.is("ai_strategy_default") {
            data.verify_exists(Item::Localization, key);
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_choice("type", &["administrative", "diplomatic", "political"]);

        vd.field_item("icon", Item::File);
        vd.field_item("desired_tax_level", Item::Level);
        vd.field_item("max_tax_level", Item::Level);
        vd.field_item("min_tax_level", Item::Level);

        // TODO verify scope type
        vd.field_script_value_rooted("undesirable_infamy_level", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("unacceptable_infamy_level", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("ideological_opinion_effect_mult", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("revolution_aversion", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("min_law_chance_to_pass", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("max_progressiveness", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("max_regressiveness", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("diplomatic_play_neutrality", Scopes::Country);
        vd.field_script_value_rooted("diplomatic_play_boldness", Scopes::Country);
        vd.field_validated_key("wargoal_maneuvers_fraction", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("enemy_country", Scopes::Country, key);
            validate_scriptvalue(bv, data, &mut sc);
        });
        vd.field_script_value_rooted("change_law_chance", Scopes::Country);

        vd.field_list_items("pro_interest_groups", Item::InterestGroup);
        vd.field_list_items("anti_interest_groups", Item::InterestGroup);
        vd.field_validated_key_block("institution_scores", validate_institution_scores);

        vd.field_validated_key("obligation_value", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_scriptvalue(bv, data, &mut sc);
        });
        vd.field_validated_key("recklessness", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_scriptvalue(bv, data, &mut sc);
        });
        vd.field_validated_key("aggression", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_scriptvalue(bv, data, &mut sc);
        });
        vd.field_script_value_rooted("wanted_construction_sector_levels", Scopes::Country);
        vd.field_script_value_rooted("wanted_army_size", Scopes::Country);
        vd.field_script_value_rooted("wanted_navy_size", Scopes::Country);

        vd.field_validated_key_block("building_group_weights", validate_building_group_weights);

        vd.field_validated_key_block("subsidies", validate_subsidies);
        vd.field_validated_key_block("war_subsidies", validate_subsidies);
        vd.field_validated_key_block("goods_stances", validate_goods_stances);

        vd.field_script_value_rooted("colonial_interest_ratio", Scopes::Country);
        vd.field_validated_key_block("strategic_region_scores", validate_strategic_region_scores);
        vd.field_validated_key_block("secret_goal_scores", validate_secret_goal_scores);
        vd.field_validated_key_block("secret_goal_weights", validate_secret_goal_weights);
        vd.field_validated_key_block("wargoal_scores", validate_wargoal_scores);
        vd.field_validated_key_block("wargoal_weights", validate_wargoal_weights);

        vd.field_validated_key_block("possible", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key); // TODO scope type
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key("weight", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            validate_scriptvalue(bv, data, &mut sc);
        });
    }
}

fn validate_institution_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::None, key); // TODO scope type
    for (key, bv) in vd.unknown_fields() {
        data.verify_exists(Item::Institution, key);
        validate_scriptvalue(bv, data, &mut sc);
    }
}

fn validate_building_group_weights(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, token) in vd.unknown_value_fields() {
        data.verify_exists(Item::BuildingGroup, key);
        token.expect_number();
    }
}

// TODO what other options?
const SUBSIDIES_TYPES: &[&str] = &["should_have", "wants_to_have"];
fn validate_subsidies(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, token) in vd.unknown_value_fields() {
        data.verify_exists(Item::BuildingType, key);
        if !SUBSIDIES_TYPES.contains(&token.as_str()) {
            warn(ErrorKey::Validation).weak().msg("unknown subsidy type").loc(token).push();
        }
    }
}

fn validate_goods_stances(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    for (key, block) in vd.unknown_block_fields() {
        data.verify_exists(Item::Goods, key);
        let mut vd = Validator::new(block, data);
        vd.req_field("stance");
        vd.field_choice("stance", &["wants_high_supply", "wants_export", "does_not_want"]);
        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

fn validate_strategic_region_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    for (key, bv) in vd.unknown_fields() {
        data.verify_exists(Item::StrategicRegion, key);
        validate_scriptvalue(bv, data, &mut sc);
    }
}

fn validate_secret_goal_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    sc.define_name("target_country", Scopes::Country, key);
    for (key, bv) in vd.unknown_fields() {
        data.verify_exists(Item::SecretGoal, key);
        validate_scriptvalue(bv, data, &mut sc);
    }
}

fn validate_secret_goal_weights(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, token) in vd.unknown_value_fields() {
        data.verify_exists(Item::SecretGoal, key);
        token.expect_number();
    }
}

fn validate_wargoal_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    sc.define_name("target_country", Scopes::Country, key);
    sc.define_name("target_state", Scopes::State, key); // might not be set
    for (key, bv) in vd.unknown_fields() {
        data.verify_exists(Item::Wargoal, key);
        validate_scriptvalue(bv, data, &mut sc);
    }
}

fn validate_wargoal_weights(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, token) in vd.unknown_value_fields() {
        data.verify_exists(Item::Wargoal, key);
        token.expect_number();
    }
}
