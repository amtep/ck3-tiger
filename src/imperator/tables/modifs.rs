#![allow(non_upper_case_globals)]

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::everything::Everything;
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

    // Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    // local_$PopType$_output
    // local_$PopType$_happyness
    // local_$PopType$_desired_pop_ratio
    if let Some(part) = name_lc.strip_prefix_unchecked("local_") {
        for &sfx in &["_output", "_desired_pop_ratio", "_happyness"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::PopType, &part, name, data, warn);
                return Some(ModifKinds::Province | ModifKinds::State);
            }
        }
    }

    // global_$PopType$_output
    // global_$PopType$_happyness
    // global_$PopType$_desired_pop_ratio
    // global_$PopType$_city_desired_pop_ratio
    if let Some(part) = name_lc.strip_prefix_unchecked("global_") {
        for &sfx in &["_output", "_desired_pop_ratio", "_city_desired_pop_ratio", "_happyness"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::PopType, &part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // $Price$_cost_modifier
    if let Some(part) = name_lc.strip_suffix_unchecked("_cost_modifier") {
        maybe_warn(Item::Price, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // $Party$_party_influence
    if let Some(part) = name_lc.strip_suffix_unchecked("_party_influence") {
        maybe_warn(Item::PartyType, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // monthly_$Party$_party_conviction
    if let Some(part) = name_lc.strip_prefix_unchecked("monthly_") {
        if let Some(part) = part.strip_suffix_unchecked("_party_conviction") {
            maybe_warn(Item::PartyType, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // $Unit$_discipline
    // $Unit$_morale
    // $Unit$_offensive
    // $Unit$_defensive
    // $Unit$_maintenance_cost
    // $Unit$_movement_speed
    for &sfx in &[
        "_discipline",
        "_morale",
        "_offensive",
        "_defensive",
        "_maintenance_cost",
        "_movement_speed",
    ] {
        if let Some(part) = name_lc.strip_suffix_unchecked(sfx) {
            maybe_warn(Item::Unit, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // $Terrain$_combat_bonus
    // $Unit$_$Terrain$_combat_bonus
    if let Some(part) = name_lc.strip_suffix_unchecked("_combat_bonus") {
        // This is tricky because both Unit and Terrain can have `_` in them.
        // Try each possible separation point in turn.
        for (i, _) in part.rmatch_indices_unchecked('_') {
            if data.item_exists_lc(Item::Terrain, &part.slice(i + 1..)) {
                // If the Terrain exists, then the prefix must be the Unit.
                maybe_warn(Item::Unit, &part.slice(..i), name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
        // Check if it's the kind without $Unit$
        maybe_warn(Item::Terrain, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // $Unit$_cost
    if let Some(part) = name_lc.strip_suffix_unchecked("_cost") {
        maybe_warn(Item::Unit, &part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // $TechnologyTable$_investment
    if let Some(part) = name_lc.strip_suffix_unchecked("_investment") {
        maybe_warn(Item::TechnologyTable, &part, name, data, warn);
        return Some(ModifKinds::Country);
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

static MODIF_MAP: Lazy<FnvHashMap<Lowercase<'static>, ModifKinds>> = Lazy::new(|| {
    let mut hash = FnvHashMap::default();
    for (s, kind) in MODIF_TABLE.iter().copied() {
        hash.insert(Lowercase::new_unchecked(s), kind);
    }
    hash
});

/// LAST UPDATED VERSION 2.0.4
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, ModifKinds)] = &[
    ("build_cost", ModifKinds::Country),
    ("build_time", ModifKinds::Country),
    ("minimum_unrest", ModifKinds::Province),
    ("local_unrest", ModifKinds::Province),
    ("global_unrest", ModifKinds::Country),
    ("tax_income", ModifKinds::Province),
    ("global_tax_income", ModifKinds::Province),
    ("local_tax_modifier", ModifKinds::Province),
    ("global_tax_modifier", ModifKinds::Country),
    ("local_population_growth", ModifKinds::Province),
    ("global_population_growth", ModifKinds::Country),
    ("local_population_capacity", ModifKinds::Province),
    ("local_population_capacity_modifier", ModifKinds::Province),
    ("global_population_capacity_modifier", ModifKinds::Country),
    ("total_population_capacity_modifier", ModifKinds::Province),
    ("local_building_slot", ModifKinds::Province),
    ("global_building_slot", ModifKinds::Country),
    ("global_monthly_state_loyalty", ModifKinds::Country),
    ("local_monthly_state_loyalty", ModifKinds::State),
    ("city_monthly_state_loyalty", ModifKinds::Province),
    ("happiness_for_wrong_culture_modifier", ModifKinds::Province),
    ("happiness_for_wrong_culture_group_modifier", ModifKinds::Province),
    ("happiness_for_same_culture_modifier", ModifKinds::Province),
    ("local_happiness_for_same_culture_modifier", ModifKinds::Province),
    ("happiness_for_same_religion_modifier", ModifKinds::Province),
    ("local_happiness_for_same_religion_modifier", ModifKinds::Province),
    ("global_population_happiness", ModifKinds::Province),
    ("local_population_happiness", ModifKinds::Province),
    ("land_morale", ModifKinds::Country),
    ("naval_morale", ModifKinds::Country),
    ("land_morale_modifier", ModifKinds::Country),
    ("naval_morale_modifier", ModifKinds::Country),
    ("non_retinue_morale_modifier", ModifKinds::Country),
    ("local_manpower", ModifKinds::Province),
    ("global_manpower", ModifKinds::Country),
    ("local_manpower_modifier", ModifKinds::Province),
    ("global_manpower_modifier", ModifKinds::Country),
    ("manpower_recovery_speed", ModifKinds::Country),
    ("attrition", ModifKinds::Province),
    ("land_unit_attrition", ModifKinds::Country),
    ("naval_unit_attrition", ModifKinds::Country),
    ("army_weight_modifier", ModifKinds::Country),
    ("navy_weight_modifier", ModifKinds::Country),
    ("max_attrition", ModifKinds::Province),
    ("supply_limit", ModifKinds::Province),
    ("supply_limit_modifier", ModifKinds::Province),
    ("global_supply_limit_modifier", ModifKinds::Country),
    ("war_exhaustion", ModifKinds::Country),
    ("max_war_exhaustion", ModifKinds::Country),
    ("fort_level", ModifKinds::Province),
    ("blockade_efficiency", ModifKinds::Country),
    ("monthly_centralization", ModifKinds::Country),
    ("monthly_legitimacy", ModifKinds::Country),
    ("agressive_expansion_impact", ModifKinds::Country),
    ("agressive_expansion_monthly_change", ModifKinds::Country),
    ("agressive_expansion_monthly_decay", ModifKinds::Country),
    ("local_ship_recruit_speed", ModifKinds::Province),
    ("local_cohort_recruit_speed", ModifKinds::Province),
    ("global_ship_recruit_speed", ModifKinds::Country),
    ("global_cohort_recruit_speed", ModifKinds::Country),
    ("garrison_size", ModifKinds::Country),
    ("garrison_growth", ModifKinds::Province),
    ("technology_investment", ModifKinds::Country),
    ("movement_cost", ModifKinds::Province),
    ("army_movement_speed", ModifKinds::Province),
    ("navy_movement_speed", ModifKinds::Province),
    ("movement_speed_if_no_road", ModifKinds::Province),
    ("local_state_trade_routes", ModifKinds::State),
    ("global_capital_trade_routes", ModifKinds::Country),
    ("global_state_trade_routes", ModifKinds::Country),
    ("research_points", ModifKinds::Country),
    ("research_points_modifier", ModifKinds::Country),
    ("local_research_points_modifier", ModifKinds::Province),
    ("omen_power", ModifKinds::Country),
    ("omen_duration", ModifKinds::Country),
    ("discipline", ModifKinds::Country),
    ("local_defensive", ModifKinds::Province),
    ("global_defensive", ModifKinds::Country),
    ("commerce_value", ModifKinds::Country),
    ("local_commerce_value_modifier", ModifKinds::Country),
    ("global_commerce_modifier", ModifKinds::Country),
    ("global_export_commerce_modifier", ModifKinds::Country),
    ("global_import_commerce_modifier", ModifKinds::Country),
    ("state_commerce_modifier", ModifKinds::Country),
    ("tribute_income_modifier", ModifKinds::Country),
    ("ruler_popularity_gain", ModifKinds::Country),
    ("max_loyalty", ModifKinds::Country),
    ("integrate_speed", ModifKinds::Country),
    ("fabricate_claim_speed", ModifKinds::Country),
    ("monthly_wage_for_character", ModifKinds::Character),
    ("monthly_wage_modifier", ModifKinds::Country),
    ("monthly_wage_on_character_modifier", ModifKinds::Character),
    ("monthly_governor_wage", ModifKinds::Country),
    ("monthly_local_governor_wage", ModifKinds::Country),
    ("monthly_character_popularity", ModifKinds::Country),
    ("monthly_character_popularity_decay", ModifKinds::Country),
    ("monthly_character_prominence", ModifKinds::Country),
    ("monthly_character_fam_prestige", ModifKinds::Country),
    ("cohort_reinforcement_speed", ModifKinds::Country),
    ("land_morale_recovery", ModifKinds::Country),
    ("naval_morale_recovery", ModifKinds::Country),
    ("siege_ability", ModifKinds::Country),
    ("assault_ability", ModifKinds::Country),
    ("siege_engineers", ModifKinds::Country),
    ("diplomatic_reputation", ModifKinds::Country),
    ("diplomatic_relations", ModifKinds::Country),
    ("max_rivals", ModifKinds::Country),
    ("max_friends", ModifKinds::Country),
    ("current_corruption", ModifKinds::Character),
    ("monthly_corruption", ModifKinds::Country),
    ("subject_opinions", ModifKinds::Country),
    ("subject_loyalty", ModifKinds::Country),
    ("loyalty_to_overlord", ModifKinds::Country),
    ("fort_maintenance_cost", ModifKinds::Country),
    ("army_maintenance_cost", ModifKinds::Country),
    ("navy_maintenance_cost", ModifKinds::Country),
    ("mercenary_land_maintenance_cost", ModifKinds::Country),
    ("mercenary_naval_maintenance_cost", ModifKinds::Country),
    ("country_civilization_value", ModifKinds::Country),
    ("local_country_civilization_value", ModifKinds::Country),
    ("local_monthly_civilization", ModifKinds::Province),
    ("global_monthly_civilization", ModifKinds::Country),
    ("global_start_experience", ModifKinds::Country),
    ("local_start_experience", ModifKinds::Province),
    ("global_cohort_start_experience", ModifKinds::Country),
    ("local_cohort_start_experience", ModifKinds::Province),
    ("global_ship_start_experience", ModifKinds::Country),
    ("local_ship_start_experience", ModifKinds::Province),
    ("experience_decay", ModifKinds::Country),
    ("monthly_experience_gain", ModifKinds::Country),
    ("martial", ModifKinds::Character),
    ("finesse", ModifKinds::Character),
    ("charisma", ModifKinds::Character),
    ("zeal", ModifKinds::Character),
    ("fertility", ModifKinds::Character),
    ("health", ModifKinds::Character),
    ("barbarian_growth", ModifKinds::Country),
    ("barbarian_spawn_chance", ModifKinds::Country),
    ("loyalty_gain_chance", ModifKinds::Country),
    ("loyalty_gain_chance_modifier", ModifKinds::Country),
    ("prominence", ModifKinds::Character),
    ("senate_influence", ModifKinds::Country),
    ("monthly_party_approval", ModifKinds::Country),
    ("monthly_tyranny", ModifKinds::Country),
    ("monthly_political_influence", ModifKinds::Country),
    ("monthly_political_influence_modifier", ModifKinds::Country),
    ("retreat_delay", ModifKinds::Country),
    ("improve_relation_impact", ModifKinds::Country),
    ("hostile_attrition", ModifKinds::Country),
    ("local_hostile_attrition", ModifKinds::Province),
    ("election_term_duration", ModifKinds::Country),
    ("ship_repair_at_sea", ModifKinds::Country),
    ("war_score_cost", ModifKinds::Country),
    ("base_resources", ModifKinds::Province),
    ("local_goods_from_slaves", ModifKinds::Province),
    ("global_goods_from_slaves", ModifKinds::Country),
    ("disallow_job", ModifKinds::Character),
    ("disallow_office", ModifKinds::Character),
    ("disallow_command", ModifKinds::Character),
    ("show_3d_fort", ModifKinds::Character),
    ("control_range_modifier", ModifKinds::Character),
    ("diplomatic_range_modifier", ModifKinds::Character),
    ("monthly_character_wealth", ModifKinds::Character),
    ("primary_heir_attraction", ModifKinds::Character),
    ("support_for_character_as_heir", ModifKinds::Character),
    ("next_ruler_legitimacy", ModifKinds::Character),
    ("num_of_clan_chiefs", ModifKinds::Character),
    ("clan_retinue_size", ModifKinds::Character),
    ("enslavement_efficiency", ModifKinds::Character),
    ("local_output_modifier", ModifKinds::Character),
    ("holdings_possible_for_character", ModifKinds::Character),
    ("available_holdings", ModifKinds::Character),
    ("holding_income_modifier", ModifKinds::Character),
    ("stability_monthly_change", ModifKinds::Country),
    ("stability_monthly_decay", ModifKinds::Country),
    ("civil_war_threshold", ModifKinds::Country),
    ("ship_capture_chance", ModifKinds::Country),
    ("naval_damage_done", ModifKinds::Country),
    ("naval_damage_taken", ModifKinds::Country),
    ("ship_cost", ModifKinds::Country),
    ("cohort_cost", ModifKinds::Country),
    ("pirate_haven", ModifKinds::Province),
    ("pirate_plunder", ModifKinds::Province),
    ("anti_piracy_cb", ModifKinds::Country),
    ("naval_range", ModifKinds::Country),
    ("monthly_military_experience", ModifKinds::Country),
    ("monthly_military_experience_modifier", ModifKinds::Country),
    ("local_pop_promotion_speed", ModifKinds::Province),
    ("global_pop_promotion_speed", ModifKinds::Province),
    ("local_pop_promotion_speed_modifier", ModifKinds::Province),
    ("global_pop_promotion_speed_modifier", ModifKinds::Province),
    ("local_pop_demotion_speed", ModifKinds::Province),
    ("global_pop_demotion_speed", ModifKinds::Province),
    ("local_pop_demotion_speed_modifier", ModifKinds::Province),
    ("global_pop_demotion_speed_modifier", ModifKinds::Province),
    ("local_migration_attraction", ModifKinds::Province),
    ("local_migration_speed", ModifKinds::Province),
    ("global_migration_speed", ModifKinds::Province),
    ("local_migration_speed_modifier", ModifKinds::Province),
    ("global_migration_speed_modifier", ModifKinds::Province),
    ("local_pop_conversion_speed", ModifKinds::Province),
    ("global_pop_conversion_speed", ModifKinds::Province),
    ("local_pop_conversion_speed_modifier", ModifKinds::Province),
    ("global_pop_conversion_speed_modifier", ModifKinds::Province),
    ("local_pop_assimilation_speed", ModifKinds::Province),
    ("global_pop_assimilation_speed", ModifKinds::Province),
    ("local_pop_assimilation_speed_modifier", ModifKinds::Province),
    ("global_pop_assimilation_speed_modifier", ModifKinds::Province),
    ("cultural_integration_speed_modifier", ModifKinds::Country),
    ("culture_happiness_modifier", ModifKinds::Country),
    ("local_monthly_food", ModifKinds::Province),
    ("global_monthly_food_modifier", ModifKinds::Country),
    ("global_food_capacity", ModifKinds::Country),
    ("local_food_capacity", ModifKinds::Province),
    ("local_monthly_food_modifier", ModifKinds::Province),
    ("local_hostile_food_multiplier", ModifKinds::Province),
    ("pop_food_consumption", ModifKinds::Province),
    ("monthly_character_experience", ModifKinds::Character),
    ("monthly_character_experience_decay", ModifKinds::Character),
    ("monthly_conviction_for_head_of_family_party", ModifKinds::Character),
    ("local_base_trade_routes", ModifKinds::Province),
    ("local_base_trade_routes_modifier", ModifKinds::Province),
    ("enable_intervene", ModifKinds::Country),
    ("character_loyalty", ModifKinds::Country),
    ("general_loyalty", ModifKinds::Country),
    ("admiral_loyalty", ModifKinds::Country),
    ("governor_loyalty", ModifKinds::Country),
    ("clan_chief_loyalty", ModifKinds::Country),
    ("levy_size_multiplier", ModifKinds::Country),
    ("great_work_total_workrate_character_modifier", ModifKinds::Character),
    ("great_work_slaves_workrate_character_modifier", ModifKinds::Character),
    ("great_work_tribals_workrate_character_modifier", ModifKinds::Character),
    ("great_work_freemen_workrate_character_modifier", ModifKinds::Character),
    ("great_work_fixed_prestige_character_modifier", ModifKinds::Character),
    ("local_combat_width_modifier", ModifKinds::Province),
    ("watercrossing_enabled_for_river", ModifKinds::Country),
    ("watercrossing_enabled_for_strait", ModifKinds::Country),
    ("watercrossing_enabled_for_shore", ModifKinds::Country),
    ("succession_value", ModifKinds::Character),
    ("fort_limit", ModifKinds::Country),
    ("local_fort_limit", ModifKinds::State),
    ("global_settlement_building_slot", ModifKinds::Country),
    ("max_research_efficiency", ModifKinds::Country),
    ("max_mercenary_stacks", ModifKinds::Country),
];
