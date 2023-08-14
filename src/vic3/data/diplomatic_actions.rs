use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DiplomaticAction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DiplomaticAction, DiplomaticAction::add)
}

impl DiplomaticAction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DiplomaticAction, key, block, Box::new(Self {}));
    }
}

impl DbKind for DiplomaticAction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("target_country", Scopes::Country, key);
        // TODO: in which fields exactly is scope:actor defined?
        sc.define_name("actor", Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_action_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_action_propose_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_bool("requires_approval");
        vd.field_bool("show_confirmation_box");
        vd.field_bool("is_hostile");
        vd.field_bool("should_notify_third_parties"); // undocumented
        vd.field_bool("show_effect_in_tooltip"); // undocumented
        vd.field_bool("violates_sovereignty"); // undocumented
        vd.field_bool("can_use_obligations"); // undocumented
        vd.field_bool("can_select"); // undocumented
        vd.field_bool("can_select_to_break"); // undocumented

        vd.field_list_items("unlocking_technologies", Item::Technology); // undocumented

        vd.field_validated_block("potential", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("accept_effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("decline_effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.advice_field("effect", "the docs say effect but the field is called accept_effect");
        vd.field_validated_block_sc("pact", &mut sc, validate_pact);
        vd.field_validated_block_sc("ai", &mut sc, validate_ai);

        vd.field_item("reverse_pact", Item::DiplomaticAction); // undocumented
        vd.field_item("transfer_pact", Item::DiplomaticAction); // undocumented
    }
}

fn validate_pact(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_numeric("cost");
    vd.field_bool("counts_for_tech_spread");
    vd.field_bool("show_in_outliner"); // undocumented
    vd.field_bool("infamy_affects_maintenance"); // undocumented
    vd.field_bool("has_junior_participant"); // undocumented
    vd.field_bool("is_two_sided_pact"); // undocumented
    vd.field_bool("is_alliance"); // undocumented
    vd.field_bool("is_defensive_pact"); // undocumented
    vd.field_bool("is_rivalry"); // undocumented
    vd.field_bool("is_embargo"); // undocumented
    vd.field_bool("is_trade_agreement"); // undocumented
    vd.field_bool("is_customs_union"); // undocumented
    vd.field_bool("is_humiliation"); // undocumented
    vd.field_bool("is_colonization_rights"); // undocumented

    vd.field_item("subject_type", Item::SubjectType); // undocumented

    vd.field_bool("recipient_pays_maintenance"); // undocumented
    vd.field_bool("recipient_gets_income_transfer");
    vd.field_bool("income_transfer_based_on_recipient");
    vd.field_numeric_range("income_transfer", 0.0, 1.0);

    vd.field_numeric("relations_progress_per_day"); // undocumented
    vd.field_numeric("relations_improvement_max"); // undocumented
    vd.field_numeric("relations_improvement_min"); // undocumented

    vd.field_item("propose_string", Item::Localization);
    vd.field_item("break_string", Item::Localization);
    vd.field_item("ask_to_end_string", Item::Localization);

    vd.field_bool("actor_requires_approval_to_break");
    vd.field_bool("target_requires_approval_to_break");
    vd.field_bool("is_breaking_hostile");
    vd.field_bool("is_target_breaking_hostile");

    vd.field_validated_block("should_auto_break", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::Yes);
    });
    for field in
        &["is_about_to_auto_break", "should_invalidate", "actor_can_break", "target_can_break"]
    {
        vd.field_validated_block(field, |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    }

    for field in &["daily_effect", "weekly_effect", "monthly_effect", "break_effect"] {
        vd.field_validated_block(field, |block, data| {
            validate_effect(block, data, sc, Tooltipped::No);
        });
    }

    vd.field_validated_block("subject_relation", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_bool("annex_on_country_formation");
    });
}

fn validate_ai(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_bool("check_acceptance_for_will_break");
    vd.field_bool("check_acceptance_for_will_propose");

    vd.field_numeric_range("max_influence_spending_fraction", 0.0, 1.0);

    for field in &["will_propose", "will_break"] {
        vd.field_validated_block(field, |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    }

    for field in &[
        "accept_score",
        "accept_break_score",
        "propose_score",
        "propose_break_score",
        "use_obligation_chance",
        "owe_obligation_chance",
        "junior_accept_score",
    ] {
        vd.field_script_value(field, sc);
    }
    vd.advice_field("use_favor_chance", "this field is called use_obligation_chance");
    vd.advice_field("owe_favor_chance", "this field is called owe_obligation_chance");
}
