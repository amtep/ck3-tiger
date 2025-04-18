#![allow(non_upper_case_globals)]

use std::sync::LazyLock;

use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::modif::ModifKinds;
use crate::report::{report, ErrorKey, Severity};
use crate::token::Token;

/// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
/// Returns None otherwise.
pub fn lookup_modif(name: &Token, data: &Everything, warn: Option<Severity>) -> Option<ModifKinds> {
    let name_lc = Lowercase::new(name.as_str());

    if let result @ Some(_) = MODIF_MAP.get(&name_lc).copied() {
        return result;
    }

    if let Some(info) = MODIF_REMOVED_MAP.get(&name_lc).copied() {
        if let Some(sev) = warn {
            let msg = format!("{name} has been removed");
            report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
        }
        return Some(ModifKinds::all());
    }

    // Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    // state_resource_cost_$Resource$
    // state_resource_$Resource$
    // temporary_state_resource_$Resource$
    for &pfx in &["state_resource_cost_", "state_resource_", "temporary_state_resource_"] {
        if let Some(part) = name_lc.strip_prefix_unchecked(pfx) {
            maybe_warn(Item::Resource, &part, name, data, warn);
            return Some(ModifKinds::State);
        }
    }

    // country_resource_cost_$Resource$
    // country_resource_$Resource$
    for &pfx in &["country_resource_cost_", "country_resource_"] {
        if let Some(part) = name_lc.strip_prefix_unchecked(pfx) {
            maybe_warn(Item::Resource, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // state_resources_$Resource$_factor
    if let Some(part) = name_lc.strip_prefix_unchecked("state_resources_") {
        if let Some(part) = part.strip_suffix_unchecked("_factor") {
            maybe_warn(Item::Resource, &part, name, data, warn);
            return Some(ModifKinds::State);
        }
    }

    // state_production_speed_$Building$_factor
    // state_repair_speed_$Building$_factor
    // production_speed_$Building$_factor
    // production_cost_$Building$_factor
    // repair_speed_$Building$_factor
    if let Some(part) = name_lc.strip_suffix_unchecked("_factor") {
        for &pfx in &["state_production_speed_", "state_repair_speed_"] {
            if let Some(part) = part.strip_prefix_unchecked(pfx) {
                maybe_warn(Item::Building, &part, name, data, warn);
                return Some(ModifKinds::State);
            }
        }
        for &pfx in &["production_speed_", "production_cost_", "repair_speed_"] {
            if let Some(part) = part.strip_prefix_unchecked(pfx) {
                maybe_warn(Item::Building, &part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // state_$Building$_max_level_terrain_limit
    // $Building$_max_level_terrain_limit
    if let Some(part) = name_lc.strip_suffix_unchecked("_max_level_terrain_limit") {
        if let Some(part) = part.strip_prefix_unchecked("state_") {
            maybe_warn(Item::Building, &part, name, data, warn);
            return Some(ModifKinds::State);
        }
        maybe_warn(Item::Building, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // $IdeaCategory$_category_type_cost_factor
    if let Some(part) = name_lc.strip_suffix_unchecked("_category_type_cost_factor") {
        maybe_warn(Item::IdeaCategory, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // module_$EquipmentModule$_design_cost_factor
    // unit_$SubUnit$_design_cost_factor
    // $Equipment$_design_cost_factor
    if let Some(part) = name_lc.strip_suffix_unchecked("_design_cost_factor") {
        if let Some(part) = part.strip_prefix_unchecked("module_") {
            maybe_warn(Item::EquipmentModule, &part, name, data, warn);
            return Some(ModifKinds::Naval | ModifKinds::Country | ModifKinds::Army);
        } else if let Some(part) = part.strip_prefix_unchecked("unit_") {
            maybe_warn(Item::SubUnit, &part, name, data, warn);
            return Some(ModifKinds::Naval | ModifKinds::Country | ModifKinds::Army);
        }
        maybe_warn(Item::Equipment, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // $IdeaGroup$_cost_factor
    // $Technology$_cost_factor
    if let Some(part) = name_lc.strip_suffix_unchecked("_cost_factor") {
        if let Some(sev) = warn {
            if !data.item_exists_lc(Item::IdeaGroup, &part)
                && !data.item_exists_lc(Item::Technology, &part)
            {
                let msg = format!("{part} not found as idea group or technology");
                let info = format!("so the modifier {name} does not exist");
                report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
            }
        }
        return Some(ModifKinds::Country);
    }

    // $IdeologyGroup$_drift
    // $IdeologyGroup$_acceptance
    for &sfx in &["_drift", "_acceptance"] {
        if let Some(part) = name_lc.strip_suffix_unchecked(sfx) {
            maybe_warn(Item::IdeologyGroup, &part, name, data, warn);
            return Some(ModifKinds::Politics);
        }
    }

    // $CombatTactic$_preferred_weight_factor
    if let Some(part) = name_lc.strip_suffix_unchecked("_preferred_weight_factor") {
        maybe_warn(Item::CombatTactic, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // production_cost_max_$NavalEquipment$
    if let Some(part) = name_lc.strip_prefix_unchecked("production_cost_max_") {
        maybe_warn(Item::CombatTactic, &part, name, data, warn);
        return Some(ModifKinds::Naval);
    }

    // experience_gain_$Unit$_training_factor
    // experience_gain_$Unit$_combat_factor
    if let Some(part) = name_lc.strip_prefix_unchecked("experience_gain_") {
        if let Some(part) = part.strip_suffix_unchecked("_training_factor") {
            maybe_warn(Item::Unit, &part, name, data, warn);
            return Some(ModifKinds::Naval | ModifKinds::Country);
        }
        if let Some(part) = part.strip_suffix_unchecked("_combat_factor") {
            maybe_warn(Item::Unit, &part, name, data, warn);
            return Some(ModifKinds::Naval | ModifKinds::Country);
        }
    }

    // unit_$Unit$_design_cost_factor
    if let Some(part) = name_lc.strip_prefix_unchecked("unit_") {
        if let Some(part) = part.strip_suffix_unchecked("_design_cost_factor") {
            maybe_warn(Item::Unit, &part, name, data, warn);
            return Some(ModifKinds::Naval | ModifKinds::Country | ModifKinds::Army);
        }
    }

    // trait_$Trait$_xp_gain_factor
    // $Trait$_xp_gain_factor -- used if the trait has prefix 'trait_'
    if let Some(part) = name_lc.strip_suffix_unchecked("_xp_gain_factor") {
        if !data.item_exists(Item::CountryLeaderTrait, part.as_str())
            && !data.item_exists(Item::UnitLeaderTrait, part.as_str())
        {
            if let Some(part) = part.strip_prefix_unchecked("trait_") {
                if let Some(sev) = warn {
                    if !data.item_exists(Item::CountryLeaderTrait, part.as_str())
                        && !data.item_exists(Item::UnitLeaderTrait, part.as_str())
                    {
                        let msg =
                            format!("could not find {part} as country leader or unit leader trait");
                        let info = format!("so the modifier {name} does not exist");
                        report(ErrorKey::MissingItem, sev)
                            .strong()
                            .msg(msg)
                            .info(info)
                            .loc(name)
                            .push();
                    }
                }
                return Some(ModifKinds::Naval | ModifKinds::Country | ModifKinds::Army);
            }
        }
        if let Some(sev) = warn {
            if !data.item_exists(Item::CountryLeaderTrait, part.as_str())
                && !data.item_exists(Item::UnitLeaderTrait, part.as_str())
            {
                let msg = format!("could not find {part} as country leader or unit leader trait");
                let info = format!("so the modifier {name} does not exist");
                report(ErrorKey::MissingItem, sev).strong().msg(msg).info(info).loc(name).push();
            }
        }
        return Some(ModifKinds::Naval | ModifKinds::Country | ModifKinds::Army);
    }

    // $SpecialProject$_speed_factor
    // undocumented: $SpecialProjectTag$_speed_factor
    // undocumented: $Specialization$_speed_factor
    if let Some(part) = name_lc.strip_suffix_unchecked("_speed_factor") {
        if let Some(sev) = warn {
            if !data.item_exists(Item::SpecialProject, part.as_str())
                && !data.item_exists(Item::SpecialProjectTag, part.as_str())
                && !data.item_exists(Item::Specialization, part.as_str())
            {
                let msg =
                    format!("could not find {part} as special project, special project tag or specialization");
                let info = format!("so the modifier {name} does not exist");
                report(ErrorKey::MissingItem, sev).strong().msg(msg).info(info).loc(name).push();
            }
        }
        return Some(ModifKinds::Country | ModifKinds::State);
    }

    // $Operation$_risk
    // $Operation$_outcome
    // $Operation$_cost
    for &sfx in &["_risk", "_outcome", "_cost"] {
        if let Some(part) = name_lc.strip_suffix_unchecked(sfx) {
            maybe_warn(Item::Operation, &part, name, data, warn);
            return Some(ModifKinds::IntelligenceAgency);
        }
    }

    None
}

fn maybe_warn(itype: Item, s: &Lowercase, name: &Token, data: &Everything, warn: Option<Severity>) {
    if let Some(sev) = warn {
        if !data.item_exists_lc(itype, s) {
            let msg = format!("could not find {itype} {s}");
            let info = format!("so the modifier {name} does not exist");
            report(ErrorKey::MissingItem, sev).strong().msg(msg).info(info).loc(name).push();
        }
    }
}

/// Return the modifier localization key
pub fn modif_loc_hoi4(name: &Token, _data: &Everything) -> String {
    // TODO: check hoi4 exceptions
    format!("MODIFIER_{}", name.as_str().to_uppercase())
}

static MODIF_MAP: LazyLock<TigerHashMap<Lowercase<'static>, ModifKinds>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (s, kind) in MODIF_TABLE.iter().copied() {
        hash.insert(Lowercase::new_unchecked(s), kind);
    }
    hash
});

/// LAST UPDATED HOI4 VERSION 1.16.4
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, ModifKinds)] = &[
    ("acclimatization_cold_climate_gain_factor", ModifKinds::Army),
    ("acclimatization_hot_climate_gain_factor", ModifKinds::Army),
    ("ace_effectiveness_factor", ModifKinds::Air),
    ("additional_brigade_column_size", ModifKinds::Country),
    ("agency_upgrade_time", ModifKinds::Country),
    ("ai_badass_factor", ModifKinds::Ai),
    ("ai_call_ally_desire_factor", ModifKinds::Ai),
    ("ai_desired_divisions_factor", ModifKinds::Ai),
    ("ai_focus_aggressive_factor", ModifKinds::Ai),
    ("ai_focus_aviation_factor", ModifKinds::Ai),
    ("ai_focus_defense_factor", ModifKinds::Ai),
    ("ai_focus_military_advancements_factor", ModifKinds::Ai),
    ("ai_focus_military_equipment_factor", ModifKinds::Ai),
    ("ai_focus_naval_air_factor", ModifKinds::Ai),
    ("ai_focus_naval_factor", ModifKinds::Ai),
    ("ai_focus_peaceful_factor", ModifKinds::Ai),
    ("ai_focus_war_production_factor", ModifKinds::Ai),
    ("ai_get_ally_desire_factor", ModifKinds::Ai),
    ("ai_join_ally_desire_factor", ModifKinds::Ai),
    ("ai_license_acceptance", ModifKinds::Ai),
    ("air_accidents", ModifKinds::Air),
    ("air_accidents_factor", ModifKinds::Air),
    ("air_ace_bonuses_factor", ModifKinds::Air),
    ("air_ace_generation_chance_factor", ModifKinds::Air),
    ("air_advisor_cost_factor", ModifKinds::Air),
    ("air_agility_factor", ModifKinds::Air),
    ("air_attack_factor", ModifKinds::Air),
    ("air_bombing_targetting", ModifKinds::Air),
    ("air_carrier_night_penalty_reduction_factor", ModifKinds::Air),
    ("air_cas_efficiency", ModifKinds::Air),
    ("air_cas_present_factor", ModifKinds::Air),
    ("air_close_air_support_org_damage_factor", ModifKinds::Air),
    ("air_defence_factor", ModifKinds::Air),
    ("air_detection", ModifKinds::Air),
    ("air_doctrine_cost_factor", ModifKinds::Country),
    ("air_equipment_upgrade_xp_cost", ModifKinds::Country),
    ("air_escort_efficiency", ModifKinds::Air),
    ("air_fuel_consumption_factor", ModifKinds::Air),
    ("air_home_defence_factor", ModifKinds::Air),
    ("air_intercept_efficiency", ModifKinds::Air),
    ("air_interception_detect_factor", ModifKinds::Air),
    ("air_manpower_requirement_factor", ModifKinds::Air),
    ("air_maximum_speed_factor", ModifKinds::Air),
    ("air_mission_efficiency", ModifKinds::Air),
    ("air_mission_xp_gain_factor", ModifKinds::Air),
    ("air_nav_efficiency", ModifKinds::Air),
    ("air_night_penalty", ModifKinds::Air),
    ("air_power_projection_factor", ModifKinds::Air),
    ("air_range_factor", ModifKinds::Air),
    ("air_strategic_bomber_bombing_factor", ModifKinds::Air),
    ("air_strategic_bomber_defence_factor", ModifKinds::Air),
    ("air_strategic_bomber_night_penalty", ModifKinds::Air),
    ("air_superiority_bonus_in_combat", ModifKinds::Army),
    ("air_superiority_detect_factor", ModifKinds::Air),
    ("air_superiority_efficiency", ModifKinds::Air),
    ("air_training_xp_gain_factor", ModifKinds::Air),
    ("air_untrained_pilots_penalty_factor", ModifKinds::Air),
    ("air_volunteer_cap", ModifKinds::Country),
    ("air_weather_penalty", ModifKinds::Air),
    ("air_wing_xp_loss_when_killed_factor", ModifKinds::Air),
    ("airforce_intel_decryption_bonus", ModifKinds::IntelligenceAgency),
    ("airforce_intel_factor", ModifKinds::IntelligenceAgency),
    ("airforce_intel_to_others", ModifKinds::Country),
    ("amphibious_invasion", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("amphibious_invasion_defence", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("annex_cost_factor", ModifKinds::Aggressive.union(ModifKinds::Peace)),
    ("armor_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_advisor_cost_factor", ModifKinds::Army),
    ("army_armor_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_armor_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_armor_speed_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_artillery_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_artillery_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_attack_against_major_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_attack_against_minor_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_attack_speed_factor", ModifKinds::Army),
    ("army_bonus_air_superiority_factor", ModifKinds::Air),
    ("army_breakthrough_against_major_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_breakthrough_against_minor_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_core_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_core_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_defence_against_major_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_defence_against_minor_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_fuel_capacity_factor", ModifKinds::Army),
    ("army_fuel_consumption_factor", ModifKinds::Army),
    ("army_infantry_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("army_infantry_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("army_intel_decryption_bonus", ModifKinds::IntelligenceAgency),
    ("army_intel_factor", ModifKinds::IntelligenceAgency),
    ("army_intel_to_others", ModifKinds::Country),
    ("army_leader_cost_factor", ModifKinds::UnitLeader),
    ("army_leader_start_attack_level", ModifKinds::UnitLeader),
    ("army_leader_start_defense_level", ModifKinds::UnitLeader),
    ("army_leader_start_level", ModifKinds::UnitLeader),
    ("army_leader_start_logistics_level", ModifKinds::UnitLeader),
    ("army_leader_start_planning_level", ModifKinds::UnitLeader),
    ("army_morale", ModifKinds::Aggressive.union(ModifKinds::Army).union(ModifKinds::Defensive)),
    (
        "army_morale_factor",
        ModifKinds::Aggressive.union(ModifKinds::Army).union(ModifKinds::Defensive),
    ),
    ("army_org", ModifKinds::Aggressive.union(ModifKinds::Army).union(ModifKinds::Defensive)),
    (
        "army_org_factor",
        ModifKinds::Aggressive.union(ModifKinds::Army).union(ModifKinds::Defensive),
    ),
    ("army_org_regain", ModifKinds::Army),
    ("army_speed_factor", ModifKinds::Army),
    ("army_speed_factor_for_controller", ModifKinds::Army.union(ModifKinds::State)),
    ("army_strength_factor", ModifKinds::Army),
    ("assign_army_leader_cp_cost", ModifKinds::Army.union(ModifKinds::Country)),
    ("assign_navy_leader_cp_cost", ModifKinds::Country.union(ModifKinds::Naval)),
    ("attack_bonus_against", ModifKinds::Army),
    ("attack_bonus_against_cores", ModifKinds::Army),
    ("attrition", ModifKinds::Aggressive.union(ModifKinds::Army).union(ModifKinds::Defensive)),
    (
        "attrition_for_controller",
        ModifKinds::Aggressive.union(ModifKinds::Army).union(ModifKinds::Defensive),
    ),
    ("autonomy_gain", ModifKinds::Autonomy),
    ("autonomy_gain_global_factor", ModifKinds::Autonomy),
    ("autonomy_gain_ll_to_overlord", ModifKinds::Autonomy),
    ("autonomy_gain_ll_to_overlord_factor", ModifKinds::Autonomy),
    ("autonomy_gain_ll_to_subject", ModifKinds::Autonomy),
    ("autonomy_gain_ll_to_subject_factor", ModifKinds::Autonomy),
    ("autonomy_gain_trade", ModifKinds::Autonomy),
    ("autonomy_gain_trade_factor", ModifKinds::Autonomy),
    ("autonomy_gain_warscore", ModifKinds::Autonomy),
    ("autonomy_gain_warscore_factor", ModifKinds::Autonomy),
    ("autonomy_manpower_share", ModifKinds::Autonomy),
    ("base_fuel_gain", ModifKinds::Country),
    ("base_fuel_gain_factor", ModifKinds::Country),
    ("boost_ideology_mission_factor", ModifKinds::IntelligenceAgency),
    ("boost_resistance_factor", ModifKinds::IntelligenceAgency),
    ("breakthrough_bonus_against", ModifKinds::Army),
    ("breakthrough_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("can_master_build_for_us", ModifKinds::Autonomy),
    ("cannot_use_abilities", ModifKinds::UnitLeader),
    (
        "carrier_capacity_penalty_reduction",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("carrier_night_traffic", ModifKinds::Air),
    (
        "carrier_sortie_hours_delay",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "carrier_traffic",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("cas_damage_reduction", ModifKinds::Army),
    ("cavalry_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("cavalry_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("choose_preferred_tactics_cost", ModifKinds::Country),
    ("cic_construction_boost", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("cic_construction_boost_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("cic_to_overlord_factor", ModifKinds::Autonomy),
    ("cic_to_target_factor", ModifKinds::Country),
    ("civil_war_involvement_tension", ModifKinds::Country),
    ("civilian_factory_use", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("civilian_intel_decryption_bonus", ModifKinds::IntelligenceAgency),
    ("civilian_intel_factor", ModifKinds::IntelligenceAgency),
    ("civilian_intel_to_others", ModifKinds::Country),
    ("coastal_bunker_effectiveness_factor", ModifKinds::Country.union(ModifKinds::State)),
    ("combat_width_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("command_abilities_cost_factor", ModifKinds::Army.union(ModifKinds::Country)),
    ("command_power_gain", ModifKinds::Army.union(ModifKinds::Country)),
    ("command_power_gain_mult", ModifKinds::Army.union(ModifKinds::Country)),
    ("commando_trait_chance_factor", ModifKinds::IntelligenceAgency),
    ("compliance_gain", ModifKinds::Country.union(ModifKinds::State)),
    ("compliance_growth", ModifKinds::Country.union(ModifKinds::State)),
    ("compliance_growth_on_our_occupied_states", ModifKinds::Country.union(ModifKinds::State)),
    ("conscription", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("conscription_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("consumer_goods_expected_value", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("consumer_goods_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("control_trade_mission_factor", ModifKinds::IntelligenceAgency),
    ("conversion_cost_civ_to_mil_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("conversion_cost_mil_to_civ_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    (
        "convoy_escort_efficiency",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "convoy_raiding_efficiency_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "convoy_retreat_speed",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("coordination_bonus", ModifKinds::Aggressive.union(ModifKinds::Army)),
    (
        "critical_receive_chance",
        ModifKinds::Country.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("crypto_department_enabled", ModifKinds::IntelligenceAgency),
    ("crypto_strength", ModifKinds::IntelligenceAgency),
    ("decryption", ModifKinds::Aggressive.union(ModifKinds::Country)),
    ("decryption_factor", ModifKinds::Aggressive.union(ModifKinds::Country)),
    ("decryption_power", ModifKinds::IntelligenceAgency),
    ("decryption_power_factor", ModifKinds::IntelligenceAgency),
    ("defence", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("defense_bonus_against", ModifKinds::Army),
    ("defense_impact_on_blueprint_stealing", ModifKinds::IntelligenceAgency),
    ("defensive_war_stability_factor", ModifKinds::Country),
    ("dig_in_speed", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("dig_in_speed_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("diplomatic_pressure_mission_factor", ModifKinds::IntelligenceAgency),
    ("disable_strategic_redeployment", ModifKinds::Army.union(ModifKinds::State)),
    ("disable_strategic_redeployment_for_controller", ModifKinds::Army.union(ModifKinds::State)),
    ("disabled_ideas", ModifKinds::Country),
    ("dockyard_donations", ModifKinds::GovernmentInExile),
    ("dont_lose_dig_in_on_attack", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("drift_defence_factor", ModifKinds::Politics),
    ("embargo_cost_factor", ModifKinds::Country.union(ModifKinds::Politics)),
    ("embargo_threshold_factor", ModifKinds::Country.union(ModifKinds::Politics)),
    ("encryption", ModifKinds::Country.union(ModifKinds::Defensive)),
    ("encryption_factor", ModifKinds::Country.union(ModifKinds::Defensive)),
    ("enemy_army_bonus_air_superiority_factor", ModifKinds::Air),
    ("enemy_army_speed_factor", ModifKinds::Army.union(ModifKinds::State)),
    (
        "enemy_attrition",
        ModifKinds::Aggressive
            .union(ModifKinds::Army)
            .union(ModifKinds::Defensive)
            .union(ModifKinds::State),
    ),
    ("enemy_declare_war_tension", ModifKinds::Aggressive.union(ModifKinds::Country)),
    ("enemy_intel_network_gain_factor_over_occupied_tag", ModifKinds::State),
    ("enemy_justify_war_goal_time", ModifKinds::Aggressive.union(ModifKinds::Country)),
    ("enemy_local_supplies", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("enemy_operative_capture_chance_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("enemy_operative_detection_chance", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("enemy_operative_detection_chance_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    (
        "enemy_operative_detection_chance_factor_over_occupied_tag",
        ModifKinds::Country.union(ModifKinds::UnitLeader),
    ),
    (
        "enemy_operative_detection_chance_over_occupied_tag",
        ModifKinds::Country.union(ModifKinds::UnitLeader),
    ),
    (
        "enemy_operative_forced_into_hiding_time_factor",
        ModifKinds::Country.union(ModifKinds::UnitLeader),
    ),
    ("enemy_operative_harmed_time_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("enemy_operative_intel_extraction_rate", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("enemy_operative_recruitment_chance", ModifKinds::IntelligenceAgency),
    ("enemy_spy_negative_status_factor", ModifKinds::Country.union(ModifKinds::State)),
    ("enemy_truck_attrition_factor", ModifKinds::State),
    ("equipment_capture", ModifKinds::Country),
    ("equipment_capture_factor", ModifKinds::Country),
    ("equipment_capture_factor_for_controller", ModifKinds::State),
    ("equipment_capture_for_controller", ModifKinds::State),
    ("equipment_conversion_speed", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("equipment_upgrade_xp_cost", ModifKinds::Country),
    ("exile_manpower_factor", ModifKinds::GovernmentInExile),
    ("exiled_divisions_attack_factor", ModifKinds::UnitLeader),
    ("exiled_divisions_defense_factor", ModifKinds::UnitLeader),
    ("exiled_government_weekly_manpower", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("experience_gain_air", ModifKinds::Air),
    ("experience_gain_air_factor", ModifKinds::Air),
    ("experience_gain_army", ModifKinds::Army.union(ModifKinds::MilitaryAdvancements)),
    ("experience_gain_army_factor", ModifKinds::Army.union(ModifKinds::MilitaryAdvancements)),
    ("experience_gain_army_unit", ModifKinds::Army.union(ModifKinds::MilitaryAdvancements)),
    ("experience_gain_army_unit_factor", ModifKinds::Army.union(ModifKinds::MilitaryAdvancements)),
    ("experience_gain_factor", ModifKinds::MilitaryAdvancements.union(ModifKinds::UnitLeader)),
    (
        "experience_gain_navy",
        ModifKinds::MilitaryAdvancements.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "experience_gain_navy_factor",
        ModifKinds::MilitaryAdvancements.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "experience_gain_navy_unit",
        ModifKinds::MilitaryAdvancements.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "experience_gain_navy_unit_factor",
        ModifKinds::MilitaryAdvancements.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("experience_loss_factor", ModifKinds::Country),
    ("extra_marine_supply_grace", ModifKinds::Army),
    ("extra_paratrooper_supply_grace", ModifKinds::Army),
    ("extra_trade_to_overlord_factor", ModifKinds::Autonomy),
    ("extra_trade_to_target_factor", ModifKinds::Country),
    ("faction_trade_opinion_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("female_divisional_commander_chance", ModifKinds::Country),
    ("female_random_admiral_chance", ModifKinds::Country.union(ModifKinds::Naval)),
    ("female_random_army_leader_chance", ModifKinds::Army.union(ModifKinds::Country)),
    ("female_random_country_leader_chance", ModifKinds::Country),
    ("female_random_operative_chance", ModifKinds::Country.union(ModifKinds::IntelligenceAgency)),
    ("female_random_scientist_chance", ModifKinds::Country.union(ModifKinds::Scientist)),
    ("field_officer_promotion_penalty", ModifKinds::Country),
    (
        "fighter_sortie_efficiency",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("floating_harbor_duration", ModifKinds::Country),
    ("floating_harbor_range", ModifKinds::Country),
    ("floating_harbor_supply", ModifKinds::Country),
    ("forced_surrender_limit", ModifKinds::Country),
    ("foreign_subversive_activites", ModifKinds::Country),
    ("fortification_collateral_chance", ModifKinds::UnitLeader),
    ("fortification_damage", ModifKinds::UnitLeader),
    ("fuel_cost", ModifKinds::Country),
    ("fuel_gain", ModifKinds::Country),
    ("fuel_gain_factor", ModifKinds::Country),
    ("fuel_gain_factor_from_states", ModifKinds::Country),
    ("fuel_gain_from_states", ModifKinds::Country),
    ("generate_wargoal_tension", ModifKinds::Country),
    ("generate_wargoal_tension_against", ModifKinds::Country),
    ("global_building_slots", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("global_building_slots_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("grant_medal_cost_factor", ModifKinds::Country),
    ("ground_attack", ModifKinds::Aggressive.union(ModifKinds::Air)),
    ("ground_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Air)),
    ("guarantee_cost", ModifKinds::Country.union(ModifKinds::Politics)),
    ("guarantee_tension", ModifKinds::Country.union(ModifKinds::Defensive)),
    ("heat_attrition", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("heat_attrition_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("improve_relations_maintain_cost_factor", ModifKinds::Country),
    ("industrial_capacity_dockyard", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("industrial_capacity_factory", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("industrial_factory_donations", ModifKinds::GovernmentInExile),
    ("industry_air_damage_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("industry_free_repair_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("industry_repair_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("initiative_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("intel_from_combat_factor", ModifKinds::Country),
    ("intel_from_operatives_factor", ModifKinds::IntelligenceAgency),
    (
        "intel_network_gain",
        ModifKinds::Country
            .union(ModifKinds::IntelligenceAgency)
            .union(ModifKinds::State)
            .union(ModifKinds::UnitLeader),
    ),
    (
        "intel_network_gain_factor",
        ModifKinds::Country
            .union(ModifKinds::IntelligenceAgency)
            .union(ModifKinds::State)
            .union(ModifKinds::UnitLeader),
    ),
    ("intelligence_agency_defense", ModifKinds::IntelligenceAgency),
    ("invasion_preparation", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("join_faction_tension", ModifKinds::Country),
    ("justify_war_goal_time", ModifKinds::Aggressive.union(ModifKinds::Country)),
    ("justify_war_goal_when_in_major_war_time", ModifKinds::Aggressive.union(ModifKinds::Country)),
    ("land_bunker_effectiveness_factor", ModifKinds::Country.union(ModifKinds::State)),
    ("land_doctrine_cost_factor", ModifKinds::Country),
    ("land_equipment_upgrade_xp_cost", ModifKinds::Country),
    ("land_night_attack", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("land_reinforce_rate", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("legitimacy_daily", ModifKinds::GovernmentInExile),
    ("legitimacy_gain_factor", ModifKinds::GovernmentInExile),
    ("lend_lease_tension", ModifKinds::Country),
    ("lend_lease_tension_with_overlord", ModifKinds::Country),
    ("license_air_purchase_cost", ModifKinds::Country),
    ("license_anti_tank_eq_cost_factor", ModifKinds::Country),
    ("license_anti_tank_eq_production_speed_factor", ModifKinds::Country),
    ("license_anti_tank_eq_tech_difference_speed_factor", ModifKinds::Country),
    ("license_armor_purchase_cost", ModifKinds::Country),
    ("license_artillery_eq_cost_factor", ModifKinds::Country),
    ("license_artillery_eq_production_speed_factor", ModifKinds::Country),
    ("license_artillery_eq_tech_difference_speed_factor", ModifKinds::Country),
    ("license_infantry_eq_cost_factor", ModifKinds::Country),
    ("license_infantry_eq_production_speed_factor", ModifKinds::Country),
    ("license_infantry_eq_tech_difference_speed_factor", ModifKinds::Country),
    ("license_infantry_purchase_cost", ModifKinds::Country),
    ("license_light_tank_eq_cost_factor", ModifKinds::Country),
    ("license_light_tank_eq_production_speed_factor", ModifKinds::Country),
    ("license_light_tank_eq_tech_difference_speed_factor", ModifKinds::Country),
    ("license_naval_purchase_cost", ModifKinds::Country),
    ("license_production_speed", ModifKinds::Country),
    ("license_purchase_cost", ModifKinds::Country),
    ("license_subject_master_purchase_cost", ModifKinds::Autonomy),
    ("license_tech_difference_speed", ModifKinds::Country),
    (
        "line_change_production_efficiency_factor",
        ModifKinds::Country.union(ModifKinds::WarProduction),
    ),
    ("local_building_slots", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_building_slots_factor", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_factories", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_factory_sabotage", ModifKinds::State),
    ("local_intel_to_enemies", ModifKinds::Defensive.union(ModifKinds::State)),
    ("local_manpower", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_non_core_manpower", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_non_core_supply_impact_factor", ModifKinds::State),
    ("local_org_regain", ModifKinds::Army),
    ("local_resources", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_resources_factor", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_supplies", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_supplies_for_controller", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("local_supply_impact_factor", ModifKinds::State),
    ("marines_special_forces_contribution_factor", ModifKinds::Army),
    ("master_build_autonomy_factor", ModifKinds::Autonomy),
    ("master_ideology_drift", ModifKinds::Politics),
    ("max_army_group_size", ModifKinds::UnitLeader),
    ("max_command_power", ModifKinds::Army.union(ModifKinds::Country)),
    ("max_command_power_mult", ModifKinds::Army.union(ModifKinds::Country)),
    ("max_commander_army_size", ModifKinds::Army),
    ("max_dig_in", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("max_dig_in_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("max_fuel", ModifKinds::Country),
    ("max_fuel_building", ModifKinds::Country),
    ("max_fuel_factor", ModifKinds::Country),
    ("max_planning", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("max_planning_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("max_surrender_limit_offset", ModifKinds::Country),
    ("max_training", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("mechanized_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("mechanized_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("mic_to_overlord_factor", ModifKinds::Autonomy),
    ("mic_to_target_factor", ModifKinds::Country),
    ("military_factory_donations", ModifKinds::GovernmentInExile),
    ("military_industrial_organization_design_team_assign_cost", ModifKinds::WarProduction),
    ("military_industrial_organization_design_team_change_cost", ModifKinds::WarProduction),
    ("military_industrial_organization_funds_gain", ModifKinds::WarProduction),
    (
        "military_industrial_organization_industrial_manufacturer_assign_cost",
        ModifKinds::WarProduction,
    ),
    ("military_industrial_organization_policy_cooldown", ModifKinds::WarProduction),
    ("military_industrial_organization_policy_cost", ModifKinds::WarProduction),
    ("military_industrial_organization_research_bonus", ModifKinds::WarProduction),
    ("military_industrial_organization_size_up_requirement", ModifKinds::WarProduction),
    ("military_industrial_organization_task_capacity", ModifKinds::WarProduction),
    ("military_leader_cost_factor", ModifKinds::UnitLeader),
    ("min_export", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("mines_planting_by_air_factor", ModifKinds::Air),
    ("mines_planting_by_fleets_factor", ModifKinds::Naval),
    ("mines_sweeping_by_air_factor", ModifKinds::Air),
    ("mines_sweeping_by_fleets_factor", ModifKinds::Naval),
    ("minimum_training_level", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("mobilization_speed", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("modifier_army_sub_unit_armored_car_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_armored_car_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_armored_car_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_armored_car_recon_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_armored_car_recon_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_armored_car_recon_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_armored_car_recon_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_armored_car_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_blackshirt_assault_battalion_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_blackshirt_assault_battalion_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_blackshirt_assault_battalion_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_blackshirt_assault_battalion_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_camelry_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_camelry_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_camelry_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_category_special_forces_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_cavalry_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_cavalry_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_cavalry_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_infantry_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_infantry_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_infantry_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_irregular_infantry_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_irregular_infantry_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_irregular_infantry_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_irregular_infantry_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_light_tank_recon_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_light_tank_recon_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_light_tank_recon_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_light_tank_recon_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_long_range_patrol_support_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_long_range_patrol_support_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_marines_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_marines_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_marines_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_marines_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_military_police_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_military_police_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_military_police_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_military_police_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_militia_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_militia_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_militia_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_militia_org_recovery_cap_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_militia_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_mountaineers_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_mountaineers_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_mountaineers_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_mountaineers_speed_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_paratrooper_attack_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_paratrooper_defence_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_paratrooper_max_org_factor", ModifKinds::Army),
    ("modifier_army_sub_unit_paratrooper_speed_factor", ModifKinds::Army),
    ("monthly_population", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("motorized_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("motorized_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("mountaineers_special_forces_contribution_factor", ModifKinds::Army),
    ("naval_accidents_chance", ModifKinds::Naval),
    ("naval_attrition", ModifKinds::Naval),
    ("naval_commando_raid_distance", ModifKinds::Naval),
    ("naval_coordination", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_critical_effect_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    (
        "naval_critical_score_chance_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_damage_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("naval_defense_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_detection", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_doctrine_cost_factor", ModifKinds::Country),
    (
        "naval_enemy_fleet_size_ratio_penalty_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_enemy_positioning_in_initial_attack",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("naval_enemy_retreat_chance", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_equipment_upgrade_xp_cost", ModifKinds::Country),
    (
        "naval_has_potf_in_combat_attack",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_has_potf_in_combat_defense",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_heavy_gun_hit_chance_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_hit_chance",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("naval_invasion_capacity", ModifKinds::Naval),
    ("naval_invasion_penalty", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_invasion_planning_bonus_speed", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("naval_invasion_prep_speed", ModifKinds::UnitLeader),
    (
        "naval_light_gun_hit_chance_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("naval_mine_hit_chance", ModifKinds::Naval),
    ("naval_mines_damage_factor", ModifKinds::Naval),
    ("naval_mines_effect_reduction", ModifKinds::Naval),
    ("naval_morale", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_morale_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_night_attack", ModifKinds::Aggressive.union(ModifKinds::Naval)),
    (
        "naval_retreat_chance",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_retreat_chance_after_initial_combat",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_retreat_speed",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_retreat_speed_after_initial_combat",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("naval_speed_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("naval_strike", ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader)),
    ("naval_strike_agility_factor", ModifKinds::Air),
    ("naval_strike_attack_factor", ModifKinds::Air),
    ("naval_strike_targetting_factor", ModifKinds::Air),
    (
        "naval_torpedo_cooldown_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_torpedo_damage_reduction_factor",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_torpedo_enemy_critical_chance_factor",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "naval_torpedo_hit_chance_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("naval_torpedo_reveal_chance_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    (
        "naval_torpedo_screen_penetration_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("navy_advisor_cost_factor", ModifKinds::Naval),
    ("navy_anti_air_attack", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("navy_anti_air_attack_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    (
        "navy_capital_ship_attack_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_capital_ship_defence_factor",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_carrier_air_agility_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_carrier_air_attack_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_carrier_air_targetting_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("navy_casualty_on_hit", ModifKinds::Naval),
    ("navy_casualty_on_sink", ModifKinds::Naval),
    ("navy_fuel_consumption_factor", ModifKinds::Naval),
    ("navy_intel_decryption_bonus", ModifKinds::IntelligenceAgency),
    ("navy_intel_factor", ModifKinds::IntelligenceAgency),
    ("navy_intel_to_others", ModifKinds::Country),
    ("navy_leader_cost_factor", ModifKinds::UnitLeader),
    ("navy_leader_start_attack_level", ModifKinds::UnitLeader),
    ("navy_leader_start_coordination_level", ModifKinds::UnitLeader),
    ("navy_leader_start_defense_level", ModifKinds::UnitLeader),
    ("navy_leader_start_level", ModifKinds::UnitLeader),
    ("navy_leader_start_maneuvering_level", ModifKinds::UnitLeader),
    ("navy_max_range", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("navy_max_range_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("navy_org", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("navy_org_factor", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    (
        "navy_screen_attack_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_screen_defence_factor",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_submarine_attack_factor",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_submarine_defence_factor",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "navy_submarine_detection_factor",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("navy_visibility", ModifKinds::Naval),
    ("navy_weather_penalty", ModifKinds::Air),
    ("new_operative_slot_bonus", ModifKinds::IntelligenceAgency),
    ("night_spotting_chance", ModifKinds::Naval),
    ("no_compliance_gain", ModifKinds::Country.union(ModifKinds::State)),
    ("no_supply_grace", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("non_core_manpower", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("nuclear_production", ModifKinds::Country),
    ("nuclear_production_factor", ModifKinds::Country),
    ("occupied_operative_recruitment_chance", ModifKinds::IntelligenceAgency),
    ("offence", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("offensive_war_stability_factor", ModifKinds::Country),
    ("operation_cost", ModifKinds::IntelligenceAgency),
    ("operation_infiltrate_outcome", ModifKinds::IntelligenceAgency),
    ("operation_outcome", ModifKinds::IntelligenceAgency),
    ("operative_death_on_capture_chance", ModifKinds::IntelligenceAgency),
    ("operative_slot", ModifKinds::IntelligenceAgency),
    ("opinion_gain_monthly", ModifKinds::Country),
    ("opinion_gain_monthly_factor", ModifKinds::Country),
    ("opinion_gain_monthly_same_ideology", ModifKinds::Country),
    ("opinion_gain_monthly_same_ideology_factor", ModifKinds::Country),
    ("org_loss_at_low_org_factor", ModifKinds::Army),
    ("org_loss_when_moving", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("out_of_supply_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("overlord_trade_cost_factor", ModifKinds::Autonomy),
    ("own_exiled_divisions_attack_factor", ModifKinds::UnitLeader),
    ("own_exiled_divisions_defense_factor", ModifKinds::UnitLeader),
    ("own_operative_capture_chance_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("own_operative_detection_chance", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("own_operative_detection_chance_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    (
        "own_operative_forced_into_hiding_time_factor",
        ModifKinds::Country.union(ModifKinds::UnitLeader),
    ),
    ("own_operative_harmed_time_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("own_operative_intel_extraction_rate", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("paradrop_organization_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("paratrooper_aa_defense", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("paratrooper_weight_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("paratroopers_special_forces_contribution_factor", ModifKinds::Army),
    ("party_popularity_stability_factor", ModifKinds::Country),
    ("peace_score_ratio_transferred_to_overlord", ModifKinds::Peace),
    ("peace_score_ratio_transferred_to_players", ModifKinds::Peace),
    ("planning_speed", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("pocket_penalty", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("political_power_cost", ModifKinds::Country),
    ("political_power_factor", ModifKinds::Country),
    ("political_power_gain", ModifKinds::Country),
    ("port_strike", ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader)),
    ("positioning", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("power_balance_daily", ModifKinds::Country),
    ("power_balance_weekly", ModifKinds::Country),
    (
        "production_factory_efficiency_gain_factor",
        ModifKinds::Country.union(ModifKinds::WarProduction),
    ),
    (
        "production_factory_max_efficiency_factor",
        ModifKinds::Country.union(ModifKinds::WarProduction),
    ),
    (
        "production_factory_start_efficiency_factor",
        ModifKinds::Country.union(ModifKinds::WarProduction),
    ),
    (
        "production_lack_of_resource_penalty_factor",
        ModifKinds::Country.union(ModifKinds::WarProduction),
    ),
    ("production_oil_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("production_speed_buildings_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("production_speed_facility_factor", ModifKinds::Country),
    ("promote_cost_factor", ModifKinds::UnitLeader),
    ("propaganda_mission_factor", ModifKinds::IntelligenceAgency),
    ("puppet_cost_factor", ModifKinds::Defensive.union(ModifKinds::Peace)),
    ("railway_gun_bombardment_factor", ModifKinds::Country),
    ("reassignment_duration_factor", ModifKinds::UnitLeader),
    ("recon_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("recon_factor_while_entrenched", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("recruitable_population", ModifKinds::State),
    ("recruitable_population_factor", ModifKinds::State),
    ("refit_ic_cost", ModifKinds::Country),
    ("refit_speed", ModifKinds::Country),
    ("repair_speed_factor", ModifKinds::Country.union(ModifKinds::Naval)),
    ("request_lease_tension", ModifKinds::Country),
    ("required_garrison_factor", ModifKinds::Country.union(ModifKinds::State)),
    ("research_sharing_per_country_bonus", ModifKinds::Country),
    ("research_sharing_per_country_bonus_factor", ModifKinds::Country),
    ("research_speed_factor", ModifKinds::Country.union(ModifKinds::MilitaryAdvancements)),
    ("resistance_activity", ModifKinds::Country.union(ModifKinds::State)),
    ("resistance_damage_to_garrison", ModifKinds::Country.union(ModifKinds::State)),
    (
        "resistance_damage_to_garrison_on_our_occupied_states",
        ModifKinds::Country.union(ModifKinds::State),
    ),
    ("resistance_decay", ModifKinds::Country.union(ModifKinds::State)),
    ("resistance_decay_on_our_occupied_states", ModifKinds::Country.union(ModifKinds::State)),
    ("resistance_garrison_penetration_chance", ModifKinds::Country.union(ModifKinds::State)),
    ("resistance_growth", ModifKinds::Country.union(ModifKinds::State)),
    ("resistance_growth_on_our_occupied_states", ModifKinds::Country.union(ModifKinds::State)),
    ("resistance_target", ModifKinds::Country.union(ModifKinds::State)),
    ("resistance_target_on_our_occupied_states", ModifKinds::Country.union(ModifKinds::State)),
    ("resource_trade_cost_bonus_per_factory", ModifKinds::Country),
    ("river_crossing_factor", ModifKinds::UnitLeader),
    ("river_crossing_factor_against", ModifKinds::Army),
    ("rocket_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("root_out_resistance_effectiveness_factor", ModifKinds::IntelligenceAgency),
    ("scientist_breakthrough_bonus_factor", ModifKinds::Character),
    ("scientist_research_bonus_factor", ModifKinds::Character),
    ("scientist_xp_gain_factor", ModifKinds::Character),
    ("screening_efficiency", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("screening_without_screens", ModifKinds::Country.union(ModifKinds::Naval)),
    ("send_volunteer_divisions_required", ModifKinds::Country),
    ("send_volunteer_factor", ModifKinds::Country),
    ("send_volunteer_size", ModifKinds::Country),
    ("send_volunteers_tension", ModifKinds::Country.union(ModifKinds::Defensive)),
    ("ships_at_battle_start", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("shore_bombardment_bonus", ModifKinds::Army),
    ("sickness_chance", ModifKinds::UnitLeader),
    ("skill_bonus_factor", ModifKinds::UnitLeader),
    (
        "sortie_efficiency",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("special_forces_attack_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("special_forces_cap", ModifKinds::Army),
    ("special_forces_cap_flat", ModifKinds::Army),
    ("special_forces_defence_factor", ModifKinds::Army.union(ModifKinds::Defensive)),
    ("special_forces_min", ModifKinds::Army),
    ("special_forces_no_supply_grace", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("special_forces_out_of_supply_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("special_forces_training_time_factor", ModifKinds::Army.union(ModifKinds::WarProduction)),
    ("special_project_facility_supply_consumption_factor", ModifKinds::Country),
    ("special_project_speed_factor", ModifKinds::Country),
    ("spotting_chance", ModifKinds::Naval.union(ModifKinds::UnitLeader)),
    ("stability_factor", ModifKinds::Country),
    ("stability_weekly", ModifKinds::Country),
    ("stability_weekly_factor", ModifKinds::Country),
    ("starting_compliance", ModifKinds::Country.union(ModifKinds::State)),
    ("state_production_speed_buildings_factor", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("state_production_speed_facility_factor", ModifKinds::State),
    ("state_resources_factor", ModifKinds::State.union(ModifKinds::WarProduction)),
    ("static_anti_air_damage_factor", ModifKinds::Defensive),
    ("static_anti_air_hit_chance_factor", ModifKinds::Defensive),
    ("strategic_bomb_visibility", ModifKinds::Air),
    (
        "strike_force_movement_org_loss",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    (
        "sub_retreat_speed",
        ModifKinds::Defensive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("subjects_autonomy_gain", ModifKinds::Autonomy),
    (
        "submarine_attack",
        ModifKinds::Aggressive.union(ModifKinds::Naval).union(ModifKinds::UnitLeader),
    ),
    ("subversive_activites_upkeep", ModifKinds::Country),
    ("supply_combat_penalties_on_core_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    (
        "supply_consumption_factor",
        ModifKinds::Aggressive.union(ModifKinds::Army).union(ModifKinds::Defensive),
    ),
    ("supply_factor", ModifKinds::Country.union(ModifKinds::State)),
    ("supply_node_range", ModifKinds::Country),
    ("surrender_limit", ModifKinds::Country),
    ("target_sabotage_factor", ModifKinds::IntelligenceAgency),
    ("targeted_legitimacy_daily", ModifKinds::GovernmentInExile),
    ("tech_air_damage_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("terrain_penalty_reduction", ModifKinds::Army.union(ModifKinds::Defensive)),
    (
        "terrain_trait_xp_gain_factor",
        ModifKinds::Army.union(ModifKinds::Country).union(ModifKinds::Naval),
    ),
    ("thermonuclear_production", ModifKinds::Country),
    ("thermonuclear_production_factor", ModifKinds::Country),
    ("trade_cost_for_target_factor", ModifKinds::Country),
    ("trade_opinion_factor", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("training_time_army", ModifKinds::Army.union(ModifKinds::WarProduction)),
    ("training_time_army_factor", ModifKinds::Army.union(ModifKinds::WarProduction)),
    ("training_time_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("transport_capacity", ModifKinds::Naval),
    ("truck_attrition", ModifKinds::Country.union(ModifKinds::State)),
    ("truck_attrition_factor", ModifKinds::Country.union(ModifKinds::State)),
    ("underway_replenishment_convoy_cost", ModifKinds::Country),
    ("underway_replenishment_range", ModifKinds::Country),
    ("unit_leader_as_advisor_cp_cost_factor", ModifKinds::Country.union(ModifKinds::UnitLeader)),
    ("unit_upkeep_attrition_factor", ModifKinds::Army),
    ("war_stability_factor", ModifKinds::Country),
    ("war_support_factor", ModifKinds::Country),
    ("war_support_weekly", ModifKinds::Country),
    ("war_support_weekly_factor", ModifKinds::Country),
    ("weekly_bombing_war_support", ModifKinds::Country),
    ("weekly_casualties_war_support", ModifKinds::Country),
    ("weekly_convoys_war_support", ModifKinds::Country),
    ("weekly_manpower", ModifKinds::Country.union(ModifKinds::WarProduction)),
    ("winter_attrition", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("winter_attrition_factor", ModifKinds::Aggressive.union(ModifKinds::Army)),
    ("wounded_chance_factor", ModifKinds::UnitLeader),
];

static MODIF_REMOVED_MAP: LazyLock<TigerHashMap<Lowercase<'static>, &'static str>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (s, info) in MODIF_REMOVED_TABLE.iter().copied() {
            hash.insert(Lowercase::new_unchecked(s), info);
        }
        hash
    });

const MODIF_REMOVED_TABLE: &[(&str, &str)] = &[];
