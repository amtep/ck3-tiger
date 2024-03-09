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
// LAST UPDATED VIC3 VERSION 1.6.0
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
    ";
