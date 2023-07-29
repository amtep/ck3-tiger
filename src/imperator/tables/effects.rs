use crate::effect::Effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;
use crate::imperator::effect_validation::{EvB, EvBv, EvV};

use Effect::*;

pub fn scope_effect(name: &Token, _data: &Everything) -> Option<(Scopes, Effect)> {
    let lwname = name.as_str().to_lowercase();

    for (from, s, effect) in SCOPE_EFFECT {
        if lwname == *s {
            return Some((Scopes::from_bits_truncate(*from), *effect));
        }
    }
    std::option::Option::None
}

/// LAST UPDATED VERSION 2.0.4
/// See `effects.log` from the game data dumps
// TODO - "-tdb-" marks blocks that still need to be done.
const SCOPE_EFFECT: &[(u64, &str, Effect)] = &[
    (State, "add_state_food", ScriptValue),
    (State, "add_state_modifier", Unchecked), // -tdb-
    (State, "remove_state_modifier", Unchecked), // -tdb-
    (State, "set_state_capital", Scope(Scopes::Province) | Integer),
    (Character, "adapt_family_name", Boolean),
    (Character, "add_as_governor", Scope(Scopes::Governorship)),
    (Character, "add_character_experience", ScriptValue),
    (Character, "add_character_modifier", Unchecked), // -tdb-
    (Character, "add_corruption", ScriptValue),
    (Character, "add_friend", Scope(Scopes::Character)),
    (Character, "add_gold", ScriptValue),
    (Character, "add_health", ScriptValue),
    (Character, "add_holding", Scope(Scopes::Province)),
    (Character, "add_loyal_veterans", ScriptValue),
    (Character, "add_loyal_veterans", ScriptValue),
    (Character, "add_loyalty", Item(Item::Loyalty)),
    (Character, "add_nickname", Item(Item::Localization)),
    (Character, "add_party_conviction", Unchecked), // -tdb-
    (Character, "add_popularity", ScriptValue),
    (Character, "add_prominence", ScriptValue),
    (Character, "add_rival", Scope(Scopes::Character)),
    (Character, "add_ruler_conviction", Unchecked),
    (Character, "add_trait", Item(Item::CharacterTrait)),
    (Character, "add_triggered_character_modifier", Unchecked), // -tdb-
    (Character, "adopt", Scope(Scopes::Character)),
    (Character, "banish", Scope(Scopes::Country)),
    (Character, "change_mercenary_employer", Scope(Scopes::Country)),
    (Character, "clear_ambition", Boolean),
    (Character, "death", Unchecked), // -tdb-
    (Character, "deify_character", Unchecked), // -tdb-
    (Character, "divorce_character", Scope(Scopes::Character)),
    (Character, "end_pregnancy", Boolean),
    (Character, "force_add_trait", Item(Item::CharacterTrait)),
    (Character, "give_office", Item(Item::Office)),
    (Character, "marry_character", Scope(Scopes::Character)),
    (Character, "move_country", Scope(Scopes::Country)),
    (Character, "move_country_with_message", Scope(Scopes::Country)),
    (Character, "pay_gold", Unchecked), // -tdb-
    (Character, "remove_all_offices", Boolean),
    (Character, "remove_as_governor", Boolean),
    (Character, "remove_as_mercenary", Boolean),
    (Character, "remove_as_researcher", Boolean),
    (Character, "remove_character_modifier", Item(Item::Modifier)),
    (Character, "remove_command", Boolean),
    (Character, "remove_friend", Scope(Scopes::Character)),
    (Character, "remove_holding", Scope(Scopes::Province)),
    (Character, "remove_loyalty", Item(Item::Loyalty)),
    (Character, "remove_office", Item(Item::Office)),
    (Character, "remove_trait", Item(Item::CharacterTrait)),
    (Character, "remove_triggered_character_modifier", Item(Item::Modifier)),
    (Character, "set_ambition", Item(Item::Ambition)),
    (Character, "set_as_minor_character", Scope(Scopes::Character)),
    (Character, "set_character_religion", Item(Item::Religion)),
    (Character, "set_culture", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Character, "set_culture_same_as", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Character, "set_family", Scope(Scopes::Family)),
    (Character, "set_firstname", Item(Item::Localization)),
    (Character, "set_home_country", Scope(Scopes::Country)),
    (Character, "set_party_leader", Unchecked),
    (Character, "update_character", Boolean),
    (Character | Unit | Legion, "add_legion_history", Unchecked), // -tdb-
    (Character | Unit, "add_to_legion", Scope(Scopes::Legion)),
    (Character, "disband_legion", Boolean),
    (Character, "add_charisma", Integer),
    (Character, "add_finesse", Integer),
    (Character, "add_martial", Integer),
    (Character, "add_zeal", Integer),
    (Character, "make_pregnant", Unchecked), // -tdb-
    (Governorship, "raise_legion", Unchecked), // TODO - these blocks should only check if there is a create_unit call inside of them
    (Treasure, "destroy_treasure", Boolean),
    (Treasure, "transfer_treasure_to_character", Scope(Scopes::Character)),
    (Treasure, "transfer_treasure_to_country", Scope(Scopes::Country)),
    (Treasure, "transfer_treasure_to_province", Scope(Scopes::Province)),
    (Country, "add_aggressive_expansion", ScriptValue),
    (Country, "add_alliance", Scope(Scopes::Country)),
    (Country, "add_centralization", ScriptValue),
    (Country, "add_country_modifier", Unchecked), // -tdb-
    (Country, "add_guarantee", Scope(Scopes::Country)),
    (Country, "add_innovation", Integer),
    (Country, "add_legitimacy", ScriptValue),
    (Country, "add_manpower", ScriptValue),
    (Country, "add_military_access", Scope(Scopes::Country)),
    (Country, "add_military_experience", ScriptValue),
    (Country, "add_new_family", Unchecked),
    (Country, "add_opinion", Unchecked), // -tdb-
    (Country, "add_party_approval", Unchecked), // -tdb-
    (Country, "add_political_influence", ScriptValue),
    (Country, "add_research", Unchecked), // -tdb-
    (Country, "add_stability", ScriptValue),
    (Country, "add_to_war", Unchecked), // -tdb-
    (Country, "add_treasury", ScriptValue),
    (Country, "add_truce", Unchecked), // -tdb-
    (Country, "add_tyranny", ScriptValue),
    (Country, "add_war_exhaustion", ScriptValue),
    (Country, "change_country_adjective", Item(Item::Localization)),
    (Country, "change_country_color", Item(Item::NamedColor)),
    (Country, "change_country_flag", Unchecked),
    (Country, "change_country_name", Item(Item::Localization)),
    (Country, "change_country_tag", Unchecked),
    (Country, "change_government", Item(Item::Government)),
    (Country, "change_law", Item(Item::Law)),
    (Country, "create_character", Unchecked),  // -tdb-
    (Country, "create_country_treasure", Unchecked),  // -tdb-
    (Country, "create_family", Scope(Scopes::Character)),
    (Country, "declare_war_with_wargoal", Unchecked),  // -tdb-
    (Country, "imprison", Unchecked),  // -tdb-
    (Country, "integrate_country_culture", Scope(Scopes::CountryCuluture)),
    (Country, "make_subject", Unchecked),  // -tdb-
    (Country, "pay_price", Item(Item::Price)),
    (Country, "recalc_succession", Boolean),
    (Country, "refund_price", Item(Item::Price)),
    (Country, "release_prisoner", Unchecked),  // -tdb-
    (Country, "remove_country_modifier", Item(Item::Modifier)),
    (Country, "remove_gurantee", Scope(Scopes::Country)),
    (Country, "remove_opinion", Unchecked),  // -tdb-
    (Country, "remove_party_leadership", Scope(Scopes::Party)),
    (Country, "reverse_add_opinion", Unchecked), // -tdb-
    (Country, "set_as_coruler", Scope(Scopes::Character)),
    (Country, "set_as_ruler", Scope(Scopes::Character)),
    (Country, "set_capital", Scope(Scopes::Province)),
    (Country, "set_country_heritage", Item(Item::Heritage)),
    (Country, "set_country_religion", Scope(Scopes::Religion)),
    (Country, "set_gender_equality", Boolean),
    (Country, "set_graphical_culture", Item(Item::Ethnicity)),
    (Country, "set_ignore_senate_approval", Boolean),
    (Country, "set_legion_recruitment", Choice(&["enabled", "disabled", "capital"])),
    (Country, "set_primary_culture", Item(Item::Culture)),
    (Country, "start_civil_war", Scope(Scopes::Character)),
    (Country, "update_allowed_parties", Boolean),
    (Country, "set_party_agenda", Unchecked),
    (Country, "break_alliance", Scope(Scopes::Country)),
    (Legion, "add_commander", Scope(Scopes::Character)),
    (Legion, "add_distinction", Item(Item::LegionDistinction)),
    (Legion, "add_legion_unit", Unchecked),
    (Legion, "move_legion", Scope(Scopes::Character)),
    (Legion, "remove_commander", Scope(Scopes::Character)),
    (Legion, "remove_distinction", Item(Item::LegionDistinction)),
    (Legion, "remove_legion_unit", Unchecked),
    (Siege, "add_breach", Integer),
    (Legion, "create_unit", Unchecked), // -tdb-
    (Unit, "add_food", ScriptValue),
    (Unit, "add_loyal_subunit", Item(Item::Unit)),
    (Unit, "add_morale", ScriptValue),
    (Unit, "add_subunit", Item(Item::Unit)),
    (Unit, "add_unit_modifier", Unchecked), // -tdb-
    (Unit, "change_unit_owner", Scope(Scopes::Country)),
    (Unit, "damage_unit_morale_percent", ScriptValue),
    (Unit, "damage_unit_percent", ScriptValue),
    (Unit, "destroy_unit", Boolean),
    (Unit, "lock_unit", ScriptValue),
    (Unit, "unlock_unit", ScriptValue),
    (Unit, "remove_unit_loyalty", Boolean),
    (Unit, "remove_unit_modifier", Item(Item::Modifier)),
    (Unit, "set_as_commander", Scope(Scopes::Character)),
    (Unit, "set_unit_size", Unchecked),
    (Unit, "split_migrants_to", ScriptValue),
    (PopType, "kill_pop", Boolean),
    (PopType, "move_pop", Scope(Scopes::Province)),
    (PopType, "set_pop_culture", Scope(Scopes::Culture)),
    (PopType, "set_pop_culture_same_as", Scope(Scopes::PopType)),
    (PopType, "set_pop_religion", Scope(Scopes::Religion)),
    (PopType, "set_pop_religion_same_as", Scope(Scopes::PopType)),
    (PopType, "set_pop_type", Item(Item::PopType)),
    (SubUnit, "add_subunit_morale", ScriptValue),
    (SubUnit, "add_subunit_strength", ScriptValue),
    (SubUnit, "destroy_subunit", Boolean),
    (SubUnit, "remove_subunit_loyalty", Yes | SubUnit),
    (SubUnit, "set_personal_loyalty", Scope(Scopes::Character)),
    (Family, "add_prestige", ScriptValue),
    (Family, "move_family", Scope(Scopes::Country)),
    (Family, "remove_family", Scope(Scopes::Country)),
    (Country | War, "force_white_peace", Scope(Scopes::War)),
    (War, "remove_from_war", Scope(Scopes::Country)),
    (Province, "add_building_level", Item(Item::Building)),
    (Province, "add_civilization_value", ScriptValue),
    (Province, "add_claim", Scope(Scopes::Country)),
    (Province, "add_permanent_province_modifier", Unchecked), // -tdb-
    (Province, "add_province_modifier", Unchecked), // -tdb-
    (Province, "add_road_towards", Scope(Scopes::Province)),
    (Province, "add_state_loyalty", ScriptValue),
    (Province, "add_vfx", Unchecked),
    (Province, "begin_great_work_construction", Unchecked),
    (Province, "change_province_name", Unchecked),
    (Province, "create_country", Unchecked),
    (Province, "create_pop", Item(Item::PopType)),
    (Province, "create_state_pop", Item(Item::PopType)),
    (Province, "define_pop", Unchecked), // -tdb-
    (Province, "finish_great_work_construction", Unchecked),
    (Province, "hide_model", Unchecked),
    (Province, "remove_building_level", Item(Item::Building)),
    (Province, "remove_claim", Scope(Scopes::Country)),
    (Province, "remove_province_deity", Boolean),
    (Province, "remove_province_modifier", Item(Item::Modifier)),
    (Province, "remove_vfx", Unchecked),
    (Province, "set_as_governor", Scope(Scopes::Character)),
    (Province, "set_city_status", Item(Item::ProvinceRank)),
    (Province, "set_conquered_by", Scope(Scopes::Country)),
    (Province, "set_controller", Scope(Scopes::Country)),
    (Province, "set_owned_by", Scope(Scopes::Country)),
    (Province, "set_province_deity", Scope(Scopes::Deity)),
    (Province, "set_trade_goods", Scope(Scopes::TradeGood)),
    (Province, "show_animated_text", Unchecked),
    (Province, "show_model", Unchecked),
    (CountryCulture, "add_country_culture_modifier", Unchecked), // -tdb-
    (CountryCulture, "add_integration_progress", ScriptValue),
    (CountryCulture, "remove_country_culture_modifier", Item(Item::Modifier)),
    (CountryCulture, "set_country_culture_right", Item(Item::PopType)),
    (None, "reset_scoring", Scope(Scopes::Country)),
    (None, "add_to_global_variable_list", VB(EvB::AddToVariableList)),
    (ALL_BUT_NONE, "add_to_list", VV(EvV::AddToList)),
    (None, "add_to_local_variable_list", VB(EvB::AddToVariableList)),
    (ALL_BUT_NONE, "add_to_temporary_list", VV(EvV::AddToList)),
    (None, "add_to_variable_list", VB(EvB::AddToVariableList)),
    (None, "assert_if", Unchecked),
    (None, "assert_read", Unchecked),
    (None, "break", Boolean),
    (None, "break_if", Control),
    (None, "change_global_variable", VB(EvB::ChangeVariable)),
    (None, "change_local_variable", VB(EvB::ChangeVariable)),
    (None, "change_variable", VB(EvB::ChangeVariable)),
    (None, "clamp_global_variable", VB(EvB::ClampVariable)),
    (None, "clamp_local_variable", VB(EvB::ClampVariable)),
    (None, "clamp_variable", VB(EvB::ClampVariable)),
    (None, "clear_global_variable_list", Unchecked),
    (None, "clear_local_variable_list", Unchecked),
    (None, "clear_saved_scope", Unchecked),
    (None, "clear_variable_list", Unchecked),
    (None, "custom_label", ControlOrLabel),
    (None, "custom_tooltip", ControlOrLabel),
    (None, "debug_log", Unchecked),
    (None, "debug_log_scopes", Boolean),
    (None, "else", Control),
    (None, "else_if", Control),
    (None, "hidden_effect", Control),
    (None, "if", Control),
    (None, "random", Control),
    (None, "random_list", VB(EvB::RandomList)),
    (ALL_BUT_NONE, "remove_from_list", VV(EvV::RemoveFromList)),
    (None, "remove_global_variable", Unchecked),
    (None, "remove_list_global_variable", VB(EvB::AddToVariableList)),
    (None, "remove_list_local_variable", VB(EvB::AddToVariableList)),
    (None, "remove_list_variable", VB(EvB::AddToVariableList)),
    (None, "remove_local_variable", Unchecked),
    (None, "remove_variable", Unchecked),
    (None, "round_global_variable", VB(EvB::RoundVariable)),
    (None, "round_local_variable", VB(EvB::RoundVariable)),
    (None, "round_variable", VB(EvB::RoundVariable)),
    (ALL_BUT_NONE, "save_scope_as", VV(EvV::SaveScope)),
    (ALL_BUT_NONE, "save_temporary_scope_as", VV(EvV::SaveScope)),
    (None, "set_global_variable", VBv(EvBv::SetVariable)),
    (None, "set_local_variable", VBv(EvBv::SetVariable)),
    (None, "set_variable", VBv(EvBv::SetVariable)),
    (None, "show_as_tooltip", Control),
    (None, "switch", VB(EvB::Switch)),
    (None, "trigger_event", Unchecked),
    (None, "while", Control),
];
