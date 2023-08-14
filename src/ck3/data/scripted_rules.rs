use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::parse::pdxfile::parse_pdx_internal;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct ScriptedRule {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ScriptedRule, ScriptedRule::add)
}

impl ScriptedRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedRule, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedRule {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        if let Some(sr_sc) = SCRIPTED_RULE_SCOPES_MAP.get(key.as_str()) {
            let mut sc = ScopeContext::new(sr_sc.root, key);
            for (name, s) in &sr_sc.names {
                sc.define_name(name, *s, key);
            }
            for (list, s) in &sr_sc.names {
                sc.define_list(list, *s, key);
            }
            validate_trigger(block, data, &mut sc, sr_sc.tooltipped);
        } else {
            let msg = "unknown scripted rule";
            err(ErrorKey::Validation).msg(msg).loc(key).push();
            let mut sc = ScopeContext::new(Scopes::non_primitive(), key);
            sc.set_strict_scopes(false);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        }
    }
}

#[derive(Debug, Clone)]
struct ScriptedRuleScopeContext {
    tooltipped: Tooltipped,
    root: Scopes,
    names: Vec<(String, Scopes)>,
    lists: Vec<(String, Scopes)>,
}

/// Processed version of [`SCRIPTED_RULES`].
static SCRIPTED_RULE_SCOPES_MAP: Lazy<FnvHashMap<String, ScriptedRuleScopeContext>> =
    Lazy::new(|| build_scripted_rule_hashmap(SCRIPTED_RULES));

// Mostly copied from build_on_action_hashmap.
// TODO: more generic facility for this?
fn build_scripted_rule_hashmap(
    description: &'static str,
) -> FnvHashMap<String, ScriptedRuleScopeContext> {
    let mut hash: FnvHashMap<String, ScriptedRuleScopeContext> = FnvHashMap::default();

    let mut block = parse_pdx_internal(description, "scripted rule builtin scopes");
    for (key, block) in block.drain_definitions_warn() {
        let root = block.get_field_value("root").expect("internal error");
        let root = Scopes::from_snake_case(root.as_str()).expect("internal error");
        let tooltipped = if block.field_value_is("tooltipped", "yes") {
            Tooltipped::Yes
        } else {
            Tooltipped::No
        };
        let mut value =
            ScriptedRuleScopeContext { tooltipped, root, names: Vec::new(), lists: Vec::new() };
        for (key, token) in block.iter_assignments() {
            if key.is("root") || key.is("tooltipped") {
                continue;
            }
            let s = Scopes::from_snake_case(token.as_str()).expect("internal error");
            value.names.push((key.to_string(), s));
        }
        for (key, block) in block.iter_definitions() {
            if key.is("list") {
                for (key, token) in block.iter_assignments() {
                    let s = Scopes::from_snake_case(token.as_str()).expect("internal error");
                    value.lists.push((key.to_string(), s));
                }
            }
        }
        hash.insert(key.to_string(), value);
    }

    hash
}

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
// LAST UPDATED CK3 VERSION 1.9.2.1
// Taken from information in common/scripted_rules/00_rules.txt
const SCRIPTED_RULES: &str = "
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
	";
