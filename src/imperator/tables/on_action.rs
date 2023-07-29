use fnv::FnvHashMap;
use once_cell::sync::Lazy; // replace with std version once it's stable

use crate::block::BV;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::parse::pdxfile::parse_pdx_internal;
use crate::scopes::{scope_from_snake_case, Scopes};
use crate::token::Token;

#[derive(Debug, Clone)]
struct OnActionScopeContext {
    root: Scopes,
    names: Vec<(String, Scopes)>,
    lists: Vec<(String, Scopes)>,
}

pub fn on_action_scopecontext(key: &Token, _data: &Everything) -> Option<ScopeContext> {
    if let Some(oa_sc) = ON_ACTION_SCOPES.get(key.as_str()) {
        let mut sc = ScopeContext::new(oa_sc.root, key);
        for (name, s) in &oa_sc.names {
            sc.define_name(name, *s, key);
        }
        for (list, s) in &oa_sc.lists {
            sc.define_list(list, *s, key);
        }
        sc.set_strict_scopes(false);
        return Some(sc);
    }

    None
}

static ON_ACTION_SCOPES: Lazy<FnvHashMap<String, OnActionScopeContext>> = Lazy::new(|| {
    build_on_action_hashmap(
        "
	yearly_country_pulse = {
		root = country
	}

    monthly_country_pulse = yearly_country_pulse
    on_deficit_pulse = yearly_country_pulse
    on_state_secession = yearly_country_pulse
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
        target = character
    }

    on_civil_war_won = on_civil_war_lost

    on_culture_reduced_right = {
        root = none
        target_culture = culture
        target = culture
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
        root = none
        target_culture = culture
        target = pop_type
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
        target_culture = culture
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
        root = none
    }

    on_reign_ending_successor = on_game_initialized
    on_deified_ruler_death = on_game_initialized
    on_great_work_anniversary = on_game_initialized
    on_battle_won = on_game_initialized

    on_legion_raised = {
        root = legion
    }

    on_legion_dissolved = on_legion_raised

    on_subject_defect = {
        root = none
        future_overlord = country
        target_subject = country
    }

    on_giving_birth = {
        root = character
        newborn = character
    }
",
    )
});

fn build_on_action_hashmap(description: &'static str) -> FnvHashMap<String, OnActionScopeContext> {
    let mut hash: FnvHashMap<String, OnActionScopeContext> = FnvHashMap::default();

    let mut block = parse_pdx_internal(description, "on action builtin scopes");
    for item in block.drain() {
        let field = item.get_field().expect("internal error");
        match field.bv() {
            BV::Value(token) => {
                // key1 = key2 means copy from key2
                let value = hash.get(token.as_str()).expect("internal error");
                hash.insert(field.key().to_string(), value.clone());
            }
            BV::Block(block) => {
                let root = block.get_field_value("root").expect("internal error");
                let root = scope_from_snake_case(root.as_str()).expect("internal error");
                let mut value = OnActionScopeContext { root, names: Vec::new(), lists: Vec::new() };
                for (key, token) in block.iter_assignments() {
                    if key.is("root") {
                        continue;
                    }
                    let s = scope_from_snake_case(token.as_str()).expect("internal error");
                    value.names.push((key.to_string(), s));
                }
                for (key, block) in block.iter_definitions() {
                    if key.is("list") {
                        for (key, token) in block.iter_assignments() {
                            let s = scope_from_snake_case(token.as_str()).expect("internal error");
                            value.lists.push((key.to_string(), s));
                        }
                    }
                }
                hash.insert(field.key().to_string(), value);
            }
        }
    }

    hash
}
