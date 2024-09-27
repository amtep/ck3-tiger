/// Specification of the hardcoded scripted rules in CK3.
///
/// Each definition is one rule.
///
/// `tooltipped` says whether the contents of this rule are tooltipped for the player.
/// It defaults to no. Left out when tooltipping is uncertain, otherwise set to yes or no.
///
/// `root` is the root of the scope context. Other fields are added named scopes.
///
/// For ease of updating, the rules are in the order they are found in the game files.
// LAST UPDATED CK3 VERSION 1.13.0.3
// Taken from information in common/scripted_rules/00_rules.txt
pub const SCRIPTED_RULES: &str = "
	can_command_troops = {
		tooltipped = no
		root = character
		army_owner = character
	}

	can_command_troops_now = {
		tooltipped = yes
		root = character
		army_owner = character
	}

	ghw_give_barony_to_beneficiary = {
		tooltipped = no
		root = province
		faith = faith
	}

	faith_creation = {
		tooltipped = yes
		root = character
	}

	faith_conversion = {
		tooltipped = yes
		root = character
		new_faith = faith
	}

	should_mother_give_house_to_bastard = {
		tooltipped = no
		root = character
	}

	can_be_knight = {
		tooltipped = no
		root = character
	}

	allowed_to_be_granted_titles_by = {
		tooltipped = yes
		root = character
		liege = character
		landed_title = landed_title
	}

	is_character_allowed_to_be_player = {
		tooltipped = no
		root = character
		will_override_government = bool
	}

	is_secret_available_for_blackmail = {
		tooltipped = yes
		root = secret
		target = character
		blackmailer = character
		secret_owner = character
		secret_target = character # may not exist
	}

	passes_faction_hard_block = {
		tooltipped = yes
		root = character
		target = character
	}

	passes_faction_soft_block = {
		tooltipped = yes
		root = character
	}

	is_dangerous_faction = {
		tooltipped = no
		root = faction
	}

	is_alliance_valid = {
		tooltipped = no
		root = character
		second = character
	}

	can_designate_heir = {
		tooltipped = yes
		root = character
	}

	cares_about_powerful_vassal_council_position = {
		tooltipped = no
		root = character
	}

	approves_of_succession_law_change = {
		tooltipped = no
		root = character
	}

	has_natural_death_second_chance = {
		tooltipped = no
		root = character
	}

	can_refund_perks = {
		tooltipped = yes
		root = character
	}

	can_defensively_join_holy_war = {
		tooltipped = no
		root = character
		attacker = character
		defender = character
	}

	can_fire_councillor = {
		tooltipped = yes
		root = character
		councillor = character
	}

	can_raid = {
		tooltipped = yes
		root = character
	}

	can_start_raid = {
		tooltipped = yes
		root = character
	}

	can_raid_across_water = {
		tooltipped = no
		root = character
	}

	can_traverse_river = {
		tooltipped = no
		root = character
	}

	is_hard_blocked_from_schemes = {
		tooltipped = no
		root = character
	}

	ai_wants_matrilineal_marriage = {
		tooltipped = no
		root = character
		secondary_actor = character
	}

	ai_wants_grand_wedding_promise = {
		tooltipped = no
		root = character
		actor = character
		secondary_actor = character
		recipient = character
		secondary_recipient = character
	}

	buildings_enabled = {
		tooltipped = yes
		root = character
	}

	can_potentially_call_ally = {
		tooltipped = no
		root = character
		ally = character
	}

	can_hybridize_culture = {
		tooltipped = yes
		root = character
		culture = culture
	}

	can_diverge_culture = {
		tooltipped = yes
		root = character
	}

	can_add_tradition = {
		tooltipped = yes
		root = character
	}

	can_replace_pillar = {
		tooltipped = yes
		root = character
	}

	is_eligible_for_court_positions = {
		root = character
	}

	can_name_after_birth = {
		tooltipped = no
		root = character
		child = character
	}

	can_adopt_court_language = {
		tooltipped = no
		root = character
		target = character
		list = {
			my_language_counties = landed_title
			their_language_counties = landed_title
			total_counties = landed_title
		}
	}

	ai_should_repair_artifact = {
		tooltipped = no
		root = character
		artifact = artifact
	}

	can_be_activity_guest = {
		root = character
		host = character
	}

	is_diarch_visibly_loyal = {
		tooltipped = no
		root = character
	}

	is_diarch_visibly_disloyal = {
		tooltipped = no
		root = character
	}

	is_diarch_able = {
		tooltipped = no
		root = character
	}

	is_diarch_valid = {
		tooltipped = no
		root = character
	}

	should_have_diarchy = {
		tooltipped = no
		root = character
	}

	can_be_acclaimed_knight = {
		tooltipped = no
		root = character
	}

	is_hostage_valid = {
		tooltipped = no
		root = character
	}

	can_create_legend = {
		root = character
	}

	can_promote_legend = {
		root = character
		legend = legend
	}

	ai_wants_to_create_own_legend = {
		tooltipped = no
		root = character
		num_legends = value
	}

	is_dominant_family = {
		root = dynasty_house
	}

	is_hireable_ruler_trigger = {
		root = character
	}

	can_hire_hireable_ruler_trigger = {
		root = character
		candidate = character
	}

	can_move_domicile = {
		root = province
		owner = character
	}
";
