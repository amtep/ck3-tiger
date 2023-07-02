use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct ScriptedRule {}

impl ScriptedRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedRule, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedRule {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        for (name, scope, tooltipped) in SCRIPTED_RULE_ROOTS {
            if key.is(name) {
                let mut sc = ScopeContext::new(*scope, key);
                sc.set_strict_scopes(false); // TODO
                validate_normal_trigger(block, data, &mut sc, *tooltipped);
                return;
            }
        }
        let msg = "unknown scripted rule";
        warn(key, ErrorKey::Validation, msg);
        let mut sc = ScopeContext::new(Scopes::non_primitive(), key);
        validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
    }
}

// The third tuple item is whether the rule is tooltipped, which is unfortunately a guess
const SCRIPTED_RULE_ROOTS: &[(&str, Scopes, Tooltipped)] = &[
    ("can_command_troops", Scopes::Character, Tooltipped::No),
    ("can_command_troops_now", Scopes::Character, Tooltipped::Yes),
    (
        "ghw_give_barony_to_beneficiary",
        Scopes::Province,
        Tooltipped::No,
    ),
    ("faith_creation", Scopes::Character, Tooltipped::Yes),
    ("faith_conversion", Scopes::Character, Tooltipped::Yes),
    (
        "should_mother_give_house_to_bastard",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("can_be_knight", Scopes::Character, Tooltipped::No),
    (
        "allowed_to_be_granted_titles_by",
        Scopes::Character,
        Tooltipped::Yes,
    ),
    (
        "is_character_allowed_to_be_player",
        Scopes::Character,
        Tooltipped::Yes,
    ),
    (
        "is_secret_available_for_blackmail",
        Scopes::Secret,
        Tooltipped::No,
    ),
    (
        "passes_faction_hard_block",
        Scopes::Character,
        Tooltipped::Yes,
    ),
    (
        "passes_faction_soft_block",
        Scopes::Character,
        Tooltipped::Yes,
    ),
    ("is_dangerous_faction", Scopes::Faction, Tooltipped::No),
    ("is_alliance_valid", Scopes::Character, Tooltipped::No),
    ("can_designate_heir", Scopes::Character, Tooltipped::Yes),
    (
        "cares_about_powerful_vassal_council_position",
        Scopes::Character,
        Tooltipped::No,
    ),
    (
        "approves_of_succession_law_change",
        Scopes::Character,
        Tooltipped::No,
    ), // TODO: verify
    (
        "has_natural_death_second_chance",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("can_refund_perks", Scopes::Character, Tooltipped::Yes),
    (
        "can_defensively_join_holy_war",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("can_fire_councillor", Scopes::Character, Tooltipped::Yes),
    ("can_raid", Scopes::Character, Tooltipped::Yes),
    ("can_start_raid", Scopes::Character, Tooltipped::Yes),
    ("can_raid_across_water", Scopes::Character, Tooltipped::No),
    ("can_traverse_river", Scopes::Character, Tooltipped::No),
    (
        "is_hard_blocked_from_schemes",
        Scopes::Character,
        Tooltipped::No,
    ),
    (
        "ai_wants_matrilineal_marriage",
        Scopes::Character,
        Tooltipped::No,
    ),
    (
        "ai_wants_grand_wedding_promise",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("buildings_enabled", Scopes::Character, Tooltipped::Yes),
    (
        "can_potentially_call_ally",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("can_hybridize_culture", Scopes::Character, Tooltipped::Yes),
    ("can_diverge_culture", Scopes::Character, Tooltipped::Yes),
    ("can_add_tradition", Scopes::Character, Tooltipped::Yes),
    ("can_replace_pillar", Scopes::Character, Tooltipped::Yes),
    (
        "is_eligible_for_court_positions",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("can_name_after_birth", Scopes::Character, Tooltipped::No),
    (
        "can_adopt_court_language",
        Scopes::Character,
        Tooltipped::No,
    ),
    (
        "ai_should_repair_artifact",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("can_be_activity_guest", Scopes::Character, Tooltipped::No),
    ("is_diarch_visibly_loyal", Scopes::Character, Tooltipped::No),
    (
        "is_diarch_visibly_disloyal",
        Scopes::Character,
        Tooltipped::No,
    ),
    ("is_diarch_able", Scopes::Character, Tooltipped::No),
    ("is_diarch_valid", Scopes::Character, Tooltipped::No),
    ("should_have_diarchy", Scopes::Character, Tooltipped::No),
    ("can_be_acclaimed_knight", Scopes::Character, Tooltipped::No),
];
