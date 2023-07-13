use fnv::FnvHashMap;
use once_cell::sync::Lazy; // replace with std version once it's stable

use crate::block::BV;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::parse::pdxfile::parse_pdx_internal;
use crate::scopes::{scope_from_snake_case, Scopes};
use crate::token::Token;

#[derive(Debug, Clone)]
struct OnActionScopeContext {
    root: Scopes,
    names: Vec<(String, Scopes)>,
    lists: Vec<(String, Scopes)>,
}

pub fn on_action_scopecontext(key: &Token, _data: &Everything) -> Option<ScopeContext> {
    if let Some(oa_sc) = ON_ACTION_SCOPES.get(key.as_str()) {
        let mut sc = ScopeContext::new(oa_sc.root, key);
        for (name, s) in &oa_sc.names {
            sc.define_name(name, *s, key);
        }
        for (list, s) in &oa_sc.lists {
            sc.define_list(list, *s, key);
        }
        sc.set_strict_scopes(false);
        return Some(sc);
    }

    None
}

static ON_ACTION_SCOPES: Lazy<FnvHashMap<String, OnActionScopeContext>> = Lazy::new(|| {
    build_on_action_hashmap(
        "
	on_monthly_pulse_country = {
		root = country
	}
	on_yearly_pulse_country = on_monthly_pulse_country
	on_half_yearly_pulse_country = on_monthly_pulse_country
	on_five_year_pulse_country = on_monthly_pulse_country
	on_decade_pulse_country = on_monthly_pulse_country

	on_monthly_pulse_character = {
		root = character
	}
	on_yearly_pulse_character = on_monthly_pulse_character
	on_half_yearly_pulse_character = on_monthly_pulse_character
	on_five_year_pulse_character = on_monthly_pulse_character
	on_decade_pulse_character = on_monthly_pulse_character

	on_monthly_pulse_state = {
		root = state
	}
	on_yearly_pulse_state = on_monthly_pulse_state
	on_half_yearly_pulse_state = on_monthly_pulse_state
	on_five_year_pulse_state = on_monthly_pulse_state
	on_decade_pulse_state = on_monthly_pulse_state

	on_battle_started = {
		root = country
		battle = battle
		attacker = character
		defender = character
		state = state
	}
	on_battle_ended = on_battle_started
	on_battle_won = on_battle_started
	on_battle_lost = on_battle_started

	on_building_built = {
		root = building
	}
	on_start_expanding_building = on_building_built

	on_merge_markets = {
		root = market
		market = market
		trade_center = state
	}

	on_retarget_link = {
		root = state
	}

	on_create_market = {
		root = market
	}

	on_research_technology_started = {
		root = country
		technology = technology
	}
	on_acquired_technology = on_research_technology_started
	on_spreading_technology = on_research_technology_started

	on_diplomatic_play_started = {
		root = diplomatic_play
		initiator = country
		target = country
	}

	on_character_creation = {
		root = character
	}
	on_character_death = on_character_creation
	on_new_ruler = on_character_creation

	on_country_default = {
		root = country
	}
	on_country_no_longer_default = on_country_default

	on_diplomatic_action = {
		root = diplomatic_action
	}
	on_diplomatic_proposal = on_diplomatic_action
	on_diplomatic_proposal_accepted = on_diplomatic_action
	on_diplomatic_proposal_owe_obligation = on_diplomatic_action
	on_diplomatic_proposal_call_in_obligation = on_diplomatic_action
	on_diplomatic_proposal_declined = on_diplomatic_action

	on_diplomatic_action_break = on_diplomatic_action
	on_diplomatic_proposal_break = on_diplomatic_action
	on_diplomatic_proposal_break_accepted = on_diplomatic_action
	on_diplomatic_proposal_break_owe_obligation = on_diplomatic_action
	on_diplomatic_proposal_break_call_in_obligation = on_diplomatic_action
	on_diplomatic_proposal_break_declined = on_diplomatic_action

	on_diplomatic_action_third_party = on_diplomatic_action
	on_diplomatic_action_third_party_accepted = on_diplomatic_action
	on_diplomatic_action_third_party_declined = on_diplomatic_action
	on_diplomatic_action_third_party_break = on_diplomatic_action
	on_diplomatic_action_third_party_break_accepted = on_diplomatic_action
	on_diplomatic_action_third_party_break_declined = on_diplomatic_action

	on_diplomats_expelled = on_diplomatic_action

	on_diplomatic_pact_auto_break = {
		root = diplomatic_pact
	}
	on_diplomatic_pact_third_party_auto_break = on_diplomatic_pact_auto_break

	on_country_released_as_independent = {
		root = country
		target = country
	}
	on_country_released_as_own_subject = on_country_released_as_independent
	on_country_released_as_overlord_subject = on_country_released_as_independent

	on_migration_target_created = {
		root = state
	}
	on_migration_target_created_other = on_migration_target_created

	on_resource_discovered = {
		root = state
	}
	on_resource_depleted = on_resource_discovered

	on_peace_agreement_signed_war_leader = {
		root = country
	}
	on_peace_agreement_signed_war_participant = on_peace_agreement_signed_war_leader
	on_peace_agreement_signed_non_participant = on_peace_agreement_signed_war_leader
	on_capitulation = on_peace_agreement_signed_war_leader
	on_self_capitulated_notification = on_peace_agreement_signed_war_leader
	on_enemy_capitulated_notification = on_peace_agreement_signed_war_leader
	on_ally_capitulated_notification = on_peace_agreement_signed_war_leader

	on_mobilized_general = {
		root = character
	}
	on_demobilized_general = on_mobilized_general

	on_diplo_play_start = {
		root = diplomatic_play
	}
	on_diplo_play_start_third_party = on_diplo_play_start
	on_diplo_play_back_down = on_diplo_play_start
	on_diplo_play_back_down_involved = on_diplo_play_start
	on_diplo_play_join_side = on_diplo_play_start
	on_diplo_play_abandon_side = on_diplo_play_start
	on_diplo_play_war_start = on_diplo_play_start
	on_diplo_play_subject_released = on_diplo_play_start
	on_diplo_play_subject_released_overlord = on_diplo_play_start
	on_diplo_play_switch_sides = {
		root = diplomatic_play
		country = country
		previous = country
	}
	on_diplo_play_declare_neutrality = on_diplo_play_start
	on_sway_offer = { root = diplomatic_play }
	on_sway_offer_owe_obligation = { root = diplomatic_play }
	on_sway_offer_accepted = { root = diplomatic_play }
	on_country_swayed = { root = diplomatic_play }
	on_sway_offer_rejected = { root = diplomatic_play }

	on_production_method_changed = { root = building }

	on_law_enactment_started = { root = country }
	on_law_checkpoint_success = { root = country }
	on_law_checkpoint_advance = { root = country }
	on_law_checkpoint_debate = { root = country }
	on_law_checkpoint_stall = { root = country }
	on_law_enactment_pass = { root = country }
	on_law_enactment_fail = { root = country }
	on_law_enactment_ended = { root = country }
	on_law_activated = { root = country }

	on_revolution_start = {
		root = country
		target = country
	}
	on_revolution_end = on_revolution_start
	on_secession_start = on_revolution_start
	on_secession_end = on_revolution_start
	on_civil_war_won = { root = country }

	# undocumented
	on_revolution_checkpoint_reached = { root = country }
	# undocumented
	on_secession_checkpoint_reached = { root = country }

	on_political_movement_supported_law_cancelled = { root = political_movement }

	on_wargoal_enforced = { root = country }

	on_ig_resigned_government = { root = interest_group }

	on_new_culture_obsession = { root = culture }

	on_rank_changed = { root = country }

	on_claim_added = {
		root = country
		actor = country
		region = state_region
	}

	on_heir_born = { root = character }

	on_secession_country_helped_by_home_country = {
		root = country
		target = country
	}
	on_secession_country_not_helped_by_home_country = on_secession_country_helped_by_home_country

	on_wargoal_added = {
		root = diplomatic_play
		actor = country
	}
	on_wargoal_removed = on_wargoal_added
	on_war_end = {
		root = diplomatic_play
		actor = country
		target = country
	}

	on_journal_entry_activated = { root = journalentry }
	on_journal_entry_deactivated = { root = journalentry }
	on_journal_entry_completed = { root = journalentry }
	on_journal_entry_failed = { root = journalentry }

	on_native_uprising = { root = country }

	on_state_incorporation = { root = state }

	on_political_movement_formed = { root = political_movement }
	on_political_movement_disbanded = { root = political_movement }

	on_colony_created = { root = state }

	on_diplomatic_incident = {
		root = strategic_region
		actor = country
		target = country
	}

	on_sub_objective_completed = { root = objective }
	on_sub_objective_failed = { root = objective }
	on_objective_completed = { root = objective }

	on_party_created = {
		root = country
		target = party
	}
	on_party_disbanded = on_party_created

	on_election_campaign_start = { root = country }
	on_election_campaign_end = { root = country }
	on_government_reformed = { root = country }

	on_obligation_owed_by_us_expired = { root = country }
	on_obligation_owed_to_us_expired = { root = country }
	on_start_supporting_unification = { root = country }
	on_unification_candidate_added = { root = country }
	on_unification_candidate_removed = { root = country }
	on_stop_supporting_unification = { root = country }

	on_enemy_convoys_raided = { root = character }
	on_our_convoys_raided = { root = character }

	on_repudiate_obligation = { root = country }

	on_character_recruitment = { root = character }

	on_naval_invasion = {
		root = country
		actor = country
		admiral = character
		general = character
		state = state
	}
",
    )
});

fn build_on_action_hashmap(description: &'static str) -> FnvHashMap<String, OnActionScopeContext> {
    let mut hash: FnvHashMap<String, OnActionScopeContext> = FnvHashMap::default();

    let mut block = parse_pdx_internal(description, "on action builtin scopes");
    for item in block.drain() {
        let field = item.get_field().expect("internal error");
        match field.bv() {
            BV::Value(token) => {
                // key1 = key2 means copy from key2
                let value = hash.get(token.as_str()).expect("internal error");
                hash.insert(field.key().to_string(), value.clone());
            }
            BV::Block(block) => {
                let root = block.get_field_value("root").expect("internal error");
                let root = scope_from_snake_case(root.as_str()).expect("internal error");
                let mut value = OnActionScopeContext { root, names: Vec::new(), lists: Vec::new() };
                for (key, token) in block.iter_assignments() {
                    if key.is("root") {
                        continue;
                    }
                    let s = scope_from_snake_case(token.as_str()).expect("internal error");
                    value.names.push((key.to_string(), s));
                }
                for (key, block) in block.iter_definitions() {
                    if key.is("list") {
                        for (key, token) in block.iter_assignments() {
                            let s = scope_from_snake_case(token.as_str()).expect("internal error");
                            value.lists.push((key.to_string(), s));
                        }
                    }
                }
                hash.insert(field.key().to_string(), value);
            }
        }
    }

    hash
}
