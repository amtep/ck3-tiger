#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use std::fmt::{Display, Formatter};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

bitflags! {
    pub struct ModifKinds: u8 {
        const Character = 0x01;
        const Province = 0x02;
        const County = 0x04;
        const Terrain = 0x08;
        const Culture = 0x10;
        const Scheme = 0x20;
        const TravelPlan = 0x40;
    }
}

impl ModifKinds {
    pub fn require(self, other: Self, token: &Token) {
        if !self.intersects(other) {
            let msg = format!("`{token}` is a modifier for {other} but expected {self}");
            error(token, ErrorKey::Modifiers, &msg);
        }
    }
}

impl Display for ModifKinds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut vec = Vec::new();
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Province) {
            vec.push("province");
        }
        if self.contains(ModifKinds::County) {
            vec.push("county");
        }
        if self.contains(ModifKinds::Terrain) {
            vec.push("terrain");
        }
        if self.contains(ModifKinds::Culture) {
            vec.push("culture");
        }
        if self.contains(ModifKinds::Scheme) {
            vec.push("scheme");
        }
        if self.contains(ModifKinds::TravelPlan) {
            vec.push("travel plan");
        }
        write!(f, "{}", vec.join(", "))
    }
}

/// LAST UPDATED VERSION 1.9.2
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
pub fn validate_modifs<'a>(
    _block: &Block,
    data: &'a Everything,
    kinds: ModifKinds,
    sc: &mut ScopeContext,
    mut vd: Validator<'a>,
) {
    // TODO: if a modif is for a wrong ModifKind, say so instead of "unknown token"

    if kinds.intersects(ModifKinds::Character) {
        vd.field_script_value("hostile_scheme_power_add", sc);
        vd.field_script_value("hostile_scheme_power_mult", sc);
        vd.field_script_value("hostile_scheme_resistance_add", sc);
        vd.field_script_value("hostile_scheme_resistance_mult", sc);
        vd.field_script_value("max_hostile_schemes_add", sc);
        vd.field_script_value("owned_hostile_scheme_success_chance_add", sc);
        vd.field_script_value("personal_scheme_power_add", sc);
        vd.field_script_value("personal_scheme_power_mult", sc);
        vd.field_script_value("personal_scheme_resistance_add", sc);
        vd.field_script_value("personal_scheme_resistance_mult", sc);
        vd.field_script_value("max_personal_schemes_add", sc);
        vd.field_script_value("owned_personal_scheme_success_chance_add", sc);
        vd.field_script_value("owned_scheme_secrecy_add", sc);
        vd.field_script_value("scheme_discovery_chance_mult", sc);

        vd.field_script_value("ai_amenity_spending", sc);
        vd.field_script_value("ai_amenity_target_baseline", sc);

        vd.field_script_value("ai_boldness", sc);
        vd.field_script_value("ai_compassion", sc);
        vd.field_script_value("ai_energy", sc);
        vd.field_script_value("ai_greed", sc);
        vd.field_script_value("ai_honor", sc);
        vd.field_script_value("ai_rationality", sc);
        vd.field_script_value("ai_sociability", sc);
        vd.field_script_value("ai_vengefulness", sc);
        vd.field_script_value("ai_zeal", sc);

        vd.field_script_value("ai_war_chance", sc);
        vd.field_script_value("ai_war_cooldown", sc);

        vd.field_script_value("army_damage_mult", sc);
        vd.field_script_value("army_maintenance_mult", sc);
        vd.field_script_value("army_pursuit_mult", sc);
        vd.field_script_value("army_screen_mult", sc);
        vd.field_script_value("army_siege_value_mult", sc);
        vd.field_script_value("army_toughness_mult", sc);

        vd.field_script_value("advantage", sc);
        vd.field_script_value("advantage_against_coreligionists", sc);
        vd.field_script_value("attacker_advantage", sc);
        vd.field_script_value("coastal_advantage", sc);
        vd.field_script_value("controlled_province_advantage", sc);
        vd.field_script_value("uncontrolled_province_advantage", sc);
        vd.field_script_value("defender_advantage", sc);
        vd.field_script_value("enemy_terrain_advantage", sc);
        vd.field_script_value("independent_primary_defender_advantage_add", sc);
        vd.field_script_value("led_by_owner_extra_advantage_add", sc);
        vd.field_script_value("random_advantage", sc);
        vd.field_script_value("same_heritage_county_advantage_add", sc);
        vd.field_script_value("winter_advantage", sc);

        vd.field_script_value("max_combat_roll", sc);
        vd.field_script_value("min_combat_roll", sc);

        vd.field_script_value("attraction_opinion", sc);
        vd.field_script_value("child_except_player_heir_opinion", sc);
        vd.field_script_value("child_opinion", sc);
        vd.field_script_value("clergy_opinion", sc);
        vd.field_script_value("close_relative_opinion", sc);
        vd.field_script_value("councillor_opinion", sc);
        vd.field_script_value("county_opinion_add_even_if_baron", sc);
        vd.field_script_value("courtier_and_guest_opinion", sc);
        vd.field_script_value("courtier_opinion", sc);
        vd.field_script_value("different_culture_opinion", sc);
        vd.field_script_value("different_faith_county_opinion_mult", sc);
        vd.field_script_value("different_faith_county_opinion_mult_even_if_baron", sc);
        vd.field_script_value("different_faith_liege_opinion", sc);
        vd.field_script_value("different_faith_opinion", sc);
        vd.field_script_value("direct_vassal_opinion", sc);
        vd.field_script_value("dynasty_house_opinion", sc);
        vd.field_script_value("dynasty_opinion", sc);
        vd.field_script_value("eligible_child_except_player_heir_opinion", sc);
        vd.field_script_value("eligible_child_opinion", sc);
        vd.field_script_value("fellow_vassal_opinion", sc);
        vd.field_script_value("general_opinion", sc);
        vd.field_script_value("guest_opinion", sc);
        vd.field_script_value("independent_ruler_opinion", sc);
        vd.field_script_value("liege_opinion", sc);
        vd.field_script_value("player_heir_opinion", sc);
        vd.field_script_value("powerful_vassal_opinion", sc);
        vd.field_script_value("prisoner_opinion", sc);
        vd.field_script_value("realm_priest_opinion", sc);
        vd.field_script_value("religious_head_opinion", sc);
        vd.field_script_value("religious_vassal_opinion", sc);
        vd.field_script_value("same_culture_opinion", sc);
        vd.field_script_value("same_faith_opinion", sc);
        vd.field_script_value("spouse_opinion", sc);
        vd.field_script_value("twin_opinion", sc);
        vd.field_script_value("vassal_opinion", sc);

        vd.field_script_value(
            "character_capital_county_monthly_development_growth_add",
            sc,
        );

        vd.field_script_value("cowed_vassal_levy_contribution_add", sc);
        vd.field_script_value("cowed_vassal_levy_contribution_mult", sc);
        vd.field_script_value("cowed_vassal_tax_contribution_add", sc);
        vd.field_script_value("cowed_vassal_tax_contribution_mult", sc);
        vd.field_script_value("happy_powerful_vassal_levy_contribution_add", sc);
        vd.field_script_value("happy_powerful_vassal_levy_contribution_mult", sc);
        vd.field_script_value("happy_powerful_vassal_tax_contribution_add", sc);
        vd.field_script_value("happy_powerful_vassal_tax_contribution_mult", sc);
        vd.field_script_value("intimidated_vassal_levy_contribution_add", sc);
        vd.field_script_value("intimidated_vassal_levy_contribution_mult", sc);
        vd.field_script_value("intimidated_vassal_tax_contribution_add", sc);
        vd.field_script_value("intimidated_vassal_tax_contribution_mult", sc);
        vd.field_script_value("vassal_levy_contribution_add", sc);
        vd.field_script_value("vassal_levy_contribution_mult", sc);
        vd.field_script_value("vassal_tax_contribution_add", sc);
        vd.field_script_value("vassal_tax_contribution_mult", sc);

        vd.field_script_value("court_grandeur_baseline_add", sc);
        vd.field_script_value("monthly_court_grandeur_change_add", sc);
        vd.field_script_value("monthly_court_grandeur_change_mult", sc);

        vd.field_script_value("cultural_head_acceptance_gain_mult", sc);
        vd.field_script_value("cultural_head_fascination_add", sc);
        vd.field_script_value("cultural_head_fascination_mult", sc);

        vd.field_script_value("diplomacy", sc);
        vd.field_script_value("diplomacy_per_piety_level", sc);
        vd.field_script_value("diplomacy_per_prestige_level", sc);
        vd.field_script_value("diplomacy_per_stress_level", sc);
        vd.field_script_value("diplomacy_scheme_power", sc);
        vd.field_script_value("diplomacy_scheme_resistance", sc);
        vd.field_script_value("negate_diplomacy_penalty_add", sc);
        vd.field_script_value("intrigue", sc);
        vd.field_script_value("intrigue_per_piety_level", sc);
        vd.field_script_value("intrigue_per_prestige_level", sc);
        vd.field_script_value("intrigue_per_stress_level", sc);
        vd.field_script_value("intrigue_scheme_power", sc);
        vd.field_script_value("intrigue_scheme_resistance", sc);
        vd.field_script_value("negate_intrigue_penalty_add", sc);
        vd.field_script_value("learning", sc);
        vd.field_script_value("learning_per_piety_level", sc);
        vd.field_script_value("learning_per_prestige_level", sc);
        vd.field_script_value("learning_per_stress_level", sc);
        vd.field_script_value("learning_scheme_power", sc);
        vd.field_script_value("learning_scheme_resistance", sc);
        vd.field_script_value("negate_learning_penalty_add", sc);
        vd.field_script_value("martial", sc);
        vd.field_script_value("martial_per_piety_level", sc);
        vd.field_script_value("martial_per_prestige_level", sc);
        vd.field_script_value("martial_per_stress_level", sc);
        vd.field_script_value("martial_scheme_power", sc);
        vd.field_script_value("martial_scheme_resistance", sc);
        vd.field_script_value("negate_martial_penalty_add", sc);
        vd.field_script_value("prowess", sc);
        vd.field_script_value("prowess_no_portrait", sc);
        vd.field_script_value("prowess_per_piety_level", sc);
        vd.field_script_value("prowess_per_prestige_level", sc);
        vd.field_script_value("prowess_per_stress_level", sc);
        vd.field_script_value("prowess_scheme_power", sc);
        vd.field_script_value("prowess_scheme_resistance", sc);
        vd.field_script_value("negate_prowess_penalty_add", sc);
        vd.field_script_value("stewardship", sc);
        vd.field_script_value("stewardship_no_portrait", sc);
        vd.field_script_value("stewardship_per_piety_level", sc);
        vd.field_script_value("stewardship_per_prestige_level", sc);
        vd.field_script_value("stewardship_per_stress_level", sc);
        vd.field_script_value("stewardship_scheme_power", sc);
        vd.field_script_value("stewardship_scheme_resistance", sc);
        vd.field_script_value("negate_stewardship_penalty_add", sc);

        vd.field_script_value("diplomatic_range_mult", sc);

        vd.field_script_value("domain_limit", sc);
        vd.field_script_value("domain_tax_different_faith_mult", sc);
        vd.field_script_value("domain_tax_different_faith_mult_even_if_baron", sc);
        vd.field_script_value("domain_tax_mult", sc);
        vd.field_script_value("domain_tax_mult_even_if_baron", sc);
        vd.field_script_value("domain_tax_same_faith_mult", sc);
        vd.field_script_value("domain_tax_same_faith_mult_even_if_baron", sc);

        vd.field_script_value("dread_baseline_add", sc);
        vd.field_script_value("dread_decay_add", sc);
        vd.field_script_value("dread_decay_mult", sc);
        vd.field_script_value("dread_gain_mult", sc);
        vd.field_script_value("dread_loss_mult", sc);
        vd.field_script_value("dread_per_tyranny_add", sc);
        vd.field_script_value("dread_per_tyranny_mult", sc);
        vd.field_script_value("monthly_dread", sc);

        vd.field_script_value("embarkation_cost_mult", sc);

        vd.field_script_value("enemy_hostile_scheme_success_chance_add", sc);
        vd.field_script_value("enemy_personal_scheme_success_chance_add", sc);

        vd.field_script_value("faith_conversion_piety_cost_add", sc);
        vd.field_script_value("faith_conversion_piety_cost_mult", sc);
        vd.field_script_value("faith_creation_piety_cost_add", sc);
        vd.field_script_value("faith_creation_piety_cost_mult", sc);

        vd.field_script_value("fertility", sc);
        vd.field_script_value("negate_fertility_penalty_add", sc);
        vd.field_script_value("genetic_trait_strengthen_chance", sc);
        vd.field_script_value("inbreeding_chance", sc);
        vd.field_script_value("health", sc);
        vd.field_script_value("negate_health_penalty_add", sc);
        vd.field_script_value("life_expectancy", sc);
        vd.field_script_value("negative_inactive_inheritance_chance", sc);
        vd.field_script_value("negative_random_genetic_chance", sc);
        vd.field_script_value("positive_inactive_inheritance_chance", sc);
        vd.field_script_value("positive_random_genetic_chance", sc);
        vd.field_script_value("years_of_fertility", sc);

        vd.field_script_value("holy_order_hire_cost_add", sc);
        vd.field_script_value("holy_order_hire_cost_mult", sc);
        vd.field_script_value("mercenary_hire_cost_add", sc);
        vd.field_script_value("mercenary_hire_cost_mult", sc);
        vd.field_script_value("same_culture_holy_order_hire_cost_add", sc);
        vd.field_script_value("same_culture_holy_order_hire_cost_mult", sc);
        vd.field_script_value("same_culture_mercenary_hire_cost_add", sc);
        vd.field_script_value("same_culture_mercenary_hire_cost_mult", sc);

        vd.field_script_value("hostile_county_attrition", sc);
        vd.field_script_value("hostile_county_attrition_raiding", sc);

        vd.field_bool("ignore_different_faith_opinion");
        vd.field_bool("ignore_negative_culture_opinion");
        vd.field_bool("ignore_negative_opinion_of_culture");
        vd.field_bool("ignore_opinion_of_different_faith");

        vd.field_script_value("knight_effectiveness_mult", sc);
        vd.field_script_value("knight_effectiveness_per_dread", sc);
        vd.field_script_value("knight_effectiveness_per_tyranny", sc);
        vd.field_script_value("knight_limit", sc);

        vd.field_script_value("levy_attack", sc);
        vd.field_script_value("levy_maintenance", sc);
        vd.field_script_value("levy_pursuit", sc);
        vd.field_script_value("levy_screen", sc);
        vd.field_script_value("levy_siege", sc);
        vd.field_script_value("levy_toughness", sc);

        vd.field_script_value("levy_reinforcement_rate_different_faith", sc);
        vd.field_script_value("levy_reinforcement_rate_different_faith_even_if_baron", sc);
        vd.field_script_value("levy_reinforcement_rate_even_if_baron", sc);
        vd.field_script_value("levy_reinforcement_rate_same_faith", sc);
        vd.field_script_value("levy_reinforcement_rate_same_faith_even_if_baron", sc);

        vd.field_script_value("long_reign_bonus_mult", sc);
        vd.field_script_value("max_loot_mult", sc);

        vd.field_script_value("maa_damage_add", sc);
        vd.field_script_value("maa_damage_mult", sc);
        vd.field_script_value("maa_pursuit_add", sc);
        vd.field_script_value("maa_pursuit_mult", sc);
        vd.field_script_value("maa_screen_add", sc);
        vd.field_script_value("maa_screen_mult", sc);
        vd.field_script_value("maa_siege_value_add", sc);
        vd.field_script_value("maa_siege_value_mult", sc);
        vd.field_script_value("maa_toughness_add", sc);
        vd.field_script_value("maa_toughness_mult", sc);

        vd.field_script_value("men_at_arms_cap", sc);
        vd.field_script_value("men_at_arms_limit", sc);
        vd.field_script_value("men_at_arms_maintenance", sc);
        vd.field_script_value("men_at_arms_maintenance_per_dread_mult", sc);
        vd.field_script_value("men_at_arms_recruitment_cost", sc);

        vd.field_script_value("monthly_county_control_change_add_even_if_baron", sc);
        vd.field_script_value("monthly_county_control_change_factor_even_if_baron", sc);

        vd.field_script_value("monthly_lifestyle_xp_gain_mult", sc);

        vd.field_script_value("monthly_dynasty_prestige", sc);
        vd.field_script_value("monthly_dynasty_prestige_mult", sc);
        vd.field_script_value("monthly_income_mult", sc);
        vd.field_script_value("monthly_income_per_stress_level_add", sc);
        vd.field_script_value("monthly_income_per_stress_level_mult", sc);
        vd.field_script_value("monthly_piety", sc);
        vd.field_script_value("monthly_piety_from_buildings_mult", sc);
        vd.field_script_value("monthly_piety_gain_mult", sc);
        vd.field_script_value("monthly_piety_gain_per_dread_add", sc);
        vd.field_script_value("monthly_piety_gain_per_dread_mult", sc);
        vd.field_script_value("monthly_piety_gain_per_happy_powerful_vassal_add", sc);
        vd.field_script_value("monthly_piety_gain_per_happy_powerful_vassal_mult", sc);
        vd.field_script_value("monthly_piety_gain_per_knight_add", sc);
        vd.field_script_value("monthly_piety_gain_per_knight_mult", sc);
        vd.field_script_value("monthly_prestige", sc);
        vd.field_script_value("monthly_prestige_from_buildings_mult", sc);
        vd.field_script_value("monthly_prestige_gain_mult", sc);
        vd.field_script_value("monthly_prestige_gain_per_dread_add", sc);
        vd.field_script_value("monthly_prestige_gain_per_dread_mult", sc);
        vd.field_script_value("monthly_prestige_gain_per_happy_powerful_vassal_add", sc);
        vd.field_script_value("monthly_prestige_gain_per_happy_powerful_vassal_mult", sc);
        vd.field_script_value("monthly_prestige_gain_per_knight_add", sc);
        vd.field_script_value("monthly_prestige_gain_per_knight_mult", sc);
        vd.field_script_value("monthly_tyranny", sc);
        vd.field_script_value("monthly_war_income_add", sc);
        vd.field_script_value("monthly_war_income_mult", sc);

        vd.field_script_value("movement_speed", sc);
        vd.field_script_value("movement_speed_land_raiding", sc);
        vd.field_script_value("naval_movement_speed_mult", sc);
        vd.field_script_value("raid_speed", sc);
        vd.field_script_value("winter_movement_speed", sc);

        vd.field_bool("no_disembark_penalty");
        vd.field_bool("no_prowess_loss_from_age");
        vd.field_bool("no_water_crossing_penalty");

        vd.field_script_value("opinion_of_different_culture", sc);
        vd.field_script_value("opinion_of_different_faith", sc);
        vd.field_script_value("opinion_of_different_faith_liege", sc);
        vd.field_script_value("opinion_of_female_rulers", sc);
        vd.field_script_value("opinion_of_liege", sc);
        vd.field_script_value("opinion_of_male_rulers", sc);
        vd.field_script_value("opinion_of_parents", sc);
        vd.field_script_value("opinion_of_same_culture", sc);
        vd.field_script_value("opinion_of_same_faith", sc);
        vd.field_script_value("opinion_of_vassal", sc);

        vd.field_script_value("piety_level_impact_mult", sc);
        vd.field_script_value("prestige_level_impact_mult", sc);

        vd.field_script_value("revolting_siege_morale_loss_add", sc);
        vd.field_script_value("revolting_siege_morale_loss_mult", sc);
        vd.field_script_value("siege_morale_loss", sc);
        vd.field_script_value("siege_phase_time", sc);
        vd.field_script_value("short_reign_duration_mult", sc);
        vd.field_script_value("stress_gain_mult", sc);
        vd.field_script_value("stress_loss_mult", sc);
        vd.field_script_value("stress_loss_per_piety_level", sc);
        vd.field_script_value("stress_loss_per_prestige_level", sc);
        vd.field_script_value("supply_capacity_add", sc);
        vd.field_script_value("supply_capacity_mult", sc);
        vd.field_script_value("supply_duration", sc);
        vd.field_script_value("title_creation_cost", sc);
        vd.field_script_value("title_creation_cost_mult", sc);

        vd.field_script_value("tolerance_advantage_mod", sc);
        vd.field_script_value("tyranny_gain_mult", sc);
        vd.field_script_value("tyranny_loss_mult", sc);
        vd.field_script_value("vassal_limit", sc);
        vd.field_script_value("vassal_tax_mult", sc);

        vd.field_script_value("accolade_glory_gain_mult", sc);
        vd.field_script_value("active_accolades", sc);

        vd.field_script_value("character_travel_safety", sc);
        vd.field_script_value("character_travel_safety_mult", sc);
        vd.field_script_value("character_travel_speed", sc);
        vd.field_script_value("character_travel_speed_mult", sc);

        vd.field_script_value("strife_opinion_gain_mult", sc);
        vd.field_script_value("strife_opinion_loss_mult", sc);

        vd.field_script_value("travel_companion_opinion", sc);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::County) {
        vd.field_script_value("county_opinion_add", sc);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::Province) {
        vd.field_script_value("monthly_income", sc);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::Terrain) {
        vd.field_script_value("counter_efficiency", sc);
        vd.field_script_value("counter_resistance", sc);
        vd.field_script_value("enemy_hard_casualty_modifier", sc);
        vd.field_script_value("hard_casualty_modifier", sc);
        vd.field_script_value("pursue_efficiency", sc);
        vd.field_script_value("retreat_losses", sc);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::County | ModifKinds::Province) {
        vd.field_script_value("additional_fort_level", sc);
        vd.field_script_value("artifact_decay_reduction_mult", sc);
        vd.field_script_value("build_gold_cost", sc);
        vd.field_script_value("build_piety_cost", sc);
        vd.field_script_value("build_prestige_cost", sc);
        vd.field_script_value("build_speed", sc);
        vd.field_script_value("building_slot_add", sc);
        vd.field_script_value("holding_build_gold_cost", sc);
        vd.field_script_value("holding_build_piety_cost", sc);
        vd.field_script_value("holding_build_prestige_cost", sc);
        vd.field_script_value("holding_build_speed", sc);
        vd.field_script_value("castle_holding_build_gold_cost", sc);
        vd.field_script_value("castle_holding_build_piety_cost", sc);
        vd.field_script_value("castle_holding_build_prestige_cost", sc);
        vd.field_script_value("castle_holding_build_speed", sc);
        vd.field_script_value("castle_holding_holding_build_gold_cost", sc);
        vd.field_script_value("castle_holding_holding_build_piety_cost", sc);
        vd.field_script_value("castle_holding_holding_build_prestige_cost", sc);
        vd.field_script_value("castle_holding_holding_build_speed", sc);
        vd.field_script_value("church_holding_build_gold_cost", sc);
        vd.field_script_value("church_holding_build_piety_cost", sc);
        vd.field_script_value("church_holding_build_prestige_cost", sc);
        vd.field_script_value("church_holding_build_speed", sc);
        vd.field_script_value("church_holding_holding_build_gold_cost", sc);
        vd.field_script_value("church_holding_holding_build_piety_cost", sc);
        vd.field_script_value("church_holding_holding_build_prestige_cost", sc);
        vd.field_script_value("church_holding_holding_build_speed", sc);
        vd.field_script_value("city_holding_build_gold_cost", sc);
        vd.field_script_value("city_holding_build_piety_cost", sc);
        vd.field_script_value("city_holding_build_prestige_cost", sc);
        vd.field_script_value("city_holding_build_speed", sc);
        vd.field_script_value("city_holding_holding_build_gold_cost", sc);
        vd.field_script_value("city_holding_holding_build_piety_cost", sc);
        vd.field_script_value("city_holding_holding_build_prestige_cost", sc);
        vd.field_script_value("city_holding_holding_build_speed", sc);
        vd.field_script_value("tribal_holding_build_gold_cost", sc);
        vd.field_script_value("tribal_holding_build_piety_cost", sc);
        vd.field_script_value("tribal_holding_build_prestige_cost", sc);
        vd.field_script_value("tribal_holding_build_speed", sc);
        vd.field_script_value("tribal_holding_holding_build_gold_cost", sc);
        vd.field_script_value("tribal_holding_holding_build_piety_cost", sc);
        vd.field_script_value("tribal_holding_holding_build_prestige_cost", sc);
        vd.field_script_value("tribal_holding_holding_build_speed", sc);
        vd.field_script_value("defender_holding_advantage", sc);
        vd.field_script_value("development_growth", sc);
        vd.field_script_value("development_growth_factor", sc);
        vd.field_script_value("fort_level", sc);
        vd.field_script_value("garrison_size", sc);
        vd.field_script_value("hostile_raid_time", sc);
        vd.field_script_value("levy_reinforcement_rate", sc);
        vd.field_script_value("levy_reinforcement_rate_friendly_territory", sc);
        vd.field_script_value("levy_size", sc);
        vd.field_script_value("monthly_county_control_change_add", sc);
        vd.field_script_value("monthly_county_control_change_factor", sc);
        vd.field_script_value("monthly_county_control_change_at_war_add", sc);
        vd.field_script_value("monthly_county_control_change_at_war_mult", sc);
        vd.field_script_value("supply_limit", sc);
        vd.field_script_value("supply_limit_mult", sc);
        vd.field_script_value("tax_mult", sc);
        vd.field_script_value("supply_limit_mult", sc);
        vd.field_script_value("travel_danger", sc);
    }

    if kinds.intersects(ModifKinds::Culture) {
        vd.field_script_value("cultural_acceptance_gain_mult", sc);
        vd.field_script_value("culture_tradition_max_add", sc);
        vd.field_script_value("mercenary_count_mult", sc);
    }

    if kinds.intersects(ModifKinds::Province) {
        vd.field_script_value("defender_winter_advantage", sc);
        vd.field_script_value("hard_casualty_winter", sc);
        vd.field_script_value("supply_loss_winter", sc);
        vd.field_script_value("stationed_maa_damage_add", sc);
        vd.field_script_value("stationed_maa_damage_mult", sc);
        vd.field_script_value("stationed_maa_pursuit_add", sc);
        vd.field_script_value("stationed_maa_pursuit_mult", sc);
        vd.field_script_value("stationed_maa_screen_add", sc);
        vd.field_script_value("stationed_maa_screen_mult", sc);
        vd.field_script_value("stationed_maa_siege_value_add", sc);
        vd.field_script_value("stationed_maa_siege_value_mult", sc);
        vd.field_script_value("stationed_maa_toughness_add", sc);
        vd.field_script_value("stationed_maa_toughness_mult", sc);
    }

    if kinds.intersects(ModifKinds::Scheme) {
        vd.field_script_value("scheme_power", sc);
        vd.field_script_value("scheme_resistance", sc);
        vd.field_script_value("scheme_secrecy", sc);
        vd.field_script_value("scheme_success_chance", sc);
    }

    if kinds.intersects(ModifKinds::TravelPlan) {
        vd.field_script_value("travel_safety", sc);
        vd.field_script_value("travel_safety_mult", sc);
        vd.field_script_value("travel_speed", sc);
        vd.field_script_value("travel_speed_mult", sc);
    }

    'outer: for (token, bv) in vd.unknown_keys() {
        for terrain_sfx in &[
            "_advantage",
            "_attrition_mult",
            "_cancel_negative_supply",
            "_max_combat_roll",
            "_min_combat_roll",
        ] {
            if let Some(terrain) = token.as_str().strip_suffix(terrain_sfx) {
                data.verify_exists_implied(Item::Terrain, terrain, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        for terrain_sfx in &[
            "_holding_construction_gold_cost",
            "_holding_construction_piety_cost",
            "_holding_construction_prestige_cost",
            "_construction_gold_cost",
            "_construction_piety_cost",
            "_construction_prestige_cost",
            "_levy_size",
            "_supply_limit",
            "_supply_limit_mult",
            "_tax_mult",
            "_travel_danger",
        ] {
            if let Some(terrain) = token.as_str().strip_suffix(terrain_sfx) {
                data.verify_exists_implied(Item::Terrain, terrain, token);
                kinds.require(
                    ModifKinds::Character | ModifKinds::County | ModifKinds::Province,
                    token,
                );
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        if let Some(x) = token.as_str().strip_suffix("_xp_gain_mult") {
            if let Some(lifestyle) = x.strip_prefix("monthly_") {
                data.verify_exists_implied(Item::Lifestyle, lifestyle, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue;
            }
        }

        for trait_track_sfx in &["_xp_degradation_mult", "_xp_gain_mult", "_xp_loss_mult"] {
            if let Some(trait_track) = token.as_str().strip_suffix(trait_track_sfx) {
                data.verify_exists_implied(Item::TraitTrack, trait_track, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        for sfx in &[
            "_levy_contribution_add",
            "_levy_contribution_mult",
            "_tax_contribution_add",
            "_tax_contribution_mult",
        ] {
            if let Some(pfx) = token.as_str().strip_suffix(sfx) {
                if !data.item_exists(Item::VassalStance, pfx)
                    && !data.item_exists(Item::GovernmentType, pfx)
                {
                    let msg = format!("unknown vassal stance or government type `{pfx}`");
                    error(token, ErrorKey::MissingItem, &msg);
                }
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        for vassal_stance_sfx in &[
            "_different_culture_opinion",
            "_different_faith_opinion",
            "_same_culture_opinion",
            "_same_faith_opinion",
        ] {
            if let Some(vassal_stance) = token.as_str().strip_suffix(vassal_stance_sfx) {
                data.verify_exists_implied(Item::VassalStance, vassal_stance, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        for scheme_sfx in &[
            "_scheme_power_add",
            "_scheme_power_mult",
            "_scheme_resistance_add",
            "_scheme_resistance_mult",
        ] {
            if let Some(scheme) = token.as_str().strip_suffix(scheme_sfx) {
                data.verify_exists_implied(Item::Scheme, scheme, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }
        if let Some(x) = token.as_str().strip_suffix("_schemes_add") {
            if let Some(scheme) = x.strip_prefix("max_") {
                data.verify_exists_implied(Item::Scheme, scheme, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue;
            }
        }

        if let Some(x) = token.as_str().strip_prefix("scheme_power_against_") {
            if let Some(relation) = x.strip_suffix("_add") {
                data.verify_exists_implied(Item::Relation, relation, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue;
            }
            if let Some(relation) = x.strip_suffix("_mult") {
                data.verify_exists_implied(Item::Relation, relation, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue;
            }
        }

        for maa_sfx in &[
            "_damage_add",
            "_damage_mult",
            "_maintenance_mult",
            "_max_size_add",
            "_max_size_mult",
            "_pursuit_add",
            "_pursuit_mult",
            "_recruitment_cost_mult",
            "_screen_add",
            "_screen_mult",
            "_siege_value_add",
            "_siege_value_mult",
            "_toughness_add",
            "_toughness_mult",
        ] {
            if let Some(mut maa_base) = token.as_str().strip_suffix(maa_sfx) {
                let mut require = ModifKinds::Character;
                if let Some(real_maa_base) = maa_base.strip_prefix("stationed_") {
                    maa_base = real_maa_base;
                    require = ModifKinds::Province;
                }
                data.verify_exists_implied(Item::MenAtArmsBase, maa_base, token);
                kinds.require(require, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        for sfx in &["_opinion_same_faith", "_vassal_opinion"] {
            if let Some(government) = token.as_str().strip_suffix(sfx) {
                data.verify_exists_implied(Item::GovernmentType, government, token);
                kinds.require(ModifKinds::Character, token);
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        for sfx in &["_development_growth", "_development_growth_factor"] {
            if let Some(something) = token.as_str().strip_suffix(sfx) {
                // TODO: if a region, also check that it has set generate_modifiers = yes
                if !data.item_exists(Item::Region, something)
                    && !data.item_exists(Item::Terrain, something)
                {
                    let msg = "unknown terrain or geographical region";
                    error(token, ErrorKey::MissingItem, msg);
                }
                if data.item_exists(Item::Region, something)
                    && !data.item_has_property(Item::Region, something, "generates_modifiers")
                {
                    let msg = format!("region {something} does not have generates_modifiers = yes");
                    let info = format!("so the modifier {token} does not exist");
                    error_info(token, ErrorKey::Validation, &msg, &info);
                }
                kinds.require(
                    ModifKinds::Character | ModifKinds::Province | ModifKinds::County,
                    token,
                );
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        for sfx in &[
            "_build_gold_cost",
            "_build_piety_cost",
            "_build_prestige_cost",
            "_build_speed",
            "_holding_build_gold_cost",
            "_holding_build_piety_cost",
            "_holding_build_prestige_cost",
            "_holding_build_speed",
        ] {
            if let Some(region) = token.as_str().strip_suffix(sfx) {
                data.verify_exists_implied(Item::Region, region, token);
                kinds.require(
                    ModifKinds::Character | ModifKinds::Province | ModifKinds::County,
                    token,
                );
                ScriptValue::validate_bv(bv, data, sc);
                continue 'outer;
            }
        }

        if let Some(something) = token.as_str().strip_suffix("_opinion") {
            if !data.item_exists(Item::Religion, something)
                && !data.item_exists(Item::Faith, something)
                && !data.item_exists(Item::Culture, something)
                && !data.item_exists(Item::ReligionFamily, something)
                && !data.item_exists(Item::GovernmentType, something)
                && !data.item_exists(Item::VassalStance, something)
            {
                error(token, ErrorKey::MissingItem, "unknown opinion type (not faith, religion, religious family, culture, or government, or vassal stance)");
            }
            kinds.require(ModifKinds::Character, token);
            ScriptValue::validate_bv(bv, data, sc);
            continue;
        }

        let msg = format!("unknown modifier `{token}`");
        warn(token, ErrorKey::Validation, &msg);
    }
}
