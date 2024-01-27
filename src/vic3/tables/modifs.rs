#![allow(non_upper_case_globals)]

use crate::everything::Everything;
use crate::item::Item;
use crate::modif::ModifKinds;
use crate::report::{report, ErrorKey, Severity};
use crate::token::Token;

/// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
/// Returns None otherwise.
pub fn lookup_modif(name: &Token, data: &Everything, warn: Option<Severity>) -> Option<ModifKinds> {
    for &(entry_name, mk) in MODIF_TABLE {
        if name.is(entry_name) {
            return Some(ModifKinds::from_bits_truncate(mk));
        }
    }

    for &(entry_name, info) in MODIF_REMOVED {
        if name.is(entry_name) {
            if let Some(sev) = warn {
                let msg = format!("{name} has been removed");
                report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
            }
            return Some(ModifKinds::all());
        }
    }

    // Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    // building_employment_$PopType$_add
    // building_employment_$PopType$_mult
    if let Some(part) = name.as_str().strip_prefix("building_employment_") {
        for sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::PopType, part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
    }
    // building_group_$BuildingGroup$_$PopType$_fertility_mult
    // building_group_$BuildingGroup$_$PopType$_mortality_mult
    // building_group_$BuildingGroup$_$PopType$_standard_of_living_add
    // building_group_$BuildingGroup$_employee_mult
    // building_group_$BuildingGroup$_fertility_mult
    // building_group_$BuildingGroup$_mortality_mult
    // building_group_$BuildingGroup$_standard_of_living_add
    // building_group_$BuildingGroup$_throughput_mult (obsolete)
    // building_group_$BuildingGroup$_unincorporated_throughput_add
    // building_group_$BuildingGroup$_throughput_add
    // building_group_$BuildingGroup$_tax_mult
    if let Some(part) = name.as_str().strip_prefix("building_group_") {
        for sfx in &["_fertility_mult", "_mortality_mult", "_standard_of_living_add"] {
            if let Some(part) = part.strip_suffix(sfx) {
                // This is tricky because both BuildingGroup and PopType can have `_` in them.
                for (i, _) in part.rmatch_indices('_') {
                    if data.item_exists(Item::PopType, &part[i + 1..]) {
                        maybe_warn(Item::BuildingGroup, &part[..i], name, data, warn);
                        return Some(ModifKinds::Building);
                    }
                }
                // Check if it's the kind without $PopType$
                maybe_warn(Item::BuildingGroup, part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
        for sfx in
            &["_employee_mult", "_tax_mult", "_unincorporated_throughput_add", "_throughput_add"]
        {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::BuildingGroup, part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
        if let Some(part) = part.strip_suffix("_throughput_mult") {
            maybe_warn(Item::BuildingGroup, part, name, data, warn);
            if let Some(sev) = warn {
                let msg = format!("`{name}` was removed in 1.5");
                let info = "it was replaced with `_add`";
                report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
            }
            return Some(ModifKinds::Building);
        }
    }

    // $BuildingType$_throughput_mult (obsolete)
    if let Some(part) = name.as_str().strip_suffix("_throughput_mult") {
        maybe_warn(Item::BuildingType, part, name, data, warn);
        if let Some(sev) = warn {
            let msg = format!("`{name}` was removed in 1.5");
            let info = "it was replaced with `_add`";
            report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
        }
        return Some(ModifKinds::Building);
    }
    // $BuildingType$_throughput_add
    if let Some(part) = name.as_str().strip_suffix("_throughput_add") {
        maybe_warn(Item::BuildingType, part, name, data, warn);
        return Some(ModifKinds::Building);
    }

    // building_$PopType$_fertility_mult
    // building_$PopType$_mortality_mult
    // building_$PopType$_shares_add
    // building_$PopType$_shares_mult
    if let Some(part) = name.as_str().strip_prefix("building_") {
        for sfx in &["_fertility_mult", "_mortality_mult", "_shares_add", "_shares_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::PopType, part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
    }

    // building_input_$Goods$_add (obsolete)
    if let Some(part) = name.as_str().strip_prefix("building_input_") {
        if let Some(part) = part.strip_suffix("_add") {
            maybe_warn(Item::Goods, part, name, data, warn);
            if let Some(sev) = warn {
                let msg = format!("`{name}` was removed in 1.5");
                let info = format!("replaced with `goods_input_{part}_add`");
                report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
            }
            return Some(ModifKinds::Building);
        }
    }
    // TODO: the _mult doesn't exist for all goods
    // goods_input_$Goods$_add
    // goods_input_$Goods$_mult
    if let Some(part) = name.as_str().strip_prefix("goods_input_") {
        for sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::Goods, part, name, data, warn);
                return Some(ModifKinds::Goods);
            }
        }
    }

    // building_output_$Goods$_add (obsolete)
    // building_output_$Goods$_mult (obsolete)
    if let Some(part) = name.as_str().strip_prefix("building_output_") {
        // TODO: some goods don't have the _mult version. Figure out why.
        for sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::Goods, part, name, data, warn);
                if let Some(sev) = warn {
                    let msg = format!("`{name}` was removed in 1.5");
                    let info = format!("it was replaced with `goods_output_{part}{sfx}`");
                    report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
                }
                return Some(ModifKinds::Building);
            }
        }
    }
    // goods_output_$Goods$_add
    // goods_output_$Goods$_mult
    if let Some(part) = name.as_str().strip_prefix("goods_output_") {
        // TODO: some goods don't have the _mult version. Figure out why.
        for sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::Goods, part, name, data, warn);
                return Some(ModifKinds::Goods);
            }
        }
    }

    // character_$BattleCondition$_mult
    if let Some(part) = name.as_str().strip_prefix("character_") {
        if let Some(part) = part.strip_suffix("_mult") {
            maybe_warn(Item::BattleCondition, part, name, data, warn);
            return Some(ModifKinds::Character);
        }
    }

    // country_$PopType$_pol_str_mult
    // country_$PopType$_voting_power_add
    if let Some(part) = name.as_str().strip_prefix("country_") {
        for sfx in &["_pol_str_mult", "_voting_power_add"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::PopType, part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // country_$Institution$_max_investment_add
    if let Some(part) = name.as_str().strip_prefix("country_") {
        if let Some(part) = part.strip_suffix("_max_investment_add") {
            maybe_warn(Item::Institution, part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // country_subsidies_$BuildingGroup$
    if let Some(part) = name.as_str().strip_prefix("country_subsidies_") {
        maybe_warn(Item::BuildingGroup, part, name, data, warn);
        return Some(ModifKinds::Country);
    }

    // interest_group_$InterestGroup$_approval_add
    // interest_group_$InterestGroup$_pol_str_mult
    // interest_group_$InterestGroup$_pop_attraction_mult
    if let Some(part) = name.as_str().strip_prefix("interest_group_") {
        for sfx in &["_approval_add", "_pol_str_mult", "_pop_attraction_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::InterestGroup, part, name, data, warn);
                return Some(ModifKinds::InterestGroup);
            }
        }
    }

    // state_$Culture$_standard_of_living_add
    // state_$Religion$_standard_of_living_add
    if let Some(part) = name.as_str().strip_prefix("state_") {
        if let Some(part) = part.strip_suffix("_standard_of_living_add") {
            if let Some(sev) = warn {
                if !data.item_exists(Item::Religion, part) && !data.item_exists(Item::Culture, part)
                {
                    let msg = format!("{part} not found as culture or religion");
                    let info = format!("so the modifier {name} does not exist");
                    report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
                }
            }
            return Some(ModifKinds::State);
        }
    }

    // state_$PopType$_dependent_wage_mult
    // state_$PopType$_investment_pool_contribution_add
    // state_$PopType$_investment_pool_efficiency_mult
    // state_$PopType$_mortality_mult
    if let Some(part) = name.as_str().strip_prefix("state_") {
        for sfx in &[
            "_dependent_wage_mult",
            "_investment_pool_contribution_add",
            "_investment_pool_efficiency_mult",
            "_mortality_mult",
        ] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::PopType, part, name, data, warn);
                return Some(ModifKinds::State);
            }
        }
    }

    // state_pop_support_$Law$_add
    // state_pop_support_$Law$_mult
    if let Some(part) = name.as_str().strip_prefix("state_pop_support_") {
        for sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::LawType, part, name, data, warn);
                return Some(ModifKinds::State);
            }
        }
    }

    // TODO: not all of these exist for all unit types
    // unit_$CombatUnit$_offense_mult
    // unit_$CombatUnit$_offense_add
    if let Some(part) = name.as_str().strip_prefix("unit_)") {
        for sfx in &["_offense_add", "_offense_mult"] {
            if let Some(part) = part.strip_suffix(sfx) {
                maybe_warn(Item::CombatUnit, part, name, data, warn);
                return Some(ModifKinds::Unit);
            }
        }
    }

    // TODO: modifiers from terrain labels

    None
}

fn maybe_warn(itype: Item, s: &str, name: &Token, data: &Everything, warn: Option<Severity>) {
    if let Some(sev) = warn {
        if !data.item_exists(itype, s) {
            let msg = format!("could not find {itype} {s}");
            let info = format!("so the modifier {name} does not exist");
            report(ErrorKey::MissingItem, sev).strong().msg(msg).info(info).loc(name).push();
        }
    }
}

// Redeclare the `ModifKinds` enums as bare numbers, so that we can do | on them in const tables.
const Battle: u16 = ModifKinds::Battle.bits();
const Building: u16 = ModifKinds::Building.bits();
const Character: u16 = ModifKinds::Character.bits();
const Country: u16 = ModifKinds::Country.bits();
const InterestGroup: u16 = ModifKinds::InterestGroup.bits();
const Market: u16 = ModifKinds::Market.bits();
const PoliticalMovement: u16 = ModifKinds::PoliticalMovement.bits();
const State: u16 = ModifKinds::State.bits();
const Tariff: u16 = ModifKinds::Tariff.bits();
const Tax: u16 = ModifKinds::Tax.bits();
const Unit: u16 = ModifKinds::Unit.bits();
// const Goods: u16 = ModifKinds::Goods.bits();
const MilitaryFormation: u16 = ModifKinds::MilitaryFormation.bits();

/// LAST UPDATED VIC3 VERSION 1.5.2
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, u16)] = &[
    ("battle_casualties_mult", Battle),
    ("battle_defense_owned_province_mult", Battle),
    ("battle_offense_owned_province_mult", Battle),
    ("building_cash_reserves_mult", Building),
    ("building_economy_of_scale_level_cap_add", Building),
    ("building_government_shares_add", Building),
    ("building_minimum_wage_mult", Building),
    ("building_mobilization_cost_mult", Building),
    ("building_production_mult", Building),
    ("building_subsistence_output_add", Building),
    ("building_subsistence_output_mult", Building),
    ("building_input_mult", Building),
    ("building_throughput_add", Building),
    ("building_throughput_oil_mult", Building),
    ("building_training_rate_add", Building),
    ("building_training_rate_mult", Building),
    ("building_unincorporated_subsistence_output_mult", Building),
    ("building_unincorporated_throughput_add", Building),
    ("building_workforce_shares_add", Building),
    ("building_working_conditions_mult", Building),
    ("character_advancement_speed_add", Character),
    ("character_command_limit_add", Character),
    ("character_command_limit_combat_unit_conscript_add", Character),
    ("character_command_limit_combat_unit_flotilla_add", Character),
    ("character_command_limit_combat_unit_regular_add", Character),
    ("character_command_limit_mult", Character),
    ("character_convoy_protection_mult", Character),
    ("character_convoy_raiding_mult", Character),
    ("character_expedition_events_explorer_mult", Character),
    ("character_health_add", Character),
    ("character_interception_add", Character),
    ("character_morale_cap_add", Character),
    ("character_popularity_add", Character),
    ("character_supply_route_cost_mult", Character),
    ("country_agitator_slots_add", Country),
    ("country_all_buildings_protected", Country),
    ("country_allow_multiple_alliances", Country),
    ("country_authority_add", Country),
    ("country_authority_mult", Country),
    ("country_bureaucracy_add", Country),
    ("country_bureaucracy_investment_cost_factor_mult", Country),
    ("country_bureaucracy_mult", Country),
    ("country_cannot_enact_laws", Country),
    ("country_company_construction_efficiency_bonus_add", Country),
    ("country_company_throughput_bonus_add", Country),
    ("country_construction_add", Country),
    ("country_consumption_tax_cost_mult", Country),
    ("country_convoys_capacity_add", Country),
    ("country_convoys_capacity_mult", Country),
    ("country_damage_relations_speed_mult", Country),
    ("country_decree_cost_mult", Country),
    ("country_diplomatic_play_maneuvers_add", Country),
    ("country_disable_investment_pool", Country),
    ("country_disallow_agitator_invites", Country),
    ("country_disallow_discriminated_migration", Country),
    ("country_disallow_migration", Country),
    ("country_expedition_events_explorer_mult", Country),
    ("country_expenses_add", Country),
    ("country_free_trade_routes_add", Country),
    ("country_gold_reserve_limit_mult", Country),
    ("country_government_buildings_protected", Country),
    ("country_government_wages_mult", Country),
    ("country_ignores_landing_craft_penalty", Country),
    ("country_improve_relations_speed_mult", Country),
    ("country_infamy_decay_mult", Country),
    ("country_infamy_generation_mult", Country),
    ("country_influence_add", Country),
    ("country_influence_mult", Country),
    ("country_institution_size_change_speed_mult", Country),
    ("country_law_enactment_success_add", Country),
    ("country_law_enactment_time_mult", Country),
    ("country_legitimacy_base_add", Country),
    ("country_legitimacy_govt_leader_clout_add", Country),
    ("country_legitimacy_govt_size_add", Country),
    ("country_legitimacy_govt_total_clout_add", Country),
    ("country_legitimacy_govt_total_votes_add", Country),
    ("country_legitimacy_headofstate_add", Country),
    ("country_legitimacy_ideological_incoherence_mult", Country),
    ("country_loan_interest_rate_add", Country),
    ("country_loan_interest_rate_mult", Country),
    ("country_loyalists_from_legitimacy_mult", Country),
    ("country_mandate_subsidies", Country),
    ("country_max_companies_add", Country),
    ("country_max_declared_interests_add", Country),
    ("country_max_declared_interests_mult", Country),
    ("country_max_weekly_construction_progress_add", Country),
    ("country_military_goods_cost_mult", Country),
    ("country_military_tech_research_speed_mult", Country),
    ("country_military_tech_spread_mult", Country),
    ("country_military_wages_mult", Country),
    ("country_minting_add", Country),
    ("country_minting_mult", Country),
    ("country_must_have_movement_to_enact_laws", Country),
    ("country_opposition_ig_approval_add", Country),
    ("country_prestige_add", Country),
    ("country_prestige_from_army_power_projection_mult", Country),
    ("country_prestige_from_navy_power_projection_mult", Country),
    ("country_prestige_mult", Country),
    ("country_private_buildings_protected", Country),
    ("country_private_construction_allocation_mult", Country),
    ("country_production_tech_research_speed_mult", Country),
    ("country_production_tech_spread_mult", Country),
    ("country_promotion_ig_attraction_mult", Country),
    ("country_radicals_from_conquest_mult", Country),
    ("country_radicals_from_legitimacy_mult", Country),
    ("country_resource_depletion_chance_mult", Country),
    ("country_resource_discovery_chance_mult", Country),
    ("country_revolution_clock_time_add", Country),
    ("country_revolution_progress_add", Country),
    ("country_revolution_progress_mult", Country),
    ("country_secession_clock_time_add", Country),
    ("country_secession_progress_add", Country),
    ("country_secession_progress_mult", Country),
    ("country_society_tech_research_speed_mult", Country),
    ("country_society_tech_spread_mult", Country),
    ("country_subsidies_all", Country),
    ("country_suppression_ig_attraction_mult", Country),
    ("country_tax_income_add", Country),
    ("country_tech_research_speed_mult", Country),
    ("country_tech_spread_add", Country),
    ("country_tech_spread_mult", Country),
    ("country_tension_decay_mult", Country),
    ("country_trade_route_competitiveness_mult", Country),
    ("country_trade_route_cost_mult", Country),
    ("country_trade_route_quantity_mult", Country),
    ("country_voting_power_base_add", Country),
    ("country_voting_power_from_literacy_add", Country),
    ("country_voting_power_wealth_threshold_add", Country),
    ("country_war_exhaustion_casualties_mult", Country),
    ("country_weekly_innovation_add", Country),
    ("country_weekly_innovation_max_add", Country),
    ("country_weekly_innovation_mult", Country),
    ("interest_group_approval_add", InterestGroup),
    ("interest_group_in_government_approval_add", InterestGroup),
    ("interest_group_in_government_attraction_mult", InterestGroup),
    ("interest_group_in_opposition_approval_add", InterestGroup),
    ("interest_group_pol_str_factor", InterestGroup),
    ("interest_group_pol_str_mult", InterestGroup),
    ("interest_group_pop_attraction_mult", InterestGroup),
    ("market_disallow_trade_routes", Market),
    ("market_land_trade_capacity_add", Market),
    ("market_max_exports_add", Market),
    ("market_max_imports_add", Market),
    ("military_formation_movement_speed_add", MilitaryFormation),
    ("military_formation_movement_speed_mult", MilitaryFormation),
    ("military_formation_mobilization_speed_add", MilitaryFormation),
    ("military_formation_mobilization_speed_mult", MilitaryFormation),
    ("military_formation_organization_gain_add", MilitaryFormation),
    ("military_formation_organization_gain_mult", MilitaryFormation),
    ("military_formation_attrition_risk_add", MilitaryFormation),
    ("military_formation_attrition_risk_mult", MilitaryFormation),
    ("political_movement_radicalism_add", PoliticalMovement),
    ("political_movement_radicalism_mult", PoliticalMovement),
    ("political_movement_support_add", PoliticalMovement),
    ("political_movement_support_mult", PoliticalMovement),
    ("state_accepted_birth_rate_mult", State),
    ("state_assimilation_mult", State),
    ("state_birth_rate_mult", State),
    ("state_building_barracks_max_level_add", State),
    ("state_building_conscription_center_max_level_add", State),
    ("state_building_construction_sector_max_level_add", State),
    ("state_building_naval_base_max_level_add", State),
    ("state_building_port_max_level_add", State),
    ("state_bureaucracy_population_base_cost_factor_mult", State),
    ("state_colony_growth_creation_mult", State),
    ("state_colony_growth_speed_mult", State),
    ("state_conscription_rate_add", State),
    ("state_conscription_rate_mult", State),
    ("state_construction_mult", State),
    ("state_conversion_mult", State),
    ("state_dependent_political_participation_add", State),
    ("state_dependent_wage_add", State),
    ("state_dependent_wage_mult", State),
    ("state_disallow_incorporation", State),
    ("state_education_access_add", State),
    ("state_education_access_wealth_add", State),
    ("state_expected_sol_from_literacy", State),
    ("state_expected_sol_mult", State),
    ("state_infrastructure_add", State),
    ("state_infrastructure_from_population_add", State),
    ("state_infrastructure_from_population_max_add", State),
    ("state_infrastructure_from_population_max_mult", State),
    ("state_infrastructure_from_population_mult", State),
    ("state_infrastructure_mult", State),
    ("state_loyalists_from_sol_change_accepted_culture_mult", State),
    ("state_loyalists_from_sol_change_accepted_religion_mult", State),
    ("state_loyalists_from_sol_change_mult", State),
    ("state_middle_expected_sol", State),
    ("state_middle_standard_of_living_add", State),
    ("state_migration_pull_add", State),
    ("state_migration_pull_mult", State),
    ("state_migration_pull_unincorporated_mult", State),
    ("state_migration_push_mult", State),
    ("state_minimum_wealth_add", State),
    ("state_market_access_price_impact", State),
    ("state_mortality_mult", State),
    ("state_mortality_turmoil_mult", State),
    ("state_mortality_wealth_mult", State),
    ("state_non_homeland_colony_growth_speed_mult", State),
    ("state_non_homeland_mortality_mult", State),
    ("state_political_strength_from_discrimination_mult", State),
    ("state_political_strength_from_wealth_mult", State),
    ("state_political_strength_from_welfare_mult", State),
    ("state_poor_expected_sol", State),
    ("state_poor_standard_of_living_add", State),
    ("state_pop_pol_str_add", State),
    ("state_pop_pol_str_mult", State),
    ("state_pop_qualifications_mult", State),
    ("state_port_range_add", State),
    ("state_radicals_from_discrimination_mult", State),
    ("state_radicals_from_sol_change_accepted_culture_mult", State),
    ("state_radicals_from_sol_change_accepted_religion_mult", State),
    ("state_radicals_from_sol_change_mult", State),
    ("state_rich_expected_sol", State),
    ("state_rich_standard_of_living_add", State),
    ("state_slave_import_mult", State),
    ("state_standard_of_living_add", State),
    ("state_tax_capacity_add", State),
    ("state_tax_capacity_mult", State),
    ("state_tax_collection_mult", State),
    ("state_tax_waste_add", State),
    ("state_turmoil_effects_mult", State),
    ("state_unincorporated_standard_of_living_add", State),
    ("state_unincorporated_starting_wages_mult", State),
    ("state_urbanization_add", State),
    ("state_urbanization_mult", State),
    ("state_urbanization_per_level_add", State),
    ("state_urbanization_per_level_mult", State),
    ("state_welfare_payments_add", State),
    ("state_working_adult_ratio_add", State),
    ("tariff_export_add", Tariff),
    ("tariff_import_add", Tariff),
    ("tax_consumption_add", Tax),
    ("tax_dividends_add", Tax),
    ("tax_heathen_add", Tax),
    ("tax_income_add", Tax),
    ("tax_land_add", Tax),
    ("tax_per_capita_add", Tax),
    ("unit_advancement_speed_mult", Unit),
    ("unit_army_defense_add", Unit),
    ("unit_army_defense_mult", Unit),
    ("unit_army_offense_add", Unit),
    ("unit_army_offense_mult", Unit),
    ("unit_convoy_defense_mult", Unit),
    ("unit_convoy_raiding_interception_mult", Unit),
    ("unit_convoy_raiding_mult", Unit),
    ("unit_convoy_requirements_mult", Unit),
    ("unit_defense_add", Unit),
    ("unit_defense_developed_add", Unit),
    ("unit_defense_developed_mult", Unit),
    ("unit_defense_elevated_add", Unit),
    ("unit_defense_elevated_mult", Unit),
    ("unit_defense_flat_add", Unit),
    ("unit_defense_flat_mult", Unit),
    ("unit_defense_forested_add", Unit),
    ("unit_defense_forested_mult", Unit),
    ("unit_defense_hazardous_add", Unit),
    ("unit_defense_hazardous_mult", Unit),
    ("unit_defense_mult", Unit),
    ("unit_defense_water_add", Unit),
    ("unit_defense_water_mult", Unit),
    ("unit_devastation_mult", Unit),
    ("unit_kill_rate_add", Unit),
    ("unit_mobilization_speed_mult", Unit),
    ("unit_morale_damage_mult", Unit),
    ("unit_morale_loss_add", Unit),
    ("unit_morale_loss_mult", Unit),
    ("unit_morale_recovery_mult", Unit),
    ("unit_navy_defense_add", Unit),
    ("unit_navy_defense_mult", Unit),
    ("unit_navy_offense_add", Unit),
    ("unit_navy_offense_mult", Unit),
    ("unit_offense_add", Unit),
    ("unit_offense_developed_add", Unit),
    ("unit_offense_developed_mult", Unit),
    ("unit_offense_elevated_add", Unit),
    ("unit_offense_elevated_mult", Unit),
    ("unit_offense_flat_add", Unit),
    ("unit_offense_flat_mult", Unit),
    ("unit_offense_forested_add", Unit),
    ("unit_offense_forested_mult", Unit),
    ("unit_offense_hazardous_add", Unit),
    ("unit_offense_hazardous_mult", Unit),
    ("unit_offense_mult", Unit),
    ("unit_offense_water_add", Unit),
    ("unit_offense_water_mult", Unit),
    ("unit_provinces_captured_mult", Unit),
    ("unit_provinces_lost_mult", Unit),
    ("unit_recovery_rate_add", Unit),
    ("unit_supply_consumption_mult", Unit),
];

const MODIF_REMOVED: &[(&str, &str)] = &[
    ("building_throughput_mult", "replaced in 1.5 with building_throughput_add"),
    ("technology_invention_cost_mult", "replaced in 1.4.0 with country_tech_research_speed_mult"),
    (
        "country_production_tech_cost_mult",
        "replaced in 1.4.0 with country_production_tech_research_speed_mult",
    ),
    (
        "country_production_weekly_innovation_mult",
        "replaced in 1.4.0 with country_production_tech_research_speed_mult",
    ),
    (
        "country_military_tech_cost_mult",
        "replaced in 1.4.0 with country_military_tech_research_speed_mult",
    ),
    (
        "country_military_weekly_innovation_mult",
        "replaced in 1.4.0 with country_military_tech_research_speed_mult",
    ),
    (
        "country_society_tech_cost_mult",
        "replaced in 1.4.0 with country_society_tech_research_speed_mult",
    ),
    (
        "country_society_weekly_innovation_mult",
        "replaced in 1.4.0 with country_society_tech_research_speed_mult",
    ),
    ("country_trade_route_exports_add", "removed in 1.5"),
    ("country_trade_route_imports_add", "removed in 1.5"),
    (
        "country_army_power_projection_add",
        "replaced in 1.5 with country_prestige_from_army_power_projection_mult",
    ),
    (
        "country_army_power_projection_mult",
        "replaced in 1.5 with country_prestige_from_army_power_projection_mult",
    ),
    (
        "country_navy_power_projection_add",
        "replaced in 1.5 with country_prestige_from_navy_power_projection_mult",
    ),
    (
        "country_navy_power_projection_mult",
        "replaced in 1.5 with country_prestige_from_navy_power_projection_mult",
    ),
    ("character_attrition_risk_add", "removed in 1.5"),
    ("character_attrition_risk_mult", "removed in 1.5"),
    ("character_convoy_protection_add", "replaced in 1.5 with character_country_protection_mult"),
    ("character_convoy_raiding_add", "replaced in 1.5 with character_country_raiding_mult"),
    ("front_advancement_speed_add", "removed in 1.5"),
    ("front_advancement_speed_mult", "removed in 1.5"),
    ("front_enemy_advancement_speed_add", "removed in 1.5"),
    ("front_enemy_advancement_speed_mult", "removed in 1.5"),
];
