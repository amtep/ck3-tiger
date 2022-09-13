#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use std::fmt::{Display, Formatter};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
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
    }
}

impl ModifKinds {
    pub fn require(self, other: Self, token: &Token) {
        if !self.intersects(other) {
            let msg = format!("`{}` is a modifier for {}", token, self);
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
        write!(f, "{}", vec.join(", "))
    }
}

/// LAST UPDATED VERSION 1.7.0
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
        vd.field_numeric("hostile_scheme_power_add");
        vd.field_numeric("hostile_scheme_power_mult");
        vd.field_numeric("hostile_scheme_resistance_add");
        vd.field_numeric("hostile_scheme_resistance_mult");
        vd.field_numeric("max_hostile_schemes_add");
        vd.field_numeric("owned_hostile_scheme_success_chance_add");
        vd.field_numeric("personal_scheme_power_add");
        vd.field_numeric("personal_scheme_power_mult");
        vd.field_numeric("personal_scheme_resistance_add");
        vd.field_numeric("personal_scheme_resistance_mult");
        vd.field_numeric("max_personal_schemes_add");
        vd.field_numeric("owned_personal_scheme_success_chance_add");
        vd.field_numeric("owned_scheme_secrecy_add");
        vd.field_numeric("scheme_discovery_chance_mult");
        // TODO: scheme_power_against_<relation>_add
        // TODO: scheme_power_against_<relation>_mult

        // TODO: <maa_base_type>_damage_add
        // TODO: <maa_base_type>_damage_mult
        // TODO: <maa_base_type>_maintenance_mult
        // TODO: <maa_base_type>_max_size_add
        // TODO: <maa_base_type>_max_size_mult
        // TODO: <maa_base_type>_pursuit_add
        // TODO: <maa_base_type>_pursuit_mult
        // TODO: <maa_base_type>_recruitment_cost_mult
        // TODO: <maa_base_type>_screen_add
        // TODO: <maa_base_type>_screen_mult
        // TODO: <maa_base_type>_siege_value_add
        // TODO: <maa_base_type>_siege_value_mult
        // TODO: <maa_base_type>_toughness_add
        // TODO: <maa_base_type>_toughness_mult

        vd.field_numeric("ai_amenity_spending");
        vd.field_numeric("ai_amenity_target_baseline");

        vd.field_script_value("ai_boldness", sc);
        vd.field_script_value("ai_compassion", sc);
        vd.field_script_value("ai_energy", sc);
        vd.field_script_value("ai_greed", sc);
        vd.field_script_value("ai_honor", sc);
        vd.field_script_value("ai_rationality", sc);
        vd.field_script_value("ai_sociability", sc);
        vd.field_script_value("ai_vengefulness", sc);
        vd.field_script_value("ai_zeal", sc);

        vd.field_numeric("ai_war_chance");
        vd.field_numeric("ai_war_cooldown");

        vd.field_numeric("army_damage_mult");
        vd.field_numeric("army_maintenance_mult");
        vd.field_numeric("army_pursuit_mult");
        vd.field_numeric("army_screen_mult");
        vd.field_numeric("army_siege_value_mult");
        vd.field_numeric("army_toughness_mult");

        vd.field_numeric("advantage");
        vd.field_numeric("advantage_against_coreligionists");
        vd.field_numeric("attacker_advantage");
        vd.field_numeric("coastal_advantage");
        vd.field_numeric("controlled_province_advantage");
        vd.field_numeric("defender_advantage");
        vd.field_numeric("enemy_terrain_advantage");
        vd.field_numeric("independent_primary_defender_advantage_add");
        vd.field_numeric("led_by_owner_extra_advantage_add");
        vd.field_numeric("random_advantage");
        vd.field_numeric("same_heritage_county_advantage_add");
        vd.field_numeric("winter_advantage");

        vd.field_numeric("max_combat_roll");
        vd.field_numeric("min_combat_roll");

        vd.field_numeric("attraction_opinion");
        vd.field_numeric("child_except_player_heir_opinion");
        vd.field_numeric("child_opinion");
        vd.field_numeric("clergy_opinion");
        vd.field_numeric("close_relative_opinion");
        vd.field_numeric("councillor_opinion");
        vd.field_numeric("county_opinion_add_even_if_baron");
        vd.field_numeric("courtier_and_guest_opinion");
        vd.field_numeric("courtier_opinion");
        vd.field_numeric("different_culture_opinion");
        vd.field_numeric("different_faith_county_opinion_mult");
        vd.field_numeric("different_faith_county_opinion_mult_even_if_baron");
        vd.field_numeric("different_faith_liege_opinion");
        vd.field_numeric("different_faith_opinion");
        vd.field_numeric("direct_vassal_opinion");
        vd.field_numeric("dynasty_house_opinion");
        vd.field_numeric("dynasty_opinion");
        vd.field_numeric("eligible_child_except_player_heir_opinion");
        vd.field_numeric("eligible_child_opinion");
        vd.field_numeric("fellow_vassal_opinion");
        vd.field_numeric("general_opinion");
        vd.field_numeric("guest_opinion");
        vd.field_numeric("independent_ruler_opinion");
        vd.field_numeric("liege_opinion");
        vd.field_numeric("player_heir_opinion");
        vd.field_numeric("powerful_vassal_opinion");
        vd.field_numeric("prisoner_opinion");
        vd.field_numeric("realm_priest_opinion");
        vd.field_numeric("religious_head_opinion");
        vd.field_numeric("religious_vassal_opinion");
        vd.field_numeric("same_culture_opinion");
        vd.field_numeric("same_faith_opinion");
        vd.field_numeric("spouse_opinion");
        vd.field_numeric("twin_opinion");
        vd.field_numeric("vassal_opinion");
        // TODO: <government>_opinion_same_faith
        // TODO: <government>_vassal_opinion

        vd.field_numeric("character_capital_county_monthly_development_growth_add");

        // TODO: <government>_levy_contribution_add
        // TODO: <government>_levy_contribution_mult
        // TODO: <government>_tax_contribution_add
        // TODO: <government>_tax_contribution_mult
        vd.field_numeric("cowed_vassal_levy_contribution_add");
        vd.field_numeric("cowed_vassal_levy_contribution_mult");
        vd.field_numeric("cowed_vassal_tax_contribution_add");
        vd.field_numeric("cowed_vassal_tax_contribution_mult");
        vd.field_numeric("happy_powerful_vassal_levy_contribution_add");
        vd.field_numeric("happy_powerful_vassal_levy_contribution_mult");
        vd.field_numeric("happy_powerful_vassal_tax_contribution_add");
        vd.field_numeric("happy_powerful_vassal_tax_contribution_mult");
        vd.field_numeric("intimidated_vassal_levy_contribution_add");
        vd.field_numeric("intimidated_vassal_levy_contribution_mult");
        vd.field_numeric("intimidated_vassal_tax_contribution_add");
        vd.field_numeric("intimidated_vassal_tax_contribution_mult");
        vd.field_numeric("vassal_levy_contribution_add");
        vd.field_numeric("vassal_levy_contribution_mult");
        vd.field_numeric("vassal_tax_contribution_add");
        vd.field_numeric("vassal_tax_contribution_mult");

        vd.field_numeric("court_grandeur_baseline_add");
        vd.field_numeric("monthly_court_grandeur_change_add");
        vd.field_numeric("monthly_court_grandeur_change_mult");

        vd.field_numeric("cultural_head_acceptance_gain_mult");
        vd.field_numeric("cultural_head_fascination_add");
        vd.field_numeric("cultural_head_fascination_mult");

        vd.field_script_value("diplomacy", sc);
        vd.field_numeric("diplomacy_per_piety_level");
        vd.field_numeric("diplomacy_per_prestige_level");
        vd.field_numeric("diplomacy_per_stress_level");
        vd.field_numeric("diplomacy_scheme_power");
        vd.field_numeric("diplomacy_scheme_resistance");
        vd.field_numeric("negate_diplomacy_penalty_add");
        vd.field_script_value("intrigue", sc);
        vd.field_numeric("intrigue_per_piety_level");
        vd.field_numeric("intrigue_per_prestige_level");
        vd.field_numeric("intrigue_per_stress_level");
        vd.field_numeric("intrigue_scheme_power");
        vd.field_numeric("intrigue_scheme_resistance");
        vd.field_numeric("negate_intrigue_penalty_add");
        vd.field_script_value("learning", sc);
        vd.field_numeric("learning_per_piety_level");
        vd.field_numeric("learning_per_prestige_level");
        vd.field_numeric("learning_per_stress_level");
        vd.field_numeric("learning_scheme_power");
        vd.field_numeric("learning_scheme_resistance");
        vd.field_numeric("negate_learning_penalty_add");
        vd.field_script_value("martial", sc);
        vd.field_numeric("martial_per_piety_level");
        vd.field_numeric("martial_per_prestige_level");
        vd.field_numeric("martial_per_stress_level");
        vd.field_numeric("martial_scheme_power");
        vd.field_numeric("martial_scheme_resistance");
        vd.field_numeric("negate_martial_penalty_add");
        vd.field_script_value("prowess", sc);
        vd.field_numeric("prowess_no_portrait");
        vd.field_numeric("prowess_per_piety_level");
        vd.field_numeric("prowess_per_prestige_level");
        vd.field_numeric("prowess_per_stress_level");
        vd.field_numeric("prowess_scheme_power");
        vd.field_numeric("prowess_scheme_resistance");
        vd.field_numeric("negate_prowess_penalty_add");
        vd.field_script_value("stewardship", sc);
        vd.field_numeric("stewardship_no_portrait");
        vd.field_numeric("stewardship_per_piety_level");
        vd.field_numeric("stewardship_per_prestige_level");
        vd.field_numeric("stewardship_per_stress_level");
        vd.field_numeric("stewardship_scheme_power");
        vd.field_numeric("stewardship_scheme_resistance");
        vd.field_numeric("negate_stewardship_penalty_add");

        vd.field_numeric("diplomatic_range_mult");

        vd.field_numeric("domain_limit");
        vd.field_numeric("domain_tax_different_faith_mult");
        vd.field_numeric("domain_tax_different_faith_mult_even_if_baron");
        vd.field_numeric("domain_tax_mult");
        vd.field_numeric("domain_tax_mult_even_if_baron");
        vd.field_numeric("domain_tax_same_faith_mult");
        vd.field_numeric("domain_tax_same_faith_mult_even_if_baron");

        vd.field_numeric("dread_baseline_add");
        vd.field_numeric("dread_decay_add");
        vd.field_numeric("dread_decay_mult");
        vd.field_numeric("dread_gain_mult");
        vd.field_numeric("dread_loss_mult");
        vd.field_numeric("dread_per_tyranny_add");
        vd.field_numeric("dread_per_tyranny_mult");
        vd.field_numeric("monthly_dread");

        vd.field_numeric("embarkation_cost_mult");

        vd.field_numeric("enemy_hostile_scheme_success_chance_add");
        vd.field_numeric("enemy_personal_scheme_success_chance_add");

        vd.field_numeric("faith_conversion_piety_cost_add");
        vd.field_numeric("faith_conversion_piety_cost_mult");
        vd.field_numeric("faith_creation_piety_cost_add");
        vd.field_numeric("faith_creation_piety_cost_mult");

        vd.field_numeric("fertility");
        vd.field_numeric("negate_fertility_penalty_add");
        vd.field_numeric("genetic_trait_strengthen_chance");
        vd.field_numeric("inbreeding_chance");
        vd.field_numeric("health");
        vd.field_numeric("negate_health_penalty_add");
        vd.field_numeric("life_expectancy");
        vd.field_numeric("negative_inactive_inheritance_chance");
        vd.field_numeric("negative_random_genetic_chance");
        vd.field_numeric("positive_inactive_inheritance_chance");
        vd.field_numeric("positive_random_genetic_chance");
        vd.field_numeric("years_of_fertility");

        vd.field_numeric("holy_order_hire_cost_add");
        vd.field_numeric("holy_order_hire_cost_mult");
        vd.field_numeric("mercenary_hire_cost_add");
        vd.field_numeric("mercenary_hire_cost_mult");
        vd.field_numeric("same_culture_holy_order_hire_cost_add");
        vd.field_numeric("same_culture_holy_order_hire_cost_mult");
        vd.field_numeric("same_culture_mercenary_hire_cost_add");
        vd.field_numeric("same_culture_mercenary_hire_cost_mult");

        vd.field_numeric("hostile_county_attrition");

        vd.field_bool("ignore_different_faith_opinion");
        vd.field_bool("ignore_negative_culture_opinion");
        vd.field_bool("ignore_negative_opinion_of_culture");
        vd.field_bool("ignore_opinion_of_different_faith");

        vd.field_numeric("knight_effectiveness_mult");
        vd.field_numeric("knight_limit");

        vd.field_numeric("levy_reinforcement_rate_different_faith");
        vd.field_numeric("levy_reinforcement_rate_different_faith_even_if_baron");
        vd.field_numeric("levy_reinforcement_rate_even_if_baron");
        vd.field_numeric("levy_reinforcement_rate_same_faith");
        vd.field_numeric("levy_reinforcement_rate_same_faith_even_if_baron");

        vd.field_numeric("long_reign_bonus_mult");
        vd.field_numeric("max_loot_mult");

        vd.field_numeric("men_at_arms_cap");
        vd.field_numeric("men_at_arms_limit");
        vd.field_numeric("men_at_arms_maintenance");
        vd.field_numeric("men_at_arms_maintenance_per_dread_mult");
        vd.field_numeric("men_at_arms_recruitment_cost");

        vd.field_numeric("monthly_county_control_change_add_even_if_baron");
        vd.field_numeric("monthly_county_control_change_factor_even_if_baron");

        vd.field_numeric("monthly_lifestyle_xp_gain_mult");

        vd.field_numeric("monthly_dynasty_prestige");
        vd.field_numeric("monthly_dynasty_prestige_mult");
        vd.field_numeric("monthly_income_mult");
        vd.field_numeric("monthly_income_per_stress_level_add");
        vd.field_numeric("monthly_income_per_stress_level_mult");
        vd.field_script_value("monthly_piety", sc);
        vd.field_numeric("monthly_piety_from_buildings_mult");
        vd.field_numeric("monthly_piety_gain_mult");
        vd.field_numeric("monthly_piety_gain_per_dread_add");
        vd.field_numeric("monthly_piety_gain_per_dread_mult");
        vd.field_numeric("monthly_piety_gain_per_happy_powerful_vassal_add");
        vd.field_numeric("monthly_piety_gain_per_happy_powerful_vassal_mult");
        vd.field_numeric("monthly_piety_gain_per_knight_add");
        vd.field_numeric("monthly_piety_gain_per_knight_mult");
        vd.field_script_value("monthly_prestige", sc);
        vd.field_numeric("monthly_prestige_from_buildings_mult");
        vd.field_numeric("monthly_prestige_gain_mult");
        vd.field_numeric("monthly_prestige_gain_per_dread_add");
        vd.field_numeric("monthly_prestige_gain_per_dread_mult");
        vd.field_numeric("monthly_prestige_gain_per_happy_powerful_vassal_add");
        vd.field_numeric("monthly_prestige_gain_per_happy_powerful_vassal_mult");
        vd.field_numeric("monthly_prestige_gain_per_knight_add");
        vd.field_numeric("monthly_prestige_gain_per_knight_mult");
        vd.field_numeric("monthly_tyranny");
        vd.field_numeric("monthly_war_income_add");
        vd.field_numeric("monthly_war_income_mult");

        vd.field_numeric("movement_speed");
        vd.field_numeric("naval_movement_speed_mult");
        vd.field_numeric("raid_speed");
        vd.field_numeric("winter_movement_speed");

        vd.field_bool("no_disembark_penalty");
        vd.field_bool("no_prowess_loss_from_age");
        vd.field_bool("no_water_crossing_penalty");

        vd.field_numeric("opinion_of_different_culture");
        vd.field_numeric("opinion_of_different_faith");
        vd.field_numeric("opinion_of_different_faith_liege");
        vd.field_numeric("opinion_of_female_rulers");
        vd.field_numeric("opinion_of_liege");
        vd.field_numeric("opinion_of_male_rulers");
        vd.field_numeric("opinion_of_parents");
        vd.field_numeric("opinion_of_same_culture");
        vd.field_numeric("opinion_of_same_faith");
        vd.field_numeric("opinion_of_vassal");

        vd.field_numeric("piety_level_impact_mult");
        vd.field_numeric("prestige_level_impact_mult");

        vd.field_numeric("random_skills_per_child");

        vd.field_numeric("revolting_siege_morale_loss_add");
        vd.field_numeric("revolting_siege_morale_loss_mult");
        vd.field_numeric("siege_morale_loss");
        vd.field_numeric("siege_phase_time");
        vd.field_numeric("short_reign_duration_mult");
        vd.field_numeric("stress_gain_mult");
        vd.field_numeric("stress_loss_mult");
        vd.field_numeric("supply_capacity_add");
        vd.field_numeric("supply_capacity_mult");
        vd.field_numeric("supply_duration");
        vd.field_numeric("title_creation_cost");
        vd.field_numeric("title_creation_cost_mult");

        vd.field_numeric("tolerance_advantage_mod");
        vd.field_numeric("tyranny_gain_mult");
        vd.field_numeric("tyranny_loss_mult");
        vd.field_numeric("vassal_limit");
        vd.field_numeric("vassal_tax_mult");
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::County) {
        vd.field_numeric("county_opinion_add");
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::Province) {
        vd.field_numeric("monthly_income");
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::Terrain) {
        vd.field_numeric("counter_efficiency");
        vd.field_numeric("enemy_hard_casualty_modifier");
        vd.field_numeric("hard_casualty_modifier");
        vd.field_numeric("pursue_efficiency");
        vd.field_numeric("retreat_losses");
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::County | ModifKinds::Province) {
        vd.field_numeric("additional_fort_level");
        vd.field_numeric("artifact_decay_reduction_mult");
        vd.field_numeric("build_gold_cost");
        vd.field_numeric("build_piety_cost");
        vd.field_numeric("build_prestige_cost");
        vd.field_numeric("build_speed");
        vd.field_numeric("holding_build_gold_cost");
        vd.field_numeric("holding_build_piety_cost");
        vd.field_numeric("holding_build_prestige_cost");
        vd.field_numeric("holding_build_speed");
        vd.field_numeric("castle_holding_build_gold_cost");
        vd.field_numeric("castle_holding_build_piety_cost");
        vd.field_numeric("castle_holding_build_prestige_cost");
        vd.field_numeric("castle_holding_build_speed");
        vd.field_numeric("castle_holding_holding_build_gold_cost");
        vd.field_numeric("castle_holding_holding_build_piety_cost");
        vd.field_numeric("castle_holding_holding_build_prestige_cost");
        vd.field_numeric("castle_holding_holding_build_speed");
        vd.field_numeric("church_holding_build_gold_cost");
        vd.field_numeric("church_holding_build_piety_cost");
        vd.field_numeric("church_holding_build_prestige_cost");
        vd.field_numeric("church_holding_build_speed");
        vd.field_numeric("church_holding_holding_build_gold_cost");
        vd.field_numeric("church_holding_holding_build_piety_cost");
        vd.field_numeric("church_holding_holding_build_prestige_cost");
        vd.field_numeric("church_holding_holding_build_speed");
        vd.field_numeric("city_holding_build_gold_cost");
        vd.field_numeric("city_holding_build_piety_cost");
        vd.field_numeric("city_holding_build_prestige_cost");
        vd.field_numeric("city_holding_build_speed");
        vd.field_numeric("city_holding_holding_build_gold_cost");
        vd.field_numeric("city_holding_holding_build_piety_cost");
        vd.field_numeric("city_holding_holding_build_prestige_cost");
        vd.field_numeric("city_holding_holding_build_speed");
        vd.field_numeric("tribal_holding_build_gold_cost");
        vd.field_numeric("tribal_holding_build_piety_cost");
        vd.field_numeric("tribal_holding_build_prestige_cost");
        vd.field_numeric("tribal_holding_build_speed");
        vd.field_numeric("tribal_holding_holding_build_gold_cost");
        vd.field_numeric("tribal_holding_holding_build_piety_cost");
        vd.field_numeric("tribal_holding_holding_build_prestige_cost");
        vd.field_numeric("tribal_holding_holding_build_speed");
        vd.field_numeric("defender_holding_advantage");
        vd.field_numeric("development_growth");
        vd.field_numeric("development_growth_factor");
        vd.field_numeric("fort_level");
        vd.field_numeric("garrison_size");
        vd.field_numeric("hostile_raid_time");
        vd.field_numeric("levy_reinforcement_rate");
        vd.field_numeric("levy_reinforcement_rate_friendly_territory");
        vd.field_numeric("levy_size");
        vd.field_numeric("monthly_county_control_change_add");
        vd.field_numeric("monthly_county_control_change_factor");
        vd.field_numeric("monthly_county_control_change_at_war_add");
        vd.field_numeric("monthly_county_control_change_at_war_mult");
        vd.field_numeric("supply_limit");
        vd.field_numeric("supply_limit_mult");
        vd.field_numeric("tax_mult");
        vd.field_numeric("supply_limit_mult");
        vd.field_numeric("world_innovation_camels_development_growth");
        vd.field_numeric("world_innovation_camels_development_growth_factor");
        vd.field_numeric("world_innovation_elephants_development_growth");
        vd.field_numeric("world_innovation_elephants_development_growth_factor");
        vd.field_numeric("world_steppe_development_growth");
        vd.field_numeric("world_steppe_development_growth_factor");
    }

    if kinds.intersects(ModifKinds::Culture) {
        vd.field_numeric("cultural_acceptance_gain_mult");
        vd.field_numeric("culture_tradition_max_add");
        vd.field_numeric("mercenary_count_mult");
    }

    if kinds.intersects(ModifKinds::Province) {
        vd.field_numeric("defender_winter_advantage");
        vd.field_numeric("hard_casualty_winter");
        vd.field_numeric("supply_loss_winter");
    }

    if kinds.intersects(ModifKinds::Scheme) {
        vd.field_numeric("scheme_power");
        vd.field_numeric("scheme_resistance");
        vd.field_numeric("scheme_secrecy");
        vd.field_numeric("scheme_success_chance");
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
                if let Some(value) = bv.expect_value() {
                    if value.as_str().parse::<f64>().is_err() {
                        error(value, ErrorKey::Validation, "expected number");
                    }
                }
                continue 'outer;
            }
        }

        for terrain_sfx in &[
            "_construction_gold_cost",
            "_construction_piety_cost",
            "_construction_prestige_cost",
            "_development_growth",
            "_development_growth_factor",
            "_holding_construction_gold_cost",
            "_holding_construction_piety_cost",
            "_holding_construction_prestige_cost",
            "_levy_size",
            "_supply_limit",
            "_supply_limit_mult",
            "_tax_mult",
        ] {
            if let Some(terrain) = token.as_str().strip_suffix(terrain_sfx) {
                data.verify_exists_implied(Item::Terrain, terrain, token);
                kinds.require(
                    ModifKinds::Character | ModifKinds::County | ModifKinds::Province,
                    token,
                );
                if let Some(value) = bv.expect_value() {
                    if value.as_str().parse::<f64>().is_err() {
                        error(value, ErrorKey::Validation, "expected number");
                    }
                }
                continue 'outer;
            }
        }

        if let Some(x) = token.as_str().strip_suffix("_xp_gain_mult") {
            if let Some(lifestyle) = x.strip_prefix("monthly_") {
                data.verify_exists_implied(Item::Lifestyle, lifestyle, token);
                kinds.require(ModifKinds::Character, token);
                if let Some(value) = bv.expect_value() {
                    if value.as_str().parse::<f64>().is_err() {
                        error(value, ErrorKey::Validation, "expected number");
                    }
                }
                continue;
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
                if let Some(value) = bv.expect_value() {
                    if value.as_str().parse::<f64>().is_err() {
                        error(value, ErrorKey::Validation, "expected number");
                    }
                }
                continue 'outer;
            }
        }
        if let Some(x) = token.as_str().strip_suffix("_schemes_add") {
            if let Some(scheme) = x.strip_prefix("max_") {
                data.verify_exists_implied(Item::Scheme, scheme, token);
                kinds.require(ModifKinds::Character, token);
                if let Some(value) = bv.expect_value() {
                    if value.as_str().parse::<f64>().is_err() {
                        error(value, ErrorKey::Validation, "expected number");
                    }
                }
                continue;
            }
        }

        if let Some(something) = token.as_str().strip_suffix("_opinion") {
            if !data.item_exists(Item::Religion, something)
                && !data.item_exists(Item::Faith, something)
                && !data.item_exists(Item::Culture, something)
                && !data.item_exists(Item::ReligiousFamily, something)
                && !data.item_exists(Item::Government, something)
            {
                error(token, ErrorKey::MissingItem, "unknown opinion type (not faith, religion, religious family, culture, or government)");
            }
            kinds.require(ModifKinds::Character, token);
            if let Some(value) = bv.expect_value() {
                if value.as_str().parse::<f64>().is_err() {
                    error(value, ErrorKey::Validation, "expected number");
                }
            }
            continue;
        }

        let msg = format!("unknown modifier `{}`", token);
        warn(token, ErrorKey::Validation, &msg);
    }

    vd.warn_remaining();
}
