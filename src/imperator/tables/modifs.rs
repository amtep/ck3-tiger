#![allow(non_upper_case_globals)]

use crate::everything::Everything;
use crate::item::Item;
use crate::modif::ModifKinds;
use crate::report::{err, ErrorKey};
use crate::token::Token;

/// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
/// Returns None otherwise.
pub fn lookup_modif(name: &Token, data: &Everything, warn: bool) -> Option<ModifKinds> {
    for &(entry_name, mk) in MODIF_TABLE {
        if name.is(entry_name) {
            return Some(ModifKinds::from_bits_truncate(mk));
        }
    }

    // Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    // local_$PopType$_output
    // local_$PopType$_happyness
    // local_$PopType$_desired_pop_ratio
    if let Some(part) = name.as_str().strip_prefix("local_") {
        for sfx in &["_output", "_desired_pop_ratio", "_happyness"] {
            if let Some(part) = part.strip_suffix(sfx) {
                if warn {
                    data.verify_exists_implied(Item::PopType, part, name);
                }
                return Some(ModifKinds::Province | ModifKinds::State);
            }
        }
    }

    // global_$PopType$_output
    // global_$PopType$_happyness
    // global_$PopType$_desired_pop_ratio
    // global_$PopType$_city_desired_pop_ratio
    if let Some(part) = name.as_str().strip_prefix("global_") {
        for sfx in &["_output", "_desired_pop_ratio", "_city_desired_pop_ratio", "_happyness"] {
            if let Some(part) = part.strip_suffix(sfx) {
                if warn {
                    data.verify_exists_implied(Item::PopType, part, name);
                }
                return Some(ModifKinds::Country);
            }
        }
    }

    // $Price$_cost_modifier
    if let Some(part) = name.as_str().strip_suffix("_cost_modifier") {
        if warn {
            data.verify_exists_implied(Item::Price, part, name);
        }
        return Some(ModifKinds::Country);
    }

    // $Party$_party_influence
    if let Some(part) = name.as_str().strip_suffix("_party_influence") {
        if warn {
            data.verify_exists_implied(Item::PartyType, part, name);
        }
        return Some(ModifKinds::Country);
    }

    // monthly_$Party$_party_conviction
    if let Some(part) = name.as_str().strip_prefix("monthly_") {
        if let Some(part) = part.strip_suffix("_party_conviction") {
            if warn {
                data.verify_exists_implied(Item::PartyType, part, name);
            }
            return Some(ModifKinds::Country);
        }
    }

    // $Unit$_discipline
    // $Unit$_morale
    // $Unit$_offensive
    // $Unit$_defensive
    // $Unit$_maintenance_cost
    // $Unit$_movement_speed
    for sfx in &["_discipline", "_morale", "_offensive", "_defensive", "_maintenance_cost", "_movement_speed"] {
        if let Some(part) = name.as_str().strip_suffix(sfx) {
            if warn {
                data.verify_exists_implied(Item::Unit, part, name);
            }
            return Some(ModifKinds::Country);
        }
    }

    // $Terrain$_combat_bonus
    // $Unit$_$Terrain$_combat_bonus
    if let Some(part) = name.as_str().strip_suffix("_combat_bonus") {
        if warn
            && !data.item_exists(Item::Terrain, part)
            && !data.item_exists(Item::Unit, s)
        {
            let msg = format!("could not find any {part}");
            let info = "Could be: $Terrain$_combat_bonus or $Unit$_$Terrain$_combat_bonus";
            warn_info(name, ErrorKey::MissingItem, &msg, info);
        }
        return Some(ModifKinds::Country);
    }

    // $Unit$_cost
    if let Some(part) = name.as_str().strip_suffix("_cost") {
        if warn {
            data.verify_exists_implied(Item::Unit, part, name);
        }
        return Some(ModifKinds::Country);
    }
    
    // $TechnologyTable$_investment
    if let Some(part) = name.as_str().strip_suffix("_investment") {
        if warn {
            data.verify_exists_implied(Item::TechnologyTable, part, name);
        }
        return Some(ModifKinds::Country);
    }

    None
}

// Redeclare the `ModifKinds` enums as bare numbers, so that we can do | on them in const tables.
const NoneModifKind: u16 = 0x0001;
const Character: u16 = 0x0002;
const Country: u16 = 0x0004;
const Province: u16 = 0x0008;
const State: u16 = 0x0010;

/// LAST UPDATED VERSION 2.0.4
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, u16)] = &[
    ("build_cost", Country),
    ("build_time", Country),
    ("minimum_unrest", Province),
    ("local_unrest", Province),
    ("global_unrest", Country),
    ("tax_income", Province),
    ("global_tax_income", Province),
    ("local_tax_modifier", Province),
    ("global_tax_modifier", Country),
    ("local_population_growth", Province),
    ("global_population_growth", Country),
    ("local_population_capacity", Province),
    ("local_population_capacity_modifier", Province),
    ("global_population_capacity_modifier", Country),
    ("total_population_capacity_modifier", Province),
    ("local_building_slot", Province),
    ("global_building_slot", Country),
    ("global_monthly_state_loyalty", Country),
    ("local_monthly_state_loyalty", State),
    ("city_monthly_state_loyalty", Province),
    ("happiness_for_wrong_culture_modifier", Province),
    ("happiness_for_wrong_culture_group_modifier", Province),
    ("happiness_for_same_culture_modifier", Province),
    ("local_happiness_for_same_culture_modifier", Province),
    ("happiness_for_same_religion_modifier", Province),
    ("local_happiness_for_same_religion_modifier", Province),
    ("global_population_happiness", Province),
    ("local_population_happiness", Province),
    ("land_morale", Country),
    ("naval_morale", Country),
    ("land_morale_modifier", Country),
    ("naval_morale_modifier", Country),
    ("non_retinue_morale_modifier", Country),
    ("local_manpower", Province),
    ("global_manpower", Country),
    ("local_manpower_modifier", Province),
    ("global_manpower_modifier", Country),
    ("manpower_recovery_speed", Country),
    ("attrition", Province),
    ("land_unit_attrition", Country),
    ("naval_unit_attrition", Country),
    ("army_weight_modifier", Country),
    ("navy_weight_modifier", Country),
    ("max_attrition", Province),
    ("supply_limit", Province),
    ("supply_limit_modifier", Province),
    ("global_supply_limit_modifier", Country),
    ("war_exhaustion", Country),
    ("max_war_exhaustion", Country),
    ("fort_level", Province),
    ("blockade_efficiency", Country),
    ("monthly_centralization", Country),
    ("monthly_legitimacy", Country),
    ("agressive_expansion_impact", Country),
    ("agressive_expansion_monthly_change", Country),
    ("agressive_expansion_monthly_decay", Country),
    ("local_ship_recruit_speed", Province),
    ("local_cohort_recruit_speed", Province),
    ("global_ship_recruit_speed", Country),
    ("global_cohort_recruit_speed", Country),
    ("garrison_size", Country),
    ("garrison_growth", Province),
    ("technology_investment", Country),
    ("movement_cost", Province),
    ("army_movement_speed", Province),
    ("navy_movement_speed", Province),
    ("movement_speed_if_no_road", Province),
    ("local_state_trade_routes", State),
    ("global_capital_trade_routes", Country),
    ("global_state_trade_routes", Country),
    ("research_points", Country),
    ("research_points_modifier", Country),
    ("local_research_points_modifier", Province),
    ("omen_power", Country),
    ("omen_duration", Country),
    ("discipline", Country),
    ("local_defensive", Province),
    ("global_defensive", Country),
    ("commerce_value", Country),
    ("local_commerce_value_modifier", Country),
    ("global_commerce_modifier", Country),
    ("global_export_commerce_modifier", Country),
    ("global_import_commerce_modifier", Country),
    ("state_commerce_modifier", Country),
    ("tribute_income_modifier", Country),
    ("ruler_popularity_gain", Country),
    ("max_loyalty", Country),
    ("integrate_speed", Country),
    ("fabricate_claim_speed", Country),
    ("monthly_wage_for_character", Character),
    ("monthly_wage_modifier", Country),
    ("monthly_wage_on_character_modifier", Character),
    ("monthly_governor_wage", Country),
    ("monthly_local_governor_wage", Country),
    ("monthly_character_popularity", Country),
    ("monthly_character_popularity_decay", Country),
    ("monthly_character_prominence", Country),
    ("monthly_character_fam_prestige", Country),
    ("cohort_reinforcement_speed", Country),
    ("land_morale_recovery", Country),
    ("naval_morale_recovery", Country),
    ("siege_ability", Country),
    ("assault_ability", Country),
    ("siege_engineers", Country),
    ("diplomatic_reputation", Country),
    ("diplomatic_relations", Country),
    ("max_rivals", Country),
    ("max_friends", Country),
    ("current_corruption", Character),
    ("monthly_corruption", Country),
    ("subject_opinions", Country),
    ("subject_loyalty", Country),
    ("loyalty_to_overlord", Country),
    ("fort_maintenance_cost", Country),
    ("army_maintenance_cost", Country),
    ("navy_maintenance_cost", Country),
    ("mercenary_land_maintenance_cost", Country),
    ("mercenary_naval_maintenance_cost", Country),
    ("country_civilization_value", Country),
    ("local_country_civilization_value", Country),
    ("local_monthly_civilization", Province),
    ("global_monthly_civilization", Country),
    ("global_start_experience", Country),
    ("local_start_experience", Province),
    ("global_cohort_start_experience", Country),
    ("local_cohort_start_experience", Province),
    ("global_ship_start_experience", Country),
    ("local_ship_start_experience", Province),
    ("experience_decay", Country),
    ("monthly_experience_gain", Country),
    ("martial", Character),
    ("finesse", Character),
    ("charisma", Character),
    ("zeal", Character),
    ("fertility", Character),
    ("health", Character),
    ("barbarian_growth", Country),
    ("barbarian_spawn_chance", Country),
    ("loyalty_gain_chance", Country),
    ("loyalty_gain_chance_modifier", Country),
    ("prominence", Character),
    ("senate_influence", Country),
    ("monthly_party_approval", Country),
    ("monthly_tyranny", Country),
    ("monthly_political_influence", Country),
    ("monthly_political_influence_modifier", Country),
    ("retreat_delay", Country),
    ("improve_relation_impact", Country),
    ("hostile_attrition", Country),
    ("local_hostile_attrition", Province),
    ("election_term_duration", Country),
    ("ship_repair_at_sea", Country),
    ("war_score_cost", Country),
    ("base_resources", Province),
    ("local_goods_from_slaves", Province),
    ("global_goods_from_slaves", Country),
    ("disallow_job", Character),
    ("disallow_office", Character),
    ("disallow_command", Character),
    ("show_3d_fort", Character),
    ("control_range_modifier", Character),
    ("diplomatic_range_modifier", Character),
    ("monthly_character_wealth", Character),
    ("primary_heir_attraction", Character),
    ("support_for_character_as_heir", Character),
    ("next_ruler_legitimacy", Character),
    ("num_of_clan_chiefs", Character),
    ("clan_retinue_size", Character),
    ("enslavement_efficiency", Character),
    ("local_output_modifier", Character),
    ("holdings_possible_for_character", Character),
    ("available_holdings", Character),
    ("holding_income_modifier", Character),
    ("stability_monthly_change", Country),
    ("stability_monthly_decay", Country),
    ("civil_war_threshold", Country),
    ("ship_capture_chance", Country),
    ("naval_damage_done", Country),
    ("naval_damage_taken", Country),
    ("ship_cost", Country),
    ("cohort_cost", Country),
    ("pirate_haven", Province),
    ("pirate_plunder", Province),
    ("anti_piracy_cb", Country),
    ("naval_range", Country),
    ("monthly_military_experience", Country),
    ("monthly_military_experience_modifier", Country),
    ("local_pop_promotion_speed", Province),
    ("global_pop_promotion_speed", Province),
    ("local_pop_promotion_speed_modifier", Province),
    ("global_pop_promotion_speed_modifier", Province),
    ("local_pop_demotion_speed", Province),
    ("global_pop_demotion_speed", Province),
    ("local_pop_demotion_speed_modifier", Province),
    ("global_pop_demotion_speed_modifier", Province),
    ("local_migration_attraction", Province),
    ("local_migration_speed", Province),
    ("global_migration_speed", Province),
    ("local_migration_speed_modifier", Province),
    ("global_migration_speed_modifier", Province),
    ("local_pop_conversion_speed", Province),
    ("global_pop_conversion_speed", Province),
    ("local_pop_conversion_speed_modifier", Province),
    ("global_pop_conversion_speed_modifier", Province),
    ("local_pop_assimilation_speed", Province),
    ("global_pop_assimilation_speed", Province),
    ("local_pop_assimilation_speed_modifier", Province),
    ("global_pop_assimilation_speed_modifier", Province),
    ("cultural_integration_speed_modifier", Country),
    ("culture_happiness_modifier", Country),
    ("local_monthly_food", Province),
    ("global_monthly_food_modifier", Country),
    ("global_food_capacity", Country),
    ("local_food_capacity", Province),
    ("local_monthly_food_modifier", Province),
    ("local_hostile_food_multiplier", Province),
    ("pop_food_consumption", Province),
    ("monthly_character_experience", Character),
    ("monthly_character_experience_decay", Character),
    ("monthly_conviction_for_head_of_family_party", Character),
    ("local_base_trade_routes", Province),
    ("local_base_trade_routes_modifier", Province),
    ("enable_intervene", Country),
    ("character_loyalty", Country),
    ("general_loyalty", Country),
    ("admiral_loyalty", Country),
    ("governor_loyalty", Country),
    ("clan_chief_loyalty", Country),
    ("levy_size_multiplier", Country),
    ("great_work_total_workrate_character_modifier", Character),
    ("great_work_slaves_workrate_character_modifier", Character),
    ("great_work_tribals_workrate_character_modifier", Character),
    ("great_work_freemen_workrate_character_modifier", Character),
    ("great_work_fixed_prestige_character_modifier", Character),
    ("local_combat_width_modifier", Province),
    ("watercrossing_enabled_for_river", Country),
    ("watercrossing_enabled_for_strait", Country),
    ("watercrossing_enabled_for_shore", Country),
    ("succession_value", Character),
    ("fort_limit", Country),
    ("local_fort_limit", State),
    ("global_settlement_building_slot", Country),
    ("max_research_efficiency", Country),
    ("max_mercenary_stacks", Country),
];
