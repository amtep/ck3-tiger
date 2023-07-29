use crate::effect::Effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::imperator::effect_validation::{EvB, EvBv, EvV};

use Effect::*;

pub fn scope_effect(name: &Token, _data: &Everything) -> Option<(Scopes, Effect)> {
    let lwname = name.as_str().to_lowercase();

    for (from, s, effect) in SCOPE_EFFECT {
        if lwname == *s {
            return Some((*from, *effect));
        }
    }
    None
}

/// LAST UPDATED VERSION 2.0.4
/// See `effects.log` from the game data dumps
// TODO - "-tdb-" marks blocks that still need to be done.
const SCOPE_EFFECT: &[(Scopes, &str, Effect)] = &[
    (Scopes::State, "add_state_food", ScriptValue),
    (Scopes::State, "add_state_modifier", Unchecked), // -tdb-
    (Scopes::State, "remove_state_modifier", Unchecked), // -tdb-
    (Scopes::State, "set_state_capital", Scope(Scopes::Province) | Integer),
    (Scopes::Character, "adapt_family_name", Boolean),
    (Scopes::Character, "add_as_governor", Scope(Scopes::Governorship)),
    (Scopes::Character, "add_character_experience", ScriptValue),
    (Scopes::Character, "add_character_modifier", Unchecked), // -tdb-
    (Scopes::Character, "add_corruption", ScriptValue),
    (Scopes::Character, "add_friend", Scope(Scopes::Character)),
    (Scopes::Character, "add_gold", ScriptValue),
    (Scopes::Character, "add_health", ScriptValue),
    (Scopes::Character, "add_holding", Scope(Scopes::Province)),
    (Scopes::Character, "add_loyal_veterans", ScriptValue),
    (Scopes::Character, "add_loyal_veterans", ScriptValue),
    (Scopes::Character, "add_loyalty", Item(Item::Loyalty)),
    (Scopes::Character, "add_nickname", Item(Item::Localization)),
    (Scopes::Character, "add_party_conviction", Unchecked), // -tdb-
    (Scopes::Character, "add_popularity", ScriptValue),
    (Scopes::Character, "add_prominence", ScriptValue),
    (Scopes::Character, "add_rival", Scope(Scopes::Character)),
    (Scopes::Character, "add_ruler_conviction", Unchecked),
    (Scopes::Character, "add_trait", Item(Item::CharacterTrait)),
    (Scopes::Character, "add_triggered_character_modifier", Unchecked), // -tdb-
    (Scopes::Character, "adopt", Scope(Scopes::Character)),
    (Scopes::Character, "banish", Scope(Scopes::Country)),
    (Scopes::Character, "change_mercenary_employer", Scope(Scopes::Country)),
    (Scopes::Character, "clear_ambition", Boolean),
    (Scopes::Character, "death", Unchecked), // -tdb-
    (Scopes::Character, "deify_character", Unchecked), // -tdb-
    (Scopes::Character, "divorce_character", Scope(Scopes::Character)),
    (Scopes::Character, "end_pregnancy", Boolean),
    (Scopes::Character, "force_add_trait", Item(Item::CharacterTrait)),
    (Scopes::Character, "give_office", Item(Item::Office)),
    (Scopes::Character, "marry_character", Scope(Scopes::Character)),
    (Scopes::Character, "move_country", Scope(Scopes::Country)),
    (Scopes::Character, "move_country_with_message", Scope(Scopes::Country)),
    (Scopes::Character, "pay_gold", Unchecked), // -tdb-
    (Scopes::Character, "remove_all_offices", Boolean),
    (Scopes::Character, "remove_as_governor", Boolean),
    (Scopes::Character, "remove_as_mercenary", Boolean),
    (Scopes::Character, "remove_as_researcher", Boolean),
    (Scopes::Character, "remove_character_modifier", Item(Item::Modifier)),
    (Scopes::Character, "remove_command", Boolean),
    (Scopes::Character, "remove_friend", Scope(Scopes::Character)),
    (Scopes::Character, "remove_holding", Scope(Scopes::Province)),
    (Scopes::Character, "remove_loyalty", Item(Item::Loyalty)),
    (Scopes::Character, "remove_office", Item(Item::Office)),
    (Scopes::Character, "remove_trait", Item(Item::CharacterTrait)),
    (Scopes::Character, "remove_triggered_character_modifier", Item(Item::Modifier)),
    (Scopes::Character, "set_ambition", Item(Item::Ambition)),
    (Scopes::Character, "set_as_minor_character", Scope(Scopes::Character)),
    (Scopes::Character, "set_character_religion", Item(Item::Religion)),
    (Scopes::Character, "set_culture", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Scopes::Character, "set_culture_same_as", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Scopes::Character, "set_family", Scope(Scopes::Family)),
    (Scopes::Character, "set_firstname", Item(Item::Localization)),
    (Scopes::Character, "set_home_country", Scope(Scopes::Country)),
    (Scopes::Character, "set_party_leader", Unchecked),
    (Scopes::Character, "update_character", Boolean),
    (Scopes::Character.union(Scopes::Unit).union(Scopes::Legion), "add_legion_history", Unchecked), // -tdb-
    (Scopes::Character.union(Scopes::Unit), "add_to_legion", Scope(Scopes::Legion)),
    (Scopes::Character, "disband_legion", Boolean),
    (Scopes::Character, "add_charisma", Integer),
    (Scopes::Character, "add_finesse", Integer),
    (Scopes::Character, "add_martial", Integer),
    (Scopes::Character, "add_zeal", Integer),
    (Scopes::Character, "make_pregnant", Unchecked), // -tdb-
    (Scopes::Governorship, "raise_legion", Unchecked), // TODO - these blocks should only check if there is a create_unit call inside of them
    (Scopes::Treasure, "destroy_treasure", Boolean),
    (Scopes::Treasure, "transfer_treasure_to_character", Scope(Scopes::Character)),
    (Scopes::Treasure, "transfer_treasure_to_country", Scope(Scopes::Country)),
    (Scopes::Treasure, "transfer_treasure_to_province", Scope(Scopes::Province)),
    (Scopes::Country, "add_aggressive_expansion", ScriptValue),
    (Scopes::Country, "add_alliance", Scope(Scopes::Country)),
    (Scopes::Country, "add_centralization", ScriptValue),
    (Scopes::Country, "add_country_modifier", Unchecked), // -tdb-
    (Scopes::Country, "add_guarantee", Scope(Scopes::Country)),
    (Scopes::Country, "add_innovation", Integer),
    (Scopes::Country, "add_legitimacy", ScriptValue),
    (Scopes::Country, "add_manpower", ScriptValue),
    (Scopes::Country, "add_military_access", Scope(Scopes::Country)),
    (Scopes::Country, "add_military_experience", ScriptValue),
    (Scopes::Country, "add_new_family", Unchecked),
    (Scopes::Country, "add_opinion", Unchecked), // -tdb-
    (Scopes::Country, "add_party_approval", Unchecked), // -tdb-
    (Scopes::Country, "add_political_influence", ScriptValue),
    (Scopes::Country, "add_research", Unchecked), // -tdb-
    (Scopes::Country, "add_stability", ScriptValue),
    (Scopes::Country, "add_to_war", Unchecked), // -tdb-
    (Scopes::Country, "add_treasury", ScriptValue),
    (Scopes::Country, "add_truce", Unchecked), // -tdb-
    (Scopes::Country, "add_tyranny", ScriptValue),
    (Scopes::Country, "add_war_exhaustion", ScriptValue),
    (Scopes::Country, "change_country_adjective", Item(Item::Localization)),
    (Scopes::Country, "change_country_color", Item(Item::NamedColor)),
    (Scopes::Country, "change_country_flag", Unchecked),
    (Scopes::Country, "change_country_name", Item(Item::Localization)),
    (Scopes::Country, "change_country_tag", Unchecked),
    (Scopes::Country, "change_government", Item(Item::Government)),
    (Scopes::Country, "change_law", Item(Item::Law)),
    (Scopes::Country, "create_character", Unchecked),  // -tdb-
    (Scopes::Country, "create_country_treasure", Unchecked),  // -tdb-
    (Scopes::Country, "create_family", Scope(Scopes::Character)),
    (Scopes::Country, "declare_war_with_wargoal", Unchecked),  // -tdb-
    (Scopes::Country, "imprison", Unchecked),  // -tdb-
    (Scopes::Country, "integrate_country_culture", Scope(Scopes::CountryCuluture)),
    (Scopes::Country, "make_subject", Unchecked),  // -tdb-
    (Scopes::Country, "pay_price", Item(Item::Price)),
    (Scopes::Country, "recalc_succession", Boolean),
    (Scopes::Country, "refund_price", Item(Item::Price)),
    (Scopes::Country, "release_prisoner", Unchecked),  // -tdb-
    (Scopes::Country, "remove_country_modifier", Item(Item::Modifier)),
    (Scopes::Country, "remove_gurantee", Scope(Scopes::Country)),
    (Scopes::Country, "remove_opinion", Unchecked),  // -tdb-
    (Scopes::Country, "remove_party_leadership", Scope(Scopes::Party)),
    (Scopes::Country, "reverse_add_opinion", Unchecked), // -tdb-
    (Scopes::Country, "set_as_coruler", Scope(Scopes::Character)),
    (Scopes::Country, "set_as_ruler", Scope(Scopes::Character)),
    (Scopes::Country, "set_capital", Scope(Scopes::Province)),
    (Scopes::Country, "set_country_heritage", Item(Item::Heritage)),
    (Scopes::Country, "set_country_religion", Scope(Scopes::Religion)),
    (Scopes::Country, "set_gender_equality", Boolean),
    (Scopes::Country, "set_graphical_culture", Item(Item::Ethnicity)),
    (Scopes::Country, "set_ignore_senate_approval", Boolean),
    (Scopes::Country, "set_legion_recruitment", Choice(&["enabled", "disabled", "capital"])),
    (Scopes::Country, "set_primary_culture", Item(Item::Culture)),
    (Scopes::Country, "start_civil_war", Scope(Scopes::Character)),
    (Scopes::Country, "update_allowed_parties", Boolean),
    (Scopes::Country, "set_party_agenda", Unchecked),
    (Scopes::Country, "break_alliance", Scope(Scopes::Country)),
    (Scopes::Legion, "add_commander", Scope(Scopes::Character)),
    (Scopes::Legion, "add_distinction", Item(Item::LegionDistinction)),
    (Scopes::Legion, "add_legion_unit", Unchecked),
    (Scopes::Legion, "move_legion", Scope(Scopes::Character)),
    (Scopes::Legion, "remove_commander", Scope(Scopes::Character)),
    (Scopes::Legion, "remove_distinction", Item(Item::LegionDistinction)),
    (Scopes::Legion, "remove_legion_unit", Unchecked),
    (Scopes::Siege, "add_breach", Integer),
    (Scopes::Legion, "create_unit", Unchecked), // -tdb-
    (Scopes::Unit, "add_food", ScriptValue),
    (Scopes::Unit, "add_loyal_subunit", Item(Item::Unit)),
    (Scopes::Unit, "add_morale", ScriptValue),
    (Scopes::Unit, "add_subunit", Item(Item::Unit)),
    (Scopes::Unit, "add_unit_modifier", Unchecked), // -tdb-
    (Scopes::Unit, "change_unit_owner", Scope(Scopes::Country)),
    (Scopes::Unit, "damage_unit_morale_percent", ScriptValue),
    (Scopes::Unit, "damage_unit_percent", ScriptValue),
    (Scopes::Unit, "destroy_unit", Boolean),
    (Scopes::Unit, "lock_unit", ScriptValue),
    (Scopes::Unit, "unlock_unit", ScriptValue),
    (Scopes::Unit, "remove_unit_loyalty", Boolean),
    (Scopes::Unit, "remove_unit_modifier", Item(Item::Modifier)),
    (Scopes::Unit, "set_as_commander", Scope(Scopes::Character)),
    (Scopes::Unit, "set_unit_size", Unchecked),
    (Scopes::Unit, "split_migrants_to", ScriptValue),
    (Scopes::PopType, "kill_pop", Boolean),
    (Scopes::PopType, "move_pop", Scope(Scopes::Province)),
    (Scopes::PopType, "set_pop_culture", Scope(Scopes::Culture)),
    (Scopes::PopType, "set_pop_culture_same_as", Scope(Scopes::PopType)),
    (Scopes::PopType, "set_pop_religion", Scope(Scopes::Religion)),
    (Scopes::PopType, "set_pop_religion_same_as", Scope(Scopes::PopType)),
    (Scopes::PopType, "set_pop_type", Item(Item::PopType)),
    (Scopes::SubUnit, "add_subunit_morale", ScriptValue),
    (Scopes::SubUnit, "add_subunit_strength", ScriptValue),
    (Scopes::SubUnit, "destroy_subunit", Boolean),
    (Scopes::SubUnit, "remove_subunit_loyalty", Yes | SubUnit),
    (Scopes::SubUnit, "set_personal_loyalty", Scope(Scopes::Character)),
    (Scopes::Family, "add_prestige", ScriptValue),
    (Scopes::Family, "move_family", Scope(Scopes::Country)),
    (Scopes::Family, "remove_family", Scope(Scopes::Country)),
    (Scopes::Country.union(Scopes::War), "force_white_peace", Scope(Scopes::War)),
    (Scopes::War, "remove_from_war", Scope(Scopes::Country)),
    (Scopes::Province, "add_building_level", Item(Item::Building)),
    (Scopes::Province, "add_civilization_value", ScriptValue),
    (Scopes::Province, "add_claim", Scope(Scopes::Country)),
    (Scopes::Province, "add_permanent_province_modifier", Unchecked), // -tdb-
    (Scopes::Province, "add_province_modifier", Unchecked), // -tdb-
    (Scopes::Province, "add_road_towards", Scope(Scopes::Province)),
    (Scopes::Province, "add_state_loyalty", ScriptValue),
    (Scopes::Province, "add_vfx", Unchecked),
    (Scopes::Province, "begin_great_work_construction", Unchecked),
    (Scopes::Province, "change_province_name", Unchecked),
    (Scopes::Province, "create_country", Unchecked),
    (Scopes::Province, "create_pop", Item(Item::PopType)),
    (Scopes::Province, "create_state_pop", Item(Item::PopType)),
    (Scopes::Province, "define_pop", Unchecked), // -tdb-
    (Scopes::Province, "finish_great_work_construction", Unchecked),
    (Scopes::Province, "hide_model", Unchecked),
    (Scopes::Province, "remove_building_level", Item(Item::Building)),
    (Scopes::Province, "remove_claim", Scope(Scopes::Country)),
    (Scopes::Province, "remove_province_deity", Boolean),
    (Scopes::Province, "remove_province_modifier", Item(Item::Modifier)),
    (Scopes::Province, "remove_vfx", Unchecked),
    (Scopes::Province, "set_as_governor", Scope(Scopes::Character)),
    (Scopes::Province, "set_city_status", Item(Item::ProvinceRank)),
    (Scopes::Province, "set_conquered_by", Scope(Scopes::Country)),
    (Scopes::Province, "set_controller", Scope(Scopes::Country)),
    (Scopes::Province, "set_owned_by", Scope(Scopes::Country)),
    (Scopes::Province, "set_province_deity", Scope(Scopes::Deity)),
    (Scopes::Province, "set_trade_goods", Scope(Scopes::TradeGood)),
    (Scopes::Province, "show_animated_text", Unchecked),
    (Scopes::Province, "show_model", Unchecked),
    (Scopes::CountryCulture, "add_country_culture_modifier", Unchecked), // -tdb-
    (Scopes::CountryCulture, "add_integration_progress", ScriptValue),
    (Scopes::CountryCulture, "remove_country_culture_modifier", Item(Item::Modifier)),
    (Scopes::CountryCulture, "set_country_culture_right", Item(Item::PopType)),
    (Scopes::None, "reset_scoring", Scope(Scopes::Country)),
    (Scopes::None, "add_to_global_variable_list", VB(EvB::AddToVariableList)),
    (Scopes::all_but_none(), "add_to_list", VV(EvV::AddToList)),
    (Scopes::None, "add_to_local_variable_list", VB(EvB::AddToVariableList)),
    (Scopes::all_but_none(), "add_to_temporary_list", VV(EvV::AddToList)),
    (Scopes::None, "add_to_variable_list", VB(EvB::AddToVariableList)),
    (Scopes::None, "assert_if", Unchecked),
    (Scopes::None, "assert_read", Unchecked),
    (Scopes::None, "break", Boolean),
    (Scopes::None, "break_if", Control),
    (Scopes::None, "change_global_variable", VB(EvB::ChangeVariable)),
    (Scopes::None, "change_local_variable", VB(EvB::ChangeVariable)),
    (Scopes::None, "change_variable", VB(EvB::ChangeVariable)),
    (Scopes::None, "clamp_global_variable", VB(EvB::ClampVariable)),
    (Scopes::None, "clamp_local_variable", VB(EvB::ClampVariable)),
    (Scopes::None, "clamp_variable", VB(EvB::ClampVariable)),
    (Scopes::None, "clear_global_variable_list", Unchecked),
    (Scopes::None, "clear_local_variable_list", Unchecked),
    (Scopes::None, "clear_saved_scope", Unchecked),
    (Scopes::None, "clear_variable_list", Unchecked),
    (Scopes::None, "custom_label", ControlOrLabel),
    (Scopes::None, "custom_tooltip", ControlOrLabel),
    (Scopes::None, "debug_log", Unchecked),
    (Scopes::None, "debug_log_scopes", Boolean),
    (Scopes::None, "else", Control),
    (Scopes::None, "else_if", Control),
    (Scopes::None, "hidden_effect", Control),
    (Scopes::None, "if", Control),
    (Scopes::None, "random", Control),
    (Scopes::None, "random_list", VB(EvB::RandomList)),
    (Scopes::all_but_none(), "remove_from_list", VV(EvV::RemoveFromList)),
    (Scopes::None, "remove_global_variable", Unchecked),
    (Scopes::None, "remove_list_global_variable", VB(EvB::AddToVariableList)),
    (Scopes::None, "remove_list_local_variable", VB(EvB::AddToVariableList)),
    (Scopes::None, "remove_list_variable", VB(EvB::AddToVariableList)),
    (Scopes::None, "remove_local_variable", Unchecked),
    (Scopes::None, "remove_variable", Unchecked),
    (Scopes::None, "round_global_variable", VB(EvB::RoundVariable)),
    (Scopes::None, "round_local_variable", VB(EvB::RoundVariable)),
    (Scopes::None, "round_variable", VB(EvB::RoundVariable)),
    (Scopes::all_but_none(), "save_scope_as", VV(EvV::SaveScope)),
    (Scopes::all_but_none(), "save_temporary_scope_as", VV(EvV::SaveScope)),
    (Scopes::None, "set_global_variable", VBv(EvBv::SetVariable)),
    (Scopes::None, "set_local_variable", VBv(EvBv::SetVariable)),
    (Scopes::None, "set_variable", VBv(EvBv::SetVariable)),
    (Scopes::None, "show_as_tooltip", Control),
    (Scopes::None, "switch", VB(EvB::Switch)),
    (Scopes::None, "trigger_event", Unchecked),
    (Scopes::None, "while", Control),
];
