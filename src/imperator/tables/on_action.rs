pub const ON_ACTION_SCOPES: &str = "
	yearly_country_pulse = {
		root = country
	}

    monthly_country_pulse = yearly_country_pulse
    on_deficit_pulse = yearly_country_pulse
    on_state_secession = {
        root = state
    }
    on_change_pantheon = yearly_country_pulse
    biyearly_country_pulse = yearly_country_pulse
    on_rebellion_in_country = yearly_country_pulse
    on_great_work_completed = yearly_country_pulse
    on_invention = yearly_country_pulse
    on_action_requiring_senate_approval = yearly_country_pulse
    on_being_released = yearly_country_pulse
    on_enacting_omen = yearly_country_pulse
    on_ending_war = yearly_country_pulse
    on_losing_war = yearly_country_pulse
    on_winning_war = yearly_country_pulse
    decade_country_pulse = yearly_country_pulse


    yearly_province_pulse = {
        root = province
    }

    on_province_colonized = yearly_province_pulse
    on_province_occupied = yearly_province_pulse
    monthly_province_pulse = yearly_province_pulse
    on_ownership_change = yearly_province_pulse

    character_traits_pulse = {
        root = character
    }

    yearly_culture_religion_switch_pulse = character_traits_pulse
    on_command_gained = character_traits_pulse
    on_office_lost = character_traits_pulse
    on_being_imprisoned = character_traits_pulse
    yearly_debt_pulse = character_traits_pulse
    on_battle_lost = character_traits_pulse
    in_land_battle = character_traits_pulse
    on_becoming_adult = character_traits_pulse
    on_policy_change = character_traits_pulse
    on_command_lost = character_traits_pulse
    on_gw_construction_job_completed = character_traits_pulse
    disease_possibilities = character_traits_pulse
    monthly_job_pulse = character_traits_pulse
    on_office_gained = character_traits_pulse
    on_siege_won = character_traits_pulse
    monthly_head_of_family_pulse = character_traits_pulse
    on_being_captured = character_traits_pulse
    on_being_born = character_traits_pulse
    on_character_death = character_traits_pulse
    monthly_ruler_pulse = character_traits_pulse
    yearly_character_murder_pulse = character_traits_pulse
    yearly_financial_support_pulse = character_traits_pulse
    yearly_medical_pulse = character_traits_pulse
    on_being_ransomed_back = character_traits_pulse
    treatment_pulse = character_traits_pulse
    on_character_created = character_traits_pulse
    yearly_character_pulse = character_traits_pulse
    on_bastard_birth = character_traits_pulse
    in_naval_battle = character_traits_pulse
    on_tenth_birthday = character_traits_pulse
    yearly_disloyal_generals_delay_pulse = character_traits_pulse
    on_zero_health = character_traits_pulse
    character_story_events = character_traits_pulse

    on_civil_war_lost = {
        root = country
        losing_side = country
        target = character
    }

    on_civil_war_won = on_civil_war_lost

    on_culture_reduced_right = {
        root = country
        target_culture = country_culture
        target = pop
    }

    on_holding_lost = {
        root = character
        target = province
    }

    on_diplomatic_annex = {
        root = country
        actor = country
    }

    on_great_work_destroyed = {
        root = country
        target = country   
    }

    on_military_annex = on_diplomatic_annex
    on_civil_war_annex = on_diplomatic_annex

    on_move_country = {
        root = character
        old_country = country
    }

    on_reign_ending = {
        root = country
        former_ruler = character
    }

    on_ruler_change = on_reign_ending

    on_culture_increased_right = {
        root = country
        target_culture = country_culture
        target = pop
    }

    on_battle_lost_country = {
        root = country
        actor = unit
        target = unit
    }

    on_great_battle_won_country = on_battle_lost_country
    on_battle_won_country = on_battle_lost_country
    on_great_battle_lost_country = on_battle_lost_country
    on_battle_lost_country = on_battle_lost_country

    on_culture_integration_0 = {
        root = country
        target_culture = country_culture
    }

    on_culture_integration_25 = on_culture_integration_0
    on_culture_integration_75 = on_culture_integration_0
    on_culture_integration_100 = on_culture_integration_0

    on_disband_legion_unit = {
        root = country
        target = legion
        unit = unit
    }

    on_game_initialized = {
        root = country
    }

    on_reign_ending_successor = on_reign_ending
    on_deified_ruler_death = on_game_initialized
    on_great_work_anniversary = on_game_initialized
    on_battle_won = on_game_initialized

    on_legion_raised = {
        root = legion
    }

    on_legion_dissolved = on_legion_raised

    on_subject_defect = {
        root = country
        future_overlord = country
        target_subject = country
    }

    on_giving_birth = {
        root = character
        newborn = character
    }
";
