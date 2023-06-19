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
use crate::scopes::Scopes;
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
    mut vd: Validator<'a>,
) {
    // TODO: if a modif is for a wrong ModifKind, say so instead of "unknown token"

    if kinds.intersects(ModifKinds::Character) {
        vd.field_script_value_rooted("hostile_scheme_power_add", Scopes::None);
        vd.field_script_value_rooted("hostile_scheme_power_mult", Scopes::None);
        vd.field_script_value_rooted("hostile_scheme_resistance_add", Scopes::None);
        vd.field_script_value_rooted("hostile_scheme_resistance_mult", Scopes::None);
        vd.field_script_value_rooted("max_hostile_schemes_add", Scopes::None);
        vd.field_script_value_rooted("owned_hostile_scheme_success_chance_add", Scopes::None);
        vd.field_script_value_rooted("personal_scheme_power_add", Scopes::None);
        vd.field_script_value_rooted("personal_scheme_power_mult", Scopes::None);
        vd.field_script_value_rooted("personal_scheme_resistance_add", Scopes::None);
        vd.field_script_value_rooted("personal_scheme_resistance_mult", Scopes::None);
        vd.field_script_value_rooted("max_personal_schemes_add", Scopes::None);
        vd.field_script_value_rooted("owned_personal_scheme_success_chance_add", Scopes::None);
        vd.field_script_value_rooted("owned_scheme_secrecy_add", Scopes::None);
        vd.field_script_value_rooted("scheme_discovery_chance_mult", Scopes::None);

        vd.field_script_value_rooted("ai_amenity_spending", Scopes::None);
        vd.field_script_value_rooted("ai_amenity_target_baseline", Scopes::None);

        vd.field_script_value_rooted("ai_boldness", Scopes::None);
        vd.field_script_value_rooted("ai_compassion", Scopes::None);
        vd.field_script_value_rooted("ai_energy", Scopes::None);
        vd.field_script_value_rooted("ai_greed", Scopes::None);
        vd.field_script_value_rooted("ai_honor", Scopes::None);
        vd.field_script_value_rooted("ai_rationality", Scopes::None);
        vd.field_script_value_rooted("ai_sociability", Scopes::None);
        vd.field_script_value_rooted("ai_vengefulness", Scopes::None);
        vd.field_script_value_rooted("ai_zeal", Scopes::None);

        vd.field_script_value_rooted("ai_war_chance", Scopes::None);
        vd.field_script_value_rooted("ai_war_cooldown", Scopes::None);

        vd.field_script_value_rooted("army_damage_mult", Scopes::None);
        vd.field_script_value_rooted("army_maintenance_mult", Scopes::None);
        vd.field_script_value_rooted("army_pursuit_mult", Scopes::None);
        vd.field_script_value_rooted("army_screen_mult", Scopes::None);
        vd.field_script_value_rooted("army_siege_value_mult", Scopes::None);
        vd.field_script_value_rooted("army_toughness_mult", Scopes::None);

        vd.field_script_value_rooted("advantage", Scopes::None);
        vd.field_script_value_rooted("advantage_against_coreligionists", Scopes::None);
        vd.field_script_value_rooted("attacker_advantage", Scopes::None);
        vd.field_script_value_rooted("coastal_advantage", Scopes::None);
        vd.field_script_value_rooted("controlled_province_advantage", Scopes::None);
        vd.field_script_value_rooted("uncontrolled_province_advantage", Scopes::None);
        vd.field_script_value_rooted("defender_advantage", Scopes::None);
        vd.field_script_value_rooted("enemy_terrain_advantage", Scopes::None);
        vd.field_script_value_rooted("independent_primary_defender_advantage_add", Scopes::None);
        vd.field_script_value_rooted("led_by_owner_extra_advantage_add", Scopes::None);
        vd.field_script_value_rooted("random_advantage", Scopes::None);
        vd.field_script_value_rooted("same_heritage_county_advantage_add", Scopes::None);
        vd.field_script_value_rooted("winter_advantage", Scopes::None);

        vd.field_script_value_rooted("max_combat_roll", Scopes::None);
        vd.field_script_value_rooted("min_combat_roll", Scopes::None);

        vd.field_script_value_rooted("attraction_opinion", Scopes::None);
        vd.field_script_value_rooted("child_except_player_heir_opinion", Scopes::None);
        vd.field_script_value_rooted("child_opinion", Scopes::None);
        vd.field_script_value_rooted("clergy_opinion", Scopes::None);
        vd.field_script_value_rooted("close_relative_opinion", Scopes::None);
        vd.field_script_value_rooted("councillor_opinion", Scopes::None);
        vd.field_script_value_rooted("county_opinion_add_even_if_baron", Scopes::None);
        vd.field_script_value_rooted("courtier_and_guest_opinion", Scopes::None);
        vd.field_script_value_rooted("courtier_opinion", Scopes::None);
        vd.field_script_value_rooted("different_culture_opinion", Scopes::None);
        vd.field_script_value_rooted("different_faith_county_opinion_mult", Scopes::None);
        vd.field_script_value_rooted(
            "different_faith_county_opinion_mult_even_if_baron",
            Scopes::None,
        );
        vd.field_script_value_rooted("different_faith_liege_opinion", Scopes::None);
        vd.field_script_value_rooted("different_faith_opinion", Scopes::None);
        vd.field_script_value_rooted("direct_vassal_opinion", Scopes::None);
        vd.field_script_value_rooted("dynasty_house_opinion", Scopes::None);
        vd.field_script_value_rooted("dynasty_opinion", Scopes::None);
        vd.field_script_value_rooted("eligible_child_except_player_heir_opinion", Scopes::None);
        vd.field_script_value_rooted("eligible_child_opinion", Scopes::None);
        vd.field_script_value_rooted("fellow_vassal_opinion", Scopes::None);
        vd.field_script_value_rooted("general_opinion", Scopes::None);
        vd.field_script_value_rooted("guest_opinion", Scopes::None);
        vd.field_script_value_rooted("independent_ruler_opinion", Scopes::None);
        vd.field_script_value_rooted("liege_opinion", Scopes::None);
        vd.field_script_value_rooted("player_heir_opinion", Scopes::None);
        vd.field_script_value_rooted("powerful_vassal_opinion", Scopes::None);
        vd.field_script_value_rooted("prisoner_opinion", Scopes::None);
        vd.field_script_value_rooted("realm_priest_opinion", Scopes::None);
        vd.field_script_value_rooted("religious_head_opinion", Scopes::None);
        vd.field_script_value_rooted("religious_vassal_opinion", Scopes::None);
        vd.field_script_value_rooted("same_culture_opinion", Scopes::None);
        vd.field_script_value_rooted("same_faith_opinion", Scopes::None);
        vd.field_script_value_rooted("spouse_opinion", Scopes::None);
        vd.field_script_value_rooted("twin_opinion", Scopes::None);
        vd.field_script_value_rooted("vassal_opinion", Scopes::None);

        vd.field_script_value_rooted(
            "character_capital_county_monthly_development_growth_add",
            Scopes::None,
        );

        vd.field_script_value_rooted("cowed_vassal_levy_contribution_add", Scopes::None);
        vd.field_script_value_rooted("cowed_vassal_levy_contribution_mult", Scopes::None);
        vd.field_script_value_rooted("cowed_vassal_tax_contribution_add", Scopes::None);
        vd.field_script_value_rooted("cowed_vassal_tax_contribution_mult", Scopes::None);
        vd.field_script_value_rooted("happy_powerful_vassal_levy_contribution_add", Scopes::None);
        vd.field_script_value_rooted("happy_powerful_vassal_levy_contribution_mult", Scopes::None);
        vd.field_script_value_rooted("happy_powerful_vassal_tax_contribution_add", Scopes::None);
        vd.field_script_value_rooted("happy_powerful_vassal_tax_contribution_mult", Scopes::None);
        vd.field_script_value_rooted("intimidated_vassal_levy_contribution_add", Scopes::None);
        vd.field_script_value_rooted("intimidated_vassal_levy_contribution_mult", Scopes::None);
        vd.field_script_value_rooted("intimidated_vassal_tax_contribution_add", Scopes::None);
        vd.field_script_value_rooted("intimidated_vassal_tax_contribution_mult", Scopes::None);
        vd.field_script_value_rooted("vassal_levy_contribution_add", Scopes::None);
        vd.field_script_value_rooted("vassal_levy_contribution_mult", Scopes::None);
        vd.field_script_value_rooted("vassal_tax_contribution_add", Scopes::None);
        vd.field_script_value_rooted("vassal_tax_contribution_mult", Scopes::None);

        vd.field_script_value_rooted("court_grandeur_baseline_add", Scopes::None);
        vd.field_script_value_rooted("monthly_court_grandeur_change_add", Scopes::None);
        vd.field_script_value_rooted("monthly_court_grandeur_change_mult", Scopes::None);

        vd.field_script_value_rooted("cultural_head_acceptance_gain_mult", Scopes::None);
        vd.field_script_value_rooted("cultural_head_fascination_add", Scopes::None);
        vd.field_script_value_rooted("cultural_head_fascination_mult", Scopes::None);

        vd.field_script_value_rooted("diplomacy", Scopes::None);
        vd.field_script_value_rooted("diplomacy_per_piety_level", Scopes::None);
        vd.field_script_value_rooted("diplomacy_per_prestige_level", Scopes::None);
        vd.field_script_value_rooted("diplomacy_per_stress_level", Scopes::None);
        vd.field_script_value_rooted("diplomacy_scheme_power", Scopes::None);
        vd.field_script_value_rooted("diplomacy_scheme_resistance", Scopes::None);
        vd.field_script_value_rooted("negate_diplomacy_penalty_add", Scopes::None);
        vd.field_script_value_rooted("intrigue", Scopes::None);
        vd.field_script_value_rooted("intrigue_per_piety_level", Scopes::None);
        vd.field_script_value_rooted("intrigue_per_prestige_level", Scopes::None);
        vd.field_script_value_rooted("intrigue_per_stress_level", Scopes::None);
        vd.field_script_value_rooted("intrigue_scheme_power", Scopes::None);
        vd.field_script_value_rooted("intrigue_scheme_resistance", Scopes::None);
        vd.field_script_value_rooted("negate_intrigue_penalty_add", Scopes::None);
        vd.field_script_value_rooted("learning", Scopes::None);
        vd.field_script_value_rooted("learning_per_piety_level", Scopes::None);
        vd.field_script_value_rooted("learning_per_prestige_level", Scopes::None);
        vd.field_script_value_rooted("learning_per_stress_level", Scopes::None);
        vd.field_script_value_rooted("learning_scheme_power", Scopes::None);
        vd.field_script_value_rooted("learning_scheme_resistance", Scopes::None);
        vd.field_script_value_rooted("negate_learning_penalty_add", Scopes::None);
        vd.field_script_value_rooted("martial", Scopes::None);
        vd.field_script_value_rooted("martial_per_piety_level", Scopes::None);
        vd.field_script_value_rooted("martial_per_prestige_level", Scopes::None);
        vd.field_script_value_rooted("martial_per_stress_level", Scopes::None);
        vd.field_script_value_rooted("martial_scheme_power", Scopes::None);
        vd.field_script_value_rooted("martial_scheme_resistance", Scopes::None);
        vd.field_script_value_rooted("negate_martial_penalty_add", Scopes::None);
        vd.field_script_value_rooted("prowess", Scopes::None);
        vd.field_script_value_rooted("prowess_no_portrait", Scopes::None);
        vd.field_script_value_rooted("prowess_per_piety_level", Scopes::None);
        vd.field_script_value_rooted("prowess_per_prestige_level", Scopes::None);
        vd.field_script_value_rooted("prowess_per_stress_level", Scopes::None);
        vd.field_script_value_rooted("prowess_scheme_power", Scopes::None);
        vd.field_script_value_rooted("prowess_scheme_resistance", Scopes::None);
        vd.field_script_value_rooted("negate_prowess_penalty_add", Scopes::None);
        vd.field_script_value_rooted("stewardship", Scopes::None);
        vd.field_script_value_rooted("stewardship_no_portrait", Scopes::None);
        vd.field_script_value_rooted("stewardship_per_piety_level", Scopes::None);
        vd.field_script_value_rooted("stewardship_per_prestige_level", Scopes::None);
        vd.field_script_value_rooted("stewardship_per_stress_level", Scopes::None);
        vd.field_script_value_rooted("stewardship_scheme_power", Scopes::None);
        vd.field_script_value_rooted("stewardship_scheme_resistance", Scopes::None);
        vd.field_script_value_rooted("negate_stewardship_penalty_add", Scopes::None);

        vd.field_script_value_rooted("diplomatic_range_mult", Scopes::None);

        vd.field_script_value_rooted("domain_limit", Scopes::None);
        vd.field_script_value_rooted("domain_tax_different_faith_mult", Scopes::None);
        vd.field_script_value_rooted(
            "domain_tax_different_faith_mult_even_if_baron",
            Scopes::None,
        );
        vd.field_script_value_rooted("domain_tax_mult", Scopes::None);
        vd.field_script_value_rooted("domain_tax_mult_even_if_baron", Scopes::None);
        vd.field_script_value_rooted("domain_tax_same_faith_mult", Scopes::None);
        vd.field_script_value_rooted("domain_tax_same_faith_mult_even_if_baron", Scopes::None);

        vd.field_script_value_rooted("dread_baseline_add", Scopes::None);
        vd.field_script_value_rooted("dread_decay_add", Scopes::None);
        vd.field_script_value_rooted("dread_decay_mult", Scopes::None);
        vd.field_script_value_rooted("dread_gain_mult", Scopes::None);
        vd.field_script_value_rooted("dread_loss_mult", Scopes::None);
        vd.field_script_value_rooted("dread_per_tyranny_add", Scopes::None);
        vd.field_script_value_rooted("dread_per_tyranny_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_dread", Scopes::None);

        vd.field_script_value_rooted("embarkation_cost_mult", Scopes::None);

        vd.field_script_value_rooted("enemy_hostile_scheme_success_chance_add", Scopes::None);
        vd.field_script_value_rooted("enemy_personal_scheme_success_chance_add", Scopes::None);

        vd.field_script_value_rooted("faith_conversion_piety_cost_add", Scopes::None);
        vd.field_script_value_rooted("faith_conversion_piety_cost_mult", Scopes::None);
        vd.field_script_value_rooted("faith_creation_piety_cost_add", Scopes::None);
        vd.field_script_value_rooted("faith_creation_piety_cost_mult", Scopes::None);

        vd.field_script_value_rooted("fertility", Scopes::None);
        vd.field_script_value_rooted("negate_fertility_penalty_add", Scopes::None);
        vd.field_script_value_rooted("genetic_trait_strengthen_chance", Scopes::None);
        vd.field_script_value_rooted("inbreeding_chance", Scopes::None);
        vd.field_script_value_rooted("health", Scopes::None);
        vd.field_script_value_rooted("negate_health_penalty_add", Scopes::None);
        vd.field_script_value_rooted("life_expectancy", Scopes::None);
        vd.field_script_value_rooted("negative_inactive_inheritance_chance", Scopes::None);
        vd.field_script_value_rooted("negative_random_genetic_chance", Scopes::None);
        vd.field_script_value_rooted("positive_inactive_inheritance_chance", Scopes::None);
        vd.field_script_value_rooted("positive_random_genetic_chance", Scopes::None);
        vd.field_script_value_rooted("years_of_fertility", Scopes::None);

        vd.field_script_value_rooted("holy_order_hire_cost_add", Scopes::None);
        vd.field_script_value_rooted("holy_order_hire_cost_mult", Scopes::None);
        vd.field_script_value_rooted("mercenary_hire_cost_add", Scopes::None);
        vd.field_script_value_rooted("mercenary_hire_cost_mult", Scopes::None);
        vd.field_script_value_rooted("same_culture_holy_order_hire_cost_add", Scopes::None);
        vd.field_script_value_rooted("same_culture_holy_order_hire_cost_mult", Scopes::None);
        vd.field_script_value_rooted("same_culture_mercenary_hire_cost_add", Scopes::None);
        vd.field_script_value_rooted("same_culture_mercenary_hire_cost_mult", Scopes::None);

        vd.field_script_value_rooted("hostile_county_attrition", Scopes::None);
        vd.field_script_value_rooted("hostile_county_attrition_raiding", Scopes::None);

        vd.field_bool("ignore_different_faith_opinion");
        vd.field_bool("ignore_negative_culture_opinion");
        vd.field_bool("ignore_negative_opinion_of_culture");
        vd.field_bool("ignore_opinion_of_different_faith");

        vd.field_script_value_rooted("knight_effectiveness_mult", Scopes::None);
        vd.field_script_value_rooted("knight_effectiveness_per_dread", Scopes::None);
        vd.field_script_value_rooted("knight_effectiveness_per_tyranny", Scopes::None);
        vd.field_script_value_rooted("knight_limit", Scopes::None);

        vd.field_script_value_rooted("levy_attack", Scopes::None);
        vd.field_script_value_rooted("levy_maintenance", Scopes::None);
        vd.field_script_value_rooted("levy_pursuit", Scopes::None);
        vd.field_script_value_rooted("levy_screen", Scopes::None);
        vd.field_script_value_rooted("levy_siege", Scopes::None);
        vd.field_script_value_rooted("levy_toughness", Scopes::None);

        vd.field_script_value_rooted("levy_reinforcement_rate_different_faith", Scopes::None);
        vd.field_script_value_rooted(
            "levy_reinforcement_rate_different_faith_even_if_baron",
            Scopes::None,
        );
        vd.field_script_value_rooted("levy_reinforcement_rate_even_if_baron", Scopes::None);
        vd.field_script_value_rooted("levy_reinforcement_rate_same_faith", Scopes::None);
        vd.field_script_value_rooted(
            "levy_reinforcement_rate_same_faith_even_if_baron",
            Scopes::None,
        );

        vd.field_script_value_rooted("long_reign_bonus_mult", Scopes::None);
        vd.field_script_value_rooted("max_loot_mult", Scopes::None);

        vd.field_script_value_rooted("maa_damage_add", Scopes::None);
        vd.field_script_value_rooted("maa_damage_mult", Scopes::None);
        vd.field_script_value_rooted("maa_pursuit_add", Scopes::None);
        vd.field_script_value_rooted("maa_pursuit_mult", Scopes::None);
        vd.field_script_value_rooted("maa_screen_add", Scopes::None);
        vd.field_script_value_rooted("maa_screen_mult", Scopes::None);
        vd.field_script_value_rooted("maa_siege_value_add", Scopes::None);
        vd.field_script_value_rooted("maa_siege_value_mult", Scopes::None);
        vd.field_script_value_rooted("maa_toughness_add", Scopes::None);
        vd.field_script_value_rooted("maa_toughness_mult", Scopes::None);

        vd.field_script_value_rooted("men_at_arms_cap", Scopes::None);
        vd.field_script_value_rooted("men_at_arms_limit", Scopes::None);
        vd.field_script_value_rooted("men_at_arms_maintenance", Scopes::None);
        vd.field_script_value_rooted("men_at_arms_maintenance_per_dread_mult", Scopes::None);
        vd.field_script_value_rooted("men_at_arms_recruitment_cost", Scopes::None);

        vd.field_script_value_rooted(
            "monthly_county_control_change_add_even_if_baron",
            Scopes::None,
        );
        vd.field_script_value_rooted(
            "monthly_county_control_change_factor_even_if_baron",
            Scopes::None,
        );

        vd.field_script_value_rooted("monthly_lifestyle_xp_gain_mult", Scopes::None);

        vd.field_script_value_rooted("monthly_dynasty_prestige", Scopes::None);
        vd.field_script_value_rooted("monthly_dynasty_prestige_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_income_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_income_per_stress_level_add", Scopes::None);
        vd.field_script_value_rooted("monthly_income_per_stress_level_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_piety", Scopes::None);
        vd.field_script_value_rooted("monthly_piety_from_buildings_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_piety_gain_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_piety_gain_per_dread_add", Scopes::None);
        vd.field_script_value_rooted("monthly_piety_gain_per_dread_mult", Scopes::None);
        vd.field_script_value_rooted(
            "monthly_piety_gain_per_happy_powerful_vassal_add",
            Scopes::None,
        );
        vd.field_script_value_rooted(
            "monthly_piety_gain_per_happy_powerful_vassal_mult",
            Scopes::None,
        );
        vd.field_script_value_rooted("monthly_piety_gain_per_knight_add", Scopes::None);
        vd.field_script_value_rooted("monthly_piety_gain_per_knight_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_prestige", Scopes::None);
        vd.field_script_value_rooted("monthly_prestige_from_buildings_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_prestige_gain_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_prestige_gain_per_dread_add", Scopes::None);
        vd.field_script_value_rooted("monthly_prestige_gain_per_dread_mult", Scopes::None);
        vd.field_script_value_rooted(
            "monthly_prestige_gain_per_happy_powerful_vassal_add",
            Scopes::None,
        );
        vd.field_script_value_rooted(
            "monthly_prestige_gain_per_happy_powerful_vassal_mult",
            Scopes::None,
        );
        vd.field_script_value_rooted("monthly_prestige_gain_per_knight_add", Scopes::None);
        vd.field_script_value_rooted("monthly_prestige_gain_per_knight_mult", Scopes::None);
        vd.field_script_value_rooted("monthly_tyranny", Scopes::None);
        vd.field_script_value_rooted("monthly_war_income_add", Scopes::None);
        vd.field_script_value_rooted("monthly_war_income_mult", Scopes::None);

        vd.field_script_value_rooted("movement_speed", Scopes::None);
        vd.field_script_value_rooted("movement_speed_land_raiding", Scopes::None);
        vd.field_script_value_rooted("naval_movement_speed_mult", Scopes::None);
        vd.field_script_value_rooted("raid_speed", Scopes::None);
        vd.field_script_value_rooted("winter_movement_speed", Scopes::None);

        vd.field_bool("no_disembark_penalty");
        vd.field_bool("no_prowess_loss_from_age");
        vd.field_bool("no_water_crossing_penalty");

        vd.field_script_value_rooted("opinion_of_different_culture", Scopes::None);
        vd.field_script_value_rooted("opinion_of_different_faith", Scopes::None);
        vd.field_script_value_rooted("opinion_of_different_faith_liege", Scopes::None);
        vd.field_script_value_rooted("opinion_of_female_rulers", Scopes::None);
        vd.field_script_value_rooted("opinion_of_liege", Scopes::None);
        vd.field_script_value_rooted("opinion_of_male_rulers", Scopes::None);
        vd.field_script_value_rooted("opinion_of_parents", Scopes::None);
        vd.field_script_value_rooted("opinion_of_same_culture", Scopes::None);
        vd.field_script_value_rooted("opinion_of_same_faith", Scopes::None);
        vd.field_script_value_rooted("opinion_of_vassal", Scopes::None);

        vd.field_script_value_rooted("piety_level_impact_mult", Scopes::None);
        vd.field_script_value_rooted("prestige_level_impact_mult", Scopes::None);

        vd.field_script_value_rooted("revolting_siege_morale_loss_add", Scopes::None);
        vd.field_script_value_rooted("revolting_siege_morale_loss_mult", Scopes::None);
        vd.field_script_value_rooted("siege_morale_loss", Scopes::None);
        vd.field_script_value_rooted("siege_phase_time", Scopes::None);
        vd.field_script_value_rooted("short_reign_duration_mult", Scopes::None);
        vd.field_script_value_rooted("stress_gain_mult", Scopes::None);
        vd.field_script_value_rooted("stress_loss_mult", Scopes::None);
        vd.field_script_value_rooted("stress_loss_per_piety_level", Scopes::None);
        vd.field_script_value_rooted("stress_loss_per_prestige_level", Scopes::None);
        vd.field_script_value_rooted("supply_capacity_add", Scopes::None);
        vd.field_script_value_rooted("supply_capacity_mult", Scopes::None);
        vd.field_script_value_rooted("supply_duration", Scopes::None);
        vd.field_script_value_rooted("title_creation_cost", Scopes::None);
        vd.field_script_value_rooted("title_creation_cost_mult", Scopes::None);

        vd.field_script_value_rooted("tolerance_advantage_mod", Scopes::None);
        vd.field_script_value_rooted("tyranny_gain_mult", Scopes::None);
        vd.field_script_value_rooted("tyranny_loss_mult", Scopes::None);
        vd.field_script_value_rooted("vassal_limit", Scopes::None);
        vd.field_script_value_rooted("vassal_tax_mult", Scopes::None);

        vd.field_script_value_rooted("accolade_glory_gain_mult", Scopes::None);
        vd.field_script_value_rooted("active_accolades", Scopes::None);

        vd.field_script_value_rooted("character_travel_safety", Scopes::None);
        vd.field_script_value_rooted("character_travel_safety_mult", Scopes::None);
        vd.field_script_value_rooted("character_travel_speed", Scopes::None);
        vd.field_script_value_rooted("character_travel_speed_mult", Scopes::None);

        vd.field_script_value_rooted("strife_opinion_gain_mult", Scopes::None);
        vd.field_script_value_rooted("strife_opinion_loss_mult", Scopes::None);

        vd.field_script_value_rooted("travel_companion_opinion", Scopes::None);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::County) {
        vd.field_script_value_rooted("county_opinion_add", Scopes::None);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::Province) {
        vd.field_script_value_rooted("monthly_income", Scopes::None);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::Terrain) {
        vd.field_script_value_rooted("counter_efficiency", Scopes::None);
        vd.field_script_value_rooted("counter_resistance", Scopes::None);
        vd.field_script_value_rooted("enemy_hard_casualty_modifier", Scopes::None);
        vd.field_script_value_rooted("hard_casualty_modifier", Scopes::None);
        vd.field_script_value_rooted("pursue_efficiency", Scopes::None);
        vd.field_script_value_rooted("retreat_losses", Scopes::None);
    }

    if kinds.intersects(ModifKinds::Character | ModifKinds::County | ModifKinds::Province) {
        vd.field_script_value_rooted("additional_fort_level", Scopes::None);
        vd.field_script_value_rooted("artifact_decay_reduction_mult", Scopes::None);
        vd.field_script_value_rooted("build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("build_speed", Scopes::None);
        vd.field_script_value_rooted("building_slot_add", Scopes::None);
        vd.field_script_value_rooted("holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("castle_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("castle_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("castle_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("castle_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("castle_holding_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("castle_holding_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("castle_holding_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("castle_holding_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("church_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("church_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("church_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("church_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("church_holding_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("church_holding_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("church_holding_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("church_holding_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("city_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("city_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("city_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("city_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("city_holding_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("city_holding_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("city_holding_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("city_holding_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_holding_build_gold_cost", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_holding_build_piety_cost", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_holding_build_prestige_cost", Scopes::None);
        vd.field_script_value_rooted("tribal_holding_holding_build_speed", Scopes::None);
        vd.field_script_value_rooted("defender_holding_advantage", Scopes::None);
        vd.field_script_value_rooted("development_growth", Scopes::None);
        vd.field_script_value_rooted("development_growth_factor", Scopes::None);
        vd.field_script_value_rooted("fort_level", Scopes::None);
        vd.field_script_value_rooted("garrison_size", Scopes::None);
        vd.field_script_value_rooted("hostile_raid_time", Scopes::None);
        vd.field_script_value_rooted("levy_reinforcement_rate", Scopes::None);
        vd.field_script_value_rooted("levy_reinforcement_rate_friendly_territory", Scopes::None);
        vd.field_script_value_rooted("levy_size", Scopes::None);
        vd.field_script_value_rooted("monthly_county_control_change_add", Scopes::None);
        vd.field_script_value_rooted("monthly_county_control_change_factor", Scopes::None);
        vd.field_script_value_rooted("monthly_county_control_change_at_war_add", Scopes::None);
        vd.field_script_value_rooted("monthly_county_control_change_at_war_mult", Scopes::None);
        vd.field_script_value_rooted("supply_limit", Scopes::None);
        vd.field_script_value_rooted("supply_limit_mult", Scopes::None);
        vd.field_script_value_rooted("tax_mult", Scopes::None);
        vd.field_script_value_rooted("supply_limit_mult", Scopes::None);
        vd.field_script_value_rooted("travel_danger", Scopes::None);
    }

    if kinds.intersects(ModifKinds::Culture) {
        vd.field_script_value_rooted("cultural_acceptance_gain_mult", Scopes::None);
        vd.field_script_value_rooted("culture_tradition_max_add", Scopes::None);
        vd.field_script_value_rooted("mercenary_count_mult", Scopes::None);
    }

    if kinds.intersects(ModifKinds::Province) {
        vd.field_script_value_rooted("defender_winter_advantage", Scopes::None);
        vd.field_script_value_rooted("hard_casualty_winter", Scopes::None);
        vd.field_script_value_rooted("supply_loss_winter", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_damage_add", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_damage_mult", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_pursuit_add", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_pursuit_mult", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_screen_add", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_screen_mult", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_siege_value_add", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_siege_value_mult", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_toughness_add", Scopes::None);
        vd.field_script_value_rooted("stationed_maa_toughness_mult", Scopes::None);
    }

    if kinds.intersects(ModifKinds::Scheme) {
        vd.field_script_value_rooted("scheme_power", Scopes::None);
        vd.field_script_value_rooted("scheme_resistance", Scopes::None);
        vd.field_script_value_rooted("scheme_secrecy", Scopes::None);
        vd.field_script_value_rooted("scheme_success_chance", Scopes::None);
    }

    if kinds.intersects(ModifKinds::TravelPlan) {
        vd.field_script_value_rooted("travel_safety", Scopes::None);
        vd.field_script_value_rooted("travel_safety_mult", Scopes::None);
        vd.field_script_value_rooted("travel_speed", Scopes::None);
        vd.field_script_value_rooted("travel_speed_mult", Scopes::None);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
                continue 'outer;
            }
        }

        if let Some(x) = token.as_str().strip_suffix("_xp_gain_mult") {
            if let Some(lifestyle) = x.strip_prefix("monthly_") {
                data.verify_exists_implied(Item::Lifestyle, lifestyle, token);
                kinds.require(ModifKinds::Character, token);
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
                continue;
            }
        }

        for trait_track_sfx in &["_xp_degradation_mult", "_xp_gain_mult", "_xp_loss_mult"] {
            if let Some(trait_track) = token.as_str().strip_suffix(trait_track_sfx) {
                data.verify_exists_implied(Item::TraitTrack, trait_track, token);
                kinds.require(ModifKinds::Character, token);
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
                continue 'outer;
            }
        }
        if let Some(x) = token.as_str().strip_suffix("_schemes_add") {
            if let Some(scheme) = x.strip_prefix("max_") {
                data.verify_exists_implied(Item::Scheme, scheme, token);
                kinds.require(ModifKinds::Character, token);
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
                continue;
            }
        }

        if let Some(x) = token.as_str().strip_prefix("scheme_power_against_") {
            if let Some(relation) = x.strip_suffix("_add") {
                data.verify_exists_implied(Item::Relation, relation, token);
                kinds.require(ModifKinds::Character, token);
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
                continue;
            }
            if let Some(relation) = x.strip_suffix("_mult") {
                data.verify_exists_implied(Item::Relation, relation, token);
                kinds.require(ModifKinds::Character, token);
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
                continue 'outer;
            }
        }

        for sfx in &["_opinion_same_faith", "_vassal_opinion"] {
            if let Some(government) = token.as_str().strip_suffix(sfx) {
                data.verify_exists_implied(Item::GovernmentType, government, token);
                kinds.require(ModifKinds::Character, token);
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
                let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
                ScriptValue::validate_bv(bv, data, &mut sc);
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
            let mut sc = ScopeContext::new_root(Scopes::None, token.clone());
            ScriptValue::validate_bv(bv, data, &mut sc);
            continue;
        }

        let msg = format!("unknown modifier `{token}`");
        warn(token, ErrorKey::Validation, &msg);
    }
}
