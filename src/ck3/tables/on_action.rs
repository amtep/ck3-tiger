// LAST UPDATED CK3 VERSION 1.10.0
pub const ON_ACTION_SCOPES: &str = "
	on_accolade_rank_change = {
		root = accolade
		positive = bool
	}
	on_accolade_glory_change = {
		root = accolade
		glory = value
	}
	on_accolade_created = {
		root = accolade
	}
	on_active_accolade_succession = {
		root = accolade
		new_owner = character
	}
	on_inactive_accolade_succession = on_active_accolade_succession
	on_accolade_acclaimed_death = {
		root = accolade
		old_acclaimed_knight = character
		new_accolade_type = bool
	}
	on_accolade_acclaimed_removal = on_accolade_acclaimed_death
	on_accolade_successor_death = {
		root = accolade
	}
	on_accolade_successor_removal = on_accolade_successor_death
	on_accolade_deactivated = {
		root = accolade
		owner = character
	}
	on_accolade_new_acclaimed_knight = {
		root = accolade
		glory = value
		new_accolade_type = bool
		new_acclaimed_knight = character
	}

	on_alliance_added = {
		root = none
		first = character
		second = character
	}
	on_alliance_removed = on_alliance_added
	on_alliance_broken = {
		root = none
		first = character
		second = character
		list = { first = character }
		list = { second = character }
	}

	on_army_monthly = {
		root = character
		army = army
	}
	on_army_enter_province = on_army_monthly
	on_siege_looting = {
		root = character
		county = landed_title
		barony = landed_title
		previous_controller = character
	}
	on_county_occupied = {
		root = character
		county = landed_title
		barony = landed_title
		previous_controller = character
		war = war
	}
	on_siege_completion = {
		root = character
		county = landed_title
		barony = landed_title
		previous_controller = character
		war = war
		list = { occupied_baronies = landed_title }
	}
	on_raid_action_start = {
		root = army
		raider = character
		barony = landed_title
		county = landed_title
	}
	on_raid_action_completion = on_raid_action_start
	on_raid_action_weekly = on_raid_action_start
	on_raid_loot_delivered = {
		root = army
		raider = character
	}
	on_defeat_raid_army = {
		root = army
		raider = character
		receiver = character
	}

	on_building_completed = {
		root = province
	}

	on_birth_mother = {
		root = character
		child = character
		mother = character
		real_father = character
		father = character
	}
	on_birth_father = on_birth_mother
	on_birth_real_father = on_birth_mother
	on_birth_child = {
		root = character
		child = character
		mother = character
		real_father = character
		father = character
		is_bastard = bool
	}
	on_pregnancy_mother = {
		root = character
		mother = character
		real_father = character
		father = character
	}
	on_pregnancy_father = on_pregnancy_mother
	on_pregnancy_ended_mother = on_pregnancy_mother

	on_combat_end_winner = {
		root = combat_side
		wipe = bool
	}
	on_combat_end_loser = on_combat_end_winner

	on_councillor_left = {
		root = character
		old_employer = character
		council_task = council_task
		councillor = character
	}

	on_stress_level_reduced = {
		root = character
	}
	on_stress_level_1 = on_stress_level_reduced
	on_stress_level_2 = on_stress_level_reduced
	on_stress_level_3 = on_stress_level_reduced
	on_stress_level_4 = on_stress_level_reduced

	on_county_faith_change = {
		root = landed_title
		old_faith = faith
	}
	on_county_culture_change = {
		root = landed_title
		old_culture = culture
	}

	on_character_culture_change = {
		root = character
	}

	on_dynasty_created = { # undocumented
		root = dynasty
	}

	on_trigger_court_events = {
		root = character
	}
	on_absent_from_royal_court = {
		root = character
		value = value
	}
	on_court_grandeur_level_changed = {
		root = character
		old_value = value
		new_value = value
	}
	on_court_language_changed = {
		root = character
	}
	on_court_type_changed = on_court_language_changed
	on_player_royal_court_first_gained = {
		root = character
	}

	on_courtier_decided_to_move_to_pool = {
		root = character
		courtier = character
		liege = character
		list = { characters = character }
	}
	on_courtier_ready_to_move_to_pool = on_courtier_decided_to_move_to_pool
	on_guest_arrived_from_pool = {
		root = character
		guest = character
		host = character
		list = { characters = character }
	}
	on_guest_ready_to_move_to_pool = {
		root = character
		guest = character
		host = character
		list = { characters = character }
		destination = province # TODO: verify scope type
	}
	on_join_court = {
		root = character
		new_employer = character
		old_employer = character # may be unset
	}
	on_leave_court = {
		root = character
		old_employer = character
	}

	on_tradition_removed = {
		root = culture
		tradition = flag  # TODO: verify scope type
	}
	on_tradition_added = on_tradition_removed
	on_culture_created = {
		root = culture
		founder = character
	}
	on_county_auto_granted_to_liege_culture = {
		root = culture
		actor = character
		landed_title = landed_title
	}
	on_county_auto_granted_to_local_culture = on_county_auto_granted_to_liege_culture

	on_death = {
		root = character
		killer = character # may be unset
	}
	on_natural_death_second_chance = {
		root = character
	}

	on_entered_diarchy = {
		root = character
		reason = flag
	}
	on_left_diarchy = {
		root = character
		old_diarch = character
	}
	on_diarch_change = {
		root = character
		reason = flag
		old_diarch = character
	}
	on_diarch_designation = {
		root = character
		former_designated_diarch = character
	}

	on_holy_order_new_lease = {
		root = holy_order
		patron = character
		barony = landed_title
	}
	on_holy_order_hired = {
		root = holy_order
		patron = character
		actor = character
	}
	on_holy_order_destroyed = {
		root = faith
		title = landed_title
		leader = character
	}

	on_perks_refunded = {
		root = character
	}

	on_ruler_designer_finished = {
		root = character
	}

	on_hook_used = {
		root = character
		target = character
	}

	on_artifact_changed_owner = {
		root = artifact
		owner = character
		old_owner = character
	}
	on_artifact_succession = {
		root = artifact
		owner = character
		old_owner = character
		old_primary = character
	}
	on_artifact_broken_through_decay = {
		root = artifact
		owner = character
	}
	on_artifact_broken_through_effect = on_artifact_broken_through_decay
	on_artifact_durability_very_low = on_artifact_broken_through_decay
	on_artifact_durability_low = {
		root = character # TODO: verify the doc
	}
	on_artifact_claim_gained = {
		root = character
		owner = character
		artifact = artifact
	}
	on_artifact_claim_lost = on_artifact_claim_gained

	on_commander_combat_finished = {
		root = character
		combat_side = combat_side
		victory = bool
	}
	on_army_combat_finished = {
		root = character
		combat_side = combat_side
		victory = bool
		list = { commanders = character }
		list = { knights = character }
	}

	on_marriage = {
		root = character
		spouse = character
	}
	on_divorce = {
		root = character
		spouse = character
		reason = flag
	}
	on_concubinage = {
		root = character
		concubine = character
	}
	on_concubinage_end = {
		root = character
		concubine = character
		reason = flag
	}
	on_betrothal_broken = {
		root = character
		second = character
		reason = flag
	}

	on_game_start = {
		root = none
	}
	on_game_start_after_lobby = on_game_start
	on_game_start_with_tutorial = on_game_start

	on_imprison = {
		root = character
		imprisoner = character
	}
	on_release_from_prison = on_imprison

	on_faith_created = {
		root = character
		old_faith = faith
	}
	on_faith_conversion = on_faith_created
	on_character_faith_change = on_faith_created
	on_faith_monthly = {
		root = faith
	}
	on_potential_great_holy_war_invalidation = {
		root = ghw
	}
	on_great_holy_war_invalidation = on_potential_great_holy_war_invalidation
	on_great_holy_war_countdown_end = on_potential_great_holy_war_invalidation
	on_great_holy_war_participant_replaced = {
		root = character
		great_holy_war = ghw
		replacement = character
	}

	yearly_global_pulse = {
		root = none
	}
	yearly_playable_pulse = {
		root = character
	}
	three_year_playable_pulse = yearly_playable_pulse
	five_year_playable_pulse = yearly_playable_pulse
	quarterly_playable_pulse = {
		root = character
		quarter = value
	}
	random_yearly_playable_pulse = yearly_playable_pulse
	random_yearly_everyone_pulse = yearly_playable_pulse
	five_year_everyone_pulse = yearly_playable_pulse
	three_year_pool_pulse = yearly_playable_pulse
	yearly_culture_pulse = {
		root = culture
	}
	three_yearly_culture_pulse = yearly_culture_pulse
	on_culture_era_changed = {
		root = culture
	}

	yearly_struggle_playable_pulse = {
		root = character
		struggle = struggle
	}
	five_year_struggle_playable_pulse = yearly_struggle_playable_pulse

	on_birthday = {
		root = character
	}

	on_title_destroyed = {
		root = character
		landed_title = landed_title
	}
	on_title_gain = {
		root = character
		title = landed_title
		previous_holder = character
		transfer_type = flag
	}
	on_title_gain_inheritance = on_title_gain
	on_title_gain_usurpation = on_title_gain
	on_title_lost = {
		root = character
		title = landed_title
		new_holder = character
		transfer_type = flag
	}
	on_explicit_claim_gain = {
		root = character
		title = landed_title
		transfer_type = flag
	}
	on_explicit_claim_lost = {
		root = character
		title = landed_title
	}
	on_rank_up = on_explicit_claim_lost
	on_rank_down = on_explicit_claim_lost
	on_vassal_gained = {
		root = character
		vassal = character
		old_liege = character
		transfer_type = flag
	}
	on_baron_found_or_created_for_title = {
		root = character
		liege = character
		title = landed_title
	}

	on_travel_plan_movement = {
		root = character
	}
	on_travel_plan_arrival = on_travel_plan_movement
	on_travel_plan_start = on_travel_plan_movement
	on_travel_plan_complete = on_travel_plan_movement
	on_travel_plan_abort = on_travel_plan_movement
	on_travel_plan_cancel = on_travel_plan_movement

	#on_travel_activity_complete = ? TODO
	#on_travel_activity_invalidated = ? TODO
	on_travel_activity_arrival_too_late = {
		root = character
		travel_plan = travel_plan
	}
	on_travel_activity_estimated_arrival_too_late = {
		root = character
		travel_plan = travel_plan
		estimated_arrival_diff_days = value
	}
	on_travel_leader_removed = {
		root = character
		travel_plan = travel_plan
		old_travel_leader = character
	}

	on_war_transferred = {
		root = character
		war = war
		defender = character
	}
	on_join_war_as_secondary = {
		root = character
		war = war
	}
	on_war_started = {
		root = casus_belli
		attacker = character
		defender = character
		claimant = character
		war = war # undocumented
	}
	on_war_won_attacker = on_war_started
	on_war_won_defender = on_war_started
	on_war_white_peace = on_war_started
	on_war_invalidated = on_war_started

	on_hostage_taken = {
		root = character
		hostage = character
		warden = character
		home_court = character
	}
	on_hostage_released = on_hostage_taken
	on_hostage_invalidated = {
		root = character
		warden = character
		home_court = character
		imprisoner = character
		reason = flag
	}
";
