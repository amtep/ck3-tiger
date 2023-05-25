use crate::block::Block;
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::{DbKind, Everything};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct ScriptedRule {}

impl ScriptedRule {
    pub fn boxed_new(_key: &Token, _block: &Block) -> Box<dyn DbKind> {
        Box::new(Self {})
    }
}

impl DbKind for ScriptedRule {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        for (name, scope, tooltipped) in SCRIPTED_RULE_ROOTS {
            if key.is(name) {
                let mut sc = ScopeContext::new_root(*scope, key.clone());
                validate_normal_trigger(block, data, &mut sc, *tooltipped);
                return;
            }
        }
        let msg = "unknown scripted rule";
        warn(key, ErrorKey::Validation, msg);
        let mut sc = ScopeContext::new_root(Scopes::non_primitive(), key.clone());
        validate_normal_trigger(block, data, &mut sc, false);
    }
}

// The third tuple item is whether the rule is tooltipped, which is unfortunately a guess
const SCRIPTED_RULE_ROOTS: &[(&str, Scopes, bool)] = &[
    ("can_command_troops", Scopes::Character, false),
    ("can_command_troops_now", Scopes::Character, true),
    ("ghw_give_barony_to_beneficiary", Scopes::Province, false),
    ("faith_creation", Scopes::Character, true),
    ("faith_conversion", Scopes::Character, true),
    (
        "should_mother_give_house_to_bastard",
        Scopes::Character,
        false,
    ),
    ("can_be_knight", Scopes::Character, false),
    ("allowed_to_be_granted_titles_by", Scopes::Character, true),
    ("is_character_allowed_to_be_player", Scopes::Character, true),
    ("is_secret_available_for_blackmail", Scopes::Secret, false),
    ("passes_faction_hard_block", Scopes::Character, true),
    ("passes_faction_soft_block", Scopes::Character, true),
    ("is_dangerous_faction", Scopes::Faction, false),
    ("is_alliance_valid", Scopes::Character, false),
    ("can_designate_heir", Scopes::Character, true),
    (
        "cares_about_powerful_vassal_council_position",
        Scopes::Character,
        false,
    ),
    (
        "approves_of_succession_law_change",
        Scopes::Character,
        false,
    ), // TODO: verify
    ("has_natural_death_second_chance", Scopes::Character, false),
    ("can_refund_perks", Scopes::Character, true),
    ("can_defensively_join_holy_war", Scopes::Character, false),
    ("can_fire_councillor", Scopes::Character, true),
    ("can_raid", Scopes::Character, true),
    ("can_start_raid", Scopes::Character, true),
    ("can_raid_across_water", Scopes::Character, false),
    ("can_traverse_river", Scopes::Character, false),
    ("is_hard_blocked_from_schemes", Scopes::Character, false),
    ("ai_wants_matrilineal_marriage", Scopes::Character, false),
    ("ai_wants_grand_wedding_promise", Scopes::Character, false),
    ("buildings_enabled", Scopes::Character, true),
    ("can_potentially_call_ally", Scopes::Character, false),
    ("can_hybridize_culture", Scopes::Character, true),
    ("can_diverge_culture", Scopes::Character, true),
    ("can_add_tradition", Scopes::Character, true),
    ("can_replace_pillar", Scopes::Character, true),
    ("is_eligible_for_court_positions", Scopes::Character, false),
    ("can_name_after_birth", Scopes::Character, false),
    ("can_adopt_court_language", Scopes::Character, false),
    ("ai_should_repair_artifact", Scopes::Character, false),
    ("can_be_activity_guest", Scopes::Character, false),
    ("is_diarch_visibly_loyal", Scopes::Character, false),
    ("is_diarch_visibly_disloyal", Scopes::Character, false),
    ("is_diarch_able", Scopes::Character, false),
    ("is_diarch_valid", Scopes::Character, false),
    ("should_have_diarchy", Scopes::Character, false),
    ("can_be_acclaimed_knight", Scopes::Character, false),
];
