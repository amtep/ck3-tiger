/// Specification of the hardcoded scripted rules in Vic3.
///
/// Each definition is one rule.
///
/// `tooltipped` says whether the contents of this rule are tooltipped for the player.
/// It defaults to no. Left out when tooltipping is uncertain, otherwise set to yes or no.
///
/// `root` is the root of the scope context. Other fields are added named scopes.
///
/// For ease of updating, the rules are in the order they are found in the game files.
// LAST UPDATED VIC3 VERSION 1.7.1
// Taken from information in common/scripted_rules/00_scripted_rules.txt
pub const SCRIPTED_RULES: &str = "
	violate_sovereignty_war_check_rule = {
		tooltipped = no
		root = war
		target_country = country
	}

	has_voting_franchise = {
		tooltipped = no
		root = country
	}
	
	can_form_power_bloc = {
		root = country
	}

	can_lead_power_bloc = {
		root = country
	}

	is_weak_power_bloc = {
		root = power_bloc
	}

	can_start_diplomatic_plays_against = {
		root = country
		target_country = country
	}

	can_join_side_in_diplomatic_play = {
		root = country
		target_country = country
		enemy_country = country
	}

	can_impose_law_default = {
		root = country
		target_country = country
		law = law_type
	}

	# TODO: find out what the scopes are here.
	unlock_power_bloc_principle_slot_1 = {
		root = power_bloc
	}
	unlock_power_bloc_principle_slot_2 = {
		root = power_bloc
	}
	unlock_power_bloc_principle_slot_3 = {
		root = power_bloc
	}
	unlock_power_bloc_principle_slot_4 = {
		root = power_bloc
	}

	# TODO: find out what the scopes are here.
	unlock_power_bloc_formation_principle_slot_1 = {
		root = ALL
	}
	unlock_power_bloc_formation_principle_slot_2 = {
		root = ALL
	}
	unlock_power_bloc_formation_principle_slot_3 = {
		root = ALL
	}
	unlock_power_bloc_formation_principle_slot_4 = {
		root = ALL
	}

	can_sign_treaty_with = {
		root = country
		other_country = country
	}
	";
