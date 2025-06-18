// LAST UPDATED VIC3 VERSION 1.8.1
pub const ON_ACTION_SCOPES: &str = "
        on_monthly_pulse = {
            root = none
        }
        on_yearly_pulse = on_monthly_pulse

	on_monthly_pulse_country = {
		root = country
	}
	on_yearly_pulse_country = on_monthly_pulse_country
	on_half_yearly_pulse_country = on_monthly_pulse_country
	on_five_year_pulse_country = on_monthly_pulse_country
	on_decade_pulse_country = on_monthly_pulse_country
        on_monthly_pulse_country_elections = on_monthly_pulse_country
        on_half_yearly_pulse_country_elections = on_monthly_pulse_country
        on_five_year_pulse_country_elections = on_monthly_pulse_country
        on_decade_pulse_country_elections = on_monthly_pulse_country

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
                enemy_country = country
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
        on_diplomatic_action_overlord_decrease_autonomy = on_diplomatic_action
        on_diplomatic_action_overlord_increase_autonomy = on_diplomatic_action
        on_diplomatic_action_subject_increase_own_autonomy = on_diplomatic_action


	on_diplomats_expelled = {
		root = diplomatic_action
		initiator = country
		target = country
	}

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
	on_law_activated = { root = law }

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

	on_wargoal_enforced = {
                root = country
                target = country
                diplomatic_play = diplomatic_play
                wargoal_impact = value
        }

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

	on_journal_entry_activated = { root = journal_entry }
	on_journal_entry_deactivated = { root = journal_entry }
	on_journal_entry_completed = { root = journal_entry }
	on_journal_entry_failed = { root = journal_entry }

	on_native_uprising = { root = country }

	on_state_incorporation = { root = state }
	on_state_owner_change = { root = state }
	on_state_created = { root = state }

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

	on_enemy_convoys_raided = {
                root = character
                region = state_region
        }
	on_our_convoys_raided = on_enemy_convoys_raided

	on_repudiate_obligation = {
		root = country
		target_country = country
	}

	on_character_recruitment = { root = character }

        on_impose_law = {
                root = country
                initiator = country
                law = law
        }

        on_power_bloc_struggle_started = { root = country }
        on_power_bloc_struggle_ended = {
                root = power_bloc
                successful_contender = country
                failed_contender = country
        }

	on_naval_invasion = {
		root = country
		actor = country
		admiral = character
		general = character
		state = state
	}

	on_company_established = {
		root = country
		company = company
	}
	on_company_disbanded = on_company_established

	on_military_formation_created = { root = military_formation }

	on_travel_deploy_to_sea_node_cancelled = {
		root = military_formation
		province = province
		target = hq|province
	}

	on_travel_track_formation_cancelled = {
		root = military_formation
		formation = military_formation
		target = hq|province
	}

	on_travel_station_in_hq_cancelled = {
		root = military_formation
		hq = hq
		target = hq|province
	}

	on_travel_to_front_cancelled = {
		root = military_formation
		front = front
		target = hq|province
	}

	on_become_independent = {
		root = country
	}
	on_become_subject = {
		root = country
	}

	on_harvest_condition_started_in_country = {
		root = country
		area = harvest_condition
		state = state
		duration = value
		num_states = value
		intensity = value
	}

	on_game_started = {
		root = none
	}
	on_game_started_after_lobby = on_game_started

	on_treaty_entered_into_force = {
		root = treaty
	}
	on_treaty_enforced = {
		root = treaty
	}
	on_treaty_proposed = {
		root = treaty
	}
	on_treaty_proposal_declined = {
		root = country
	}
	on_country_released_as_company_subject = {
		root = country
		target = country
	}
	on_country_withdrawn_from_treaty = {
		root = treaty
		withdrawing_country = country
		non_withdrawing_country = country
	}
	on_country_broke_treaty = on_country_withdrawn_from_treaty
	on_treaty_dissolved = {
		root = country
		second_country = country
	}
";
