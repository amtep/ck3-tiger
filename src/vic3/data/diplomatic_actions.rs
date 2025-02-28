use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::{Validator, ValueValidator};

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

        if block.get_field_bool("requires_approval").unwrap_or(true) {
            let loca = format!("{key}_action_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
            // The actions that are not shown in the lens are meant to be
            // enforced through diplomatic plays, rather than as proposals.
            if block.get_field_bool("show_in_lens").unwrap_or(true) {
                let loca = format!("{key}_proposal_accepted_name");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_accepted_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_declined_name");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_declined_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_notification_name");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_notification_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_third_party_accepted_name");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_third_party_accepted_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_third_party_declined_name");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_proposal_third_party_declined_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        } else {
            let loca = format!("{key}_action_notification_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_action_notification_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            if block.get_field_bool("should_notify_third_parties").unwrap_or(false) {
                let loca = format!("{key}_action_notification_third_party_name");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_action_notification_third_party_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        }
        if block.has_key("pact") {
            let loca = format!("{key}_pact_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_action_propose_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_action_break_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_action_notification_break_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_action_notification_break_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            if block.get_field_bool("should_notify_third_parties").unwrap_or(false) {
                let loca = format!("{key}_action_notification_third_party_break_name");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("{key}_action_notification_third_party_break_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        }

        vd.field_validated_list("groups", |token, data| {
            let mut vd = ValueValidator::new(token, data);
            vd.choice(&[
                "general",
                "subject",
                "overlord",
                "power_bloc",
                "power_bloc_leader",
                "power_bloc_member",
            ]);
        });

        vd.field_bool("requires_approval");
        vd.field_bool("uses_random_approval");
        vd.field_bool("show_confirmation_box");
        vd.field_bool("is_hostile");
        vd.field_bool("should_notify_third_parties"); // undocumented
        vd.field_bool("show_effect_in_tooltip"); // undocumented
        vd.field_bool("violates_sovereignty"); // undocumented
        vd.field_bool("can_use_obligations"); // undocumented
        vd.field_bool("can_select"); // undocumented
        vd.field_bool("can_select_to_break"); // undocumented
        vd.field_bool("show_in_lens"); // undocumented

        vd.field_choice(
            "state_selection",
            &[
                "first_required",
                "first_optional",
                "second_required",
                "second_optional",
                "both_required",
                "both_optional",
                "any_required",
            ],
        );
        for field in &["first_state_list", "second_state_list"] {
            vd.field_choice(
                field,
                &[
                    "first_country",
                    "second_country",
                    "all",
                    "first_country_and_subjects",
                    "second_country_and_subjects",
                ],
            );
        }

        vd.field_list_items("unlocking_technologies", Item::Technology); // undocumented

        vd.field_validated_block("selectable", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("potential", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        for field in &["first_state_trigger", "second_state_trigger"] {
            vd.field_validated_key_block(field, |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::State, key);
                sc.define_name("country", Scopes::Country, key);
                sc.define_name("target_country", Scopes::Country, key);
                validate_trigger(block, data, &mut sc, Tooltipped::Yes);
            });
        }
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

        vd.field_item("confirmation_sound", Item::Sound);
        vd.field_item("request_sound", Item::Sound);
        vd.field_item("hostile_sound", Item::Sound);
        vd.field_item("benign_sound", Item::Sound);

        // undocumented

        vd.field_item("texture", Item::File);
    }
}

fn validate_pact(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_numeric("cost");
    vd.field_bool("counts_for_tech_spread");
    vd.field_bool("show_in_outliner"); // undocumented
    vd.field_bool("can_be_used_in_sway_offers"); // undocumented
    vd.field_integer("sway_maneuvers_cost"); // undocumented
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
    vd.field_bool("is_hostile"); // undocumented
    vd.field_bool("is_guarantee_independence"); // undocumented
    vd.field_bool("exempt_from_service"); // undocumented
    vd.field_bool("is_foreign_investment_rights"); // undocumented
    vd.field_bool("junior_support_only_against_overlord"); // undocumented

    vd.field_item("subject_type", Item::SubjectType); // undocumented

    vd.field_bool("recipient_pays_maintenance"); // undocumented
    vd.replaced_field(
        "recipient_gets_income_transfer",
        "renamed to second_country_gets_income_transfer",
    );
    vd.field_bool("second_country_gets_income_transfer");
    vd.replaced_field(
        "income_transfer_based_on_recipient",
        "renamed to income_transfer_based_on_second_country",
    );
    vd.field_bool("income_transfer_based_on_second_country");
    vd.field_validated_block("income_transfer_to_pops", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.replaced_field("allow_discriminated", "allow_non_fully_accepted");
        vd.field_bool("allow_non_fully_accepted");
        for field in &["upper_strata_pops", "middle_strata_pops", "lower_strata_pops"] {
            vd.field_script_value(field, sc);
        }
    });
    vd.field_numeric("income_transfer"); // may be negative
    vd.field_numeric("max_paying_country_income_to_transfer");

    vd.field_numeric("relations_progress_per_day"); // undocumented
    vd.field_numeric("relations_improvement_max"); // undocumented
    vd.field_numeric("relations_improvement_min"); // undocumented

    vd.field_integer("forced_duration");

    vd.field_item("propose_string", Item::Localization);
    vd.field_item("break_string", Item::Localization);
    vd.field_item("ask_to_end_string", Item::Localization);

    vd.field_bool("actor_requires_approval_to_break");
    vd.field_bool("target_requires_approval_to_break");
    vd.field_bool("is_breaking_hostile");
    vd.field_bool("is_target_breaking_hostile");

    vd.replaced_field(
        "is_about_to_auto_break",
        "show_about_to_break_warning in requirement_to_maintain",
    );
    vd.replaced_field("should_auto_break", "trigger in requirement_to_maintain");
    vd.replaced_field("should_invalidate", "trigger in requirement_to_maintain");
    for field in &["actor_can_break", "target_can_break"] {
        vd.field_validated_block(field, |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    }
    vd.multi_field_validated_block("requirement_to_maintain", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_block("show_about_to_break_warning", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    });

    vd.replaced_field("break_effect", "manual_break_effect and auto_break_effect");
    for field in &[
        "daily_effect",
        "weekly_effect",
        "monthly_effect",
        "manual_break_effect",
        "auto_break_effect",
    ] {
        vd.field_validated_block(field, |block, data| {
            validate_effect(block, data, sc, Tooltipped::No);
        });
    }

    vd.field_validated_block("subject_relation", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_bool("annex_on_country_formation");
    });

    vd.multi_field_item("auto_support_type", Item::DiplomaticPlay);

    // undocumented

    // TODO: the existence of first_foreign_pro_country_lobby_member_modifier
    // and first_foreign_anti_country_lobby_member_modifier is a guess. Verify.
    for field in &[
        "first_modifier",
        "second_modifier",
        "first_foreign_pro_country_lobby_member_modifier",
        "first_foreign_anti_country_lobby_member_modifier",
        "second_foreign_pro_country_lobby_member_modifier",
        "second_foreign_anti_country_lobby_member_modifier",
    ] {
        vd.field_validated_block(field, |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
    }

    vd.field_choice("market_owner", &["first_country", "second_country"]);
}

fn validate_ai(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_script_value_rooted("evaluation_chance", Scopes::Country);

    for field in &["will_select_as_first_state", "will_select_as_second_state"] {
        vd.field_validated_key_block(field, |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::State, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }

    vd.field_bool("check_acceptance_for_will_break");
    vd.field_bool("check_acceptance_for_will_propose");

    vd.field_numeric_range("max_influence_spending_fraction", 0.0..=1.0);

    for field in &[
        "will_propose_with_states",
        "will_propose",
        "will_break",
        "will_propose_even_if_not_accepted",
    ] {
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
