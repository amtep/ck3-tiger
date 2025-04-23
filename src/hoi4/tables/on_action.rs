use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;

pub const ON_ACTION_SCOPES: &str = "";

// LAST UPDATED HOI4 VERSION 1.16.4
pub fn on_action_scopecontext_hoi4(key: &Token, data: &Everything) -> Option<ScopeContext> {
    // TODO: verify scopes for on_border_war_lost
    // TODO: verify scopes for on_faction_formed
    // TODO: verify scopes for on_unit_leader_created
    match key.as_str().to_lowercase().as_ref() {
        "on_startup" => {
            return Some(ScopeContext::new(Scopes::None, key));
        }
        "on_new_term_election"
        | "on_wargoal_expire"
        | "on_uncapitulation"
        | "on_government_change"
        | "on_coup_succeeded"
        | "on_war"
        | "on_peaceconference_ended"
        // TODO: temp var old_ideology_token is available for effects
        | "on_ruling_party_change"
        | "on_government_exiled"
        | "on_daily"
        | "on_weekly"
        | "on_monthly"
        | "on_yearly"
        | "on_peace" => {
            return Some(ScopeContext::new(Scopes::Country, key));
        }
        "on_ace_promoted"
        | "on_ace_killed"
        | "on_ace_killed_on_accident"
        | "on_non_ace_killed_other_ace" => {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.push_as_from(Scopes::Ace, key);
            return Some(sc);
        }
        "on_aces_killed_each_other" | "on_ace_killed_by_ace" | "on_ace_killed_other_ace" => {
            // THIS = country, FROM = ace, PREV = enemy ace
            let mut sc = ScopeContext::new_with_prev(Scopes::Country, Scopes::Ace, key);
            sc.push_as_from(Scopes::Ace, key);
            return Some(sc);
        }
        "on_justifying_wargoal_pulse"
        | "on_leave_faction"
        | "on_create_faction"
        | "on_offer_join_faction"
        | "on_join_faction"
        | "on_declare_war"
        | "on_faction_formed"
        | "on_capitulation"
        | "on_capitulation_immediate"
        | "on_civil_war_end"
        | "on_puppet"
        | "on_release_as_puppet"
        | "on_annex"
        | "on_war_relation_added"
        | "on_subject_free"
        | "on_subject_autonomy_level_change"
        | "on_subject_annexed"
        | "on_release_as_free"
        | "on_exile_government_reinstated"
        | "on_civil_war_end_before_annexation"
        | "on_pride_of_the_fleet_sunk"
        | "on_assume_faction_leadership"
        | "on_fully_decrypted_cipher"
        | "on_activated_active_decryption_bonuses"
        | "on_peaceconference_started"
        | "on_liberate"
        | "on_send_volunteers"
        | "on_join_allies"
        | "on_call_allies"
        | "on_before_peace_conference_start" => {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_nuke_drop" => {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.push_as_from(Scopes::State, key);
            return Some(sc);
        }
        "on_border_war_lost" => {
            return Some(ScopeContext::new(Scopes::State, key));
        }
        "on_state_control_changed" => {
            // ROOT is new controller, FROM is old controller, FROM.FROM is state ID
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.push_as_from(Scopes::State, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_army_leader_daily" | "on_army_leader_won_combat" | "on_army_leader_lost_combat"
        | "on_operative_recruited" | "on_operative_created" => {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_army_leader_promoted" => {
            return Some(ScopeContext::new(Scopes::Character, key));
        }
        "on_naval_invasion" | "on_paradrop" => {
            // ROOT = country that invades, THIS = state that is invaded, FROM = state that the invasion started
            let mut sc = ScopeContext::new_separate_root(Scopes::Country, Scopes::State, key);
            sc.push_as_from(Scopes::State, key);
            return Some(sc);
        }
        "on_unit_leader_promote_from_ranks_veteran"
            | "on_unit_leader_promote_from_ranks_green" => {
            // Unit leader scope, FROM is unit
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.push_as_from(Scopes::Division, key);
            return Some(sc);
        }
        "on_add_history" => {
            return Some(ScopeContext::new(Scopes::Division, key));
        }
        "on_units_paradropped_in_state" => {
            // ROOT = state that was dropped into, FROM = dropping country
            let mut sc = ScopeContext::new(Scopes::State, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_host_changed_from_capitulation" => {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.push_as_from(Scopes::Country, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_unit_leader_created" | "on_unit_leader_level_up" => {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.push_as_from(Scopes::Country, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_operative_on_mission_spotted" | "on_operative_captured" | "on_operative_death" => {
            let mut sc = ScopeContext::new_separate_root(Scopes::Country, Scopes::Character, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_operative_detected_during_operation" => {
            let mut sc = ScopeContext::new_separate_root(Scopes::Country, Scopes::Character, key);
            sc.push_as_from(Scopes::State, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_operation_completed" => {
            let mut sc = ScopeContext::new_separate_root(Scopes::Country, Scopes::Operation, key);
            sc.push_as_from(Scopes::Country, key);
            return Some(sc);
        }
        "on_mio_size_increased"
        | "on_mio_design_team_assigned_to_tech"
        | "on_mio_design_team_assigned_to_variant"
        | "on_mio_industrial_manufacturer_assigned"
        | "on_mio_tech_research_cancelled"
        | "on_mio_tech_research_completed"
        | "on_mio_industrial_manufacturer_unassigned"
            => {
            return Some(ScopeContext::new(Scopes::IndustrialOrg, key));
        }
        "on_project_completion" => {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.push_as_from(Scopes::SpecialProject, key);
            return Some(sc);
        }
        _ => (),
    }

    for pfx in &["on_daily_", "on_weekly_", "on_monthly_", "on_yearly_"] {
        if let Some(tag) = key.strip_prefix(pfx) {
            data.verify_exists(Item::CountryTag, &tag);
            return Some(ScopeContext::new(Scopes::Country, key));
        }
    }

    // In Hoi4 you can't define new on_actions
    let msg = format!("unknown on_action `{key}`");
    err(ErrorKey::UnknownField).msg(msg).loc(key).push();
    None
}
