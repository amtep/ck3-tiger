#![allow(non_upper_case_globals)]

use crate::errorkey::ErrorKey;
use crate::errors::{error_info, warn_info};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::ModifKinds;
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

    // Vassal stance opinions
    for sfx in &[
        "_different_culture_opinion",
        "_different_faith_opinion",
        "_same_culture_opinion",
        "_same_faith_opinion",
    ] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            return modif_check(
                name,
                s,
                Item::VassalStance,
                ModifKinds::Character,
                data,
                warn,
            );
        }
    }

    // government type opinions
    for sfx in &["_vassal_opinion", "_opinion_same_faith"] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            return modif_check(
                name,
                s,
                Item::GovernmentType,
                ModifKinds::Character,
                data,
                warn,
            );
        }
    }

    // other opinions
    if let Some(s) = name.as_str().strip_suffix("_opinion") {
        if warn
            && !data.item_exists(Item::Culture, s)
            && !data.item_exists(Item::Faith, s)
            && !data.item_exists(Item::Religion, s)
            && !data.item_exists(Item::ReligionFamily, s)
            && !data.item_exists(Item::GovernmentType, s)
            && !data.item_exists(Item::VassalStance, s)
        {
            let msg = format!("could not find any {s}");
            let info = "Could be a culture, faith, religion, religion family, government type, or vassal stance";
            warn_info(name, ErrorKey::MissingItem, &msg, info);
        }
        return Some(ModifKinds::Character);
    }

    // levy and tax contributions
    for sfx in &[
        "_levy_contribution_add",
        "_levy_contribution_mult",
        "_tax_contribution_add",
        "_tax_contribution_mult",
    ] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            if warn
                && !data.item_exists(Item::GovernmentType, s)
                && !data.item_exists(Item::VassalStance, s)
            {
                let msg = format!("could not find any {s}");
                let info = "Could be a government type or vassal stance";
                warn_info(name, ErrorKey::MissingItem, &msg, info);
            }
            return Some(ModifKinds::Character);
        }
    }

    // men-at-arms types
    for sfx in &[
        "_damage_add",
        "_damage_mult",
        "_pursuit_add",
        "_pursuit_mult",
        "_screen_add",
        "_screen_mult",
        "_siege_value_add",
        "_siege_value_mult",
        "_toughness_add",
        "_toughness_mult",
    ] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            if let Some(s) = s.strip_prefix("stationed_") {
                return modif_check(
                    name,
                    s,
                    Item::MenAtArmsBase,
                    ModifKinds::Province,
                    data,
                    warn,
                );
            }
            return modif_check(
                name,
                s,
                Item::MenAtArmsBase,
                ModifKinds::Character,
                data,
                warn,
            );
        }
    }

    // men-at-arms types, non-stationed
    for sfx in &[
        "_maintenance_mult",
        "_max_size_add",
        "_max_size_mult",
        "_recruitment_cost_mult",
    ] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            return modif_check(
                name,
                s,
                Item::MenAtArmsBase,
                ModifKinds::Character,
                data,
                warn,
            );
        }
    }

    // scheme types
    for sfx in &[
        "_scheme_power_add",
        "_scheme_power_mult",
        "_scheme_resistance_add",
        "_scheme_resistance_mult",
    ] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            return modif_check(name, s, Item::Scheme, ModifKinds::Character, data, warn);
        }
    }

    // terrain
    for sfx in &[
        "_advantage",
        "_attrition_mult",
        "_cancel_negative_supply",
        "_max_combat_roll",
        "_min_combat_roll",
    ] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            return modif_check(name, s, Item::Terrain, ModifKinds::Character, data, warn);
        }
    }

    // monthly_$LIFESTYLE$_xp_gain_mult
    if let Some(s) = name.as_str().strip_prefix("monthly_") {
        if let Some(s) = s.strip_suffix("_xp_gain_mult") {
            return modif_check(name, s, Item::Lifestyle, ModifKinds::Character, data, warn);
        }
    }

    // The names of individual tracks in a multi-track trait start with `trait_track_` before the track name.
    // It's also possible to use the names of traits that have one or more tracks directly, without the trait_track_.
    // Presumably it applies to all of a trait's tracks in that case.
    for sfx in &["_xp_degradation_mult", "_xp_gain_mult", "_xp_loss_mult"] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            if let Some(s) = s.strip_prefix("trait_track_") {
                return modif_check(name, s, Item::TraitTrack, ModifKinds::Character, data, warn);
            }
            if warn {
                data.verify_exists_implied(Item::Trait, s, name);
                if !data.traits.has_track(s) {
                    let msg = format!("trait {s} does not have an xp track");
                    let info = format!("so the modifier {name} does not exist");
                    error_info(name, ErrorKey::Validation, &msg, &info);
                }
            }
            return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
        }
    }

    // max_$SCHEME_TYPE$_schemes_add
    if let Some(s) = name.as_str().strip_prefix("max_") {
        if let Some(s) = s.strip_suffix("_schemes_add") {
            return modif_check(name, s, Item::Scheme, ModifKinds::Character, data, warn);
        }
    }

    // scheme power against scripted relation
    if let Some(s) = name.as_str().strip_prefix("scheme_power_against_") {
        for sfx in &["_add", "_mult"] {
            if let Some(s) = s.strip_suffix(sfx) {
                return modif_check(name, s, Item::Relation, ModifKinds::Character, data, warn);
            }
        }
    }

    // geographical region or terrain
    for sfx in &["_development_growth", "_development_growth_factor"] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            if data.item_exists(Item::Region, s) {
                if warn && !data.item_has_property(Item::Region, s, "generates_modifiers") {
                    let msg = format!("region {s} does not have `generates_modifiers = yes`");
                    let info = format!("so the modifier {name} does not exist");
                    error_info(name, ErrorKey::Validation, &msg, &info);
                }
            } else if warn && !data.item_exists(Item::Terrain, s) {
                let msg = format!("could not find any {s}");
                let info = "Could be a geographical region or terrain";
                warn_info(name, ErrorKey::MissingItem, &msg, info);
            }
            return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
        }
    }

    // holding type
    for sfx in &[
        "_build_gold_cost",
        "_build_piety_cost",
        "_build_prestige_cost",
        "_build_speed",
    ] {
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            if data.item_exists(Item::Holding, s) {
                return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
            }
            if let Some(s) = s.strip_suffix("_holding") {
                if data.item_exists(Item::Holding, s) {
                    return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
                }
            }
            if warn {
                data.verify_exists_implied(Item::Holding, s, name);
            }
            return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
        }
    }

    // terrain type
    for sfx in &[
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
        if let Some(s) = name.as_str().strip_suffix(sfx) {
            return modif_check(
                name,
                s,
                Item::Terrain,
                ModifKinds::Character | ModifKinds::Province | ModifKinds::County,
                data,
                warn,
            );
        }
    }

    None
}

fn modif_check(
    name: &Token,
    s: &str,
    itype: Item,
    mk: ModifKinds,
    data: &Everything,
    warn: bool,
) -> Option<ModifKinds> {
    if warn {
        data.verify_exists_implied(itype, s, name);
    }
    Some(mk)
}

// Redeclare the `ModifKinds` enums as bare numbers, so that we can to | on them in const tables.
const Character: u8 = 0x01;
const Province: u8 = 0x02;
const County: u8 = 0x04;
const Terrain: u8 = 0x08;
const Culture: u8 = 0x10;
const Scheme: u8 = 0x20;
const TravelPlan: u8 = 0x40;

/// LAST UPDATED VERSION 1.9.2
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, u8)] = &[
    ("accolade_glory_gain_mult", Character),
    ("active_accolades", Character),
    ("additional_fort_level", Character | Province | County),
    ("advantage", Character),
    ("advantage_against_coreligionists", Character),
    ("ai_amenity_spending", Character),
    ("ai_amenity_target_baseline", Character),
    ("ai_boldness", Character),
    ("ai_compassion", Character),
    ("ai_energy", Character),
    ("ai_greed", Character),
    ("ai_honor", Character),
    ("ai_rationality", Character),
    ("ai_sociability", Character),
    ("ai_vengefulness", Character),
    ("ai_war_chance", Character),
    ("ai_war_cooldown", Character),
    ("ai_zeal", Character),
    ("army_damage_mult", Character),
    ("army_maintenance_mult", Character),
    ("army_pursuit_mult", Character),
    ("army_screen_mult", Character),
    ("army_siege_value_mult", Character),
    ("army_toughness_mult", Character),
    (
        "artifact_decay_reduction_mult",
        Character | Province | County,
    ),
    ("attacker_advantage", Character),
    ("attraction_opinion", Character),
    ("build_gold_cost", Character | Province | County),
    ("building_slot_add", Character | Province | County),
    ("build_piety_cost", Character | Province | County),
    ("build_prestige_cost", Character | Province | County),
    ("build_speed", Character | Province | County),
    (
        "character_capital_county_monthly_development_growth_add",
        Character,
    ),
    ("character_travel_safety", Character),
    ("character_travel_safety_mult", Character),
    ("character_travel_speed", Character),
    ("character_travel_speed_mult", Character),
    ("child_except_player_heir_opinion", Character),
    ("child_opinion", Character),
    ("clergy_opinion", Character),
    ("close_relative_opinion", Character),
    ("coastal_advantage", Character),
    ("controlled_province_advantage", Character),
    ("councillor_opinion", Character),
    ("counter_efficiency", Character | Terrain),
    ("counter_resistance", Character | Terrain),
    ("county_opinion_add", Character | County),
    ("county_opinion_add_even_if_baron", Character),
    ("court_grandeur_baseline_add", Character),
    ("courtier_and_guest_opinion", Character),
    ("courtier_opinion", Character),
    ("cowed_vassal_levy_contribution_add", Character),
    ("cowed_vassal_levy_contribution_mult", Character),
    ("cowed_vassal_tax_contribution_add", Character),
    ("cowed_vassal_tax_contribution_mult", Character),
    ("cultural_acceptance_gain_mult", Culture),
    ("cultural_head_acceptance_gain_mult", Character),
    ("cultural_head_fascination_add", Character),
    ("cultural_head_fascination_mult", Character),
    ("culture_tradition_max_add", Culture),
    ("defender_advantage", Character),
    ("defender_holding_advantage", Character | Province | County),
    ("defender_winter_advantage", Province),
    ("development_growth", Character | Province | County),
    ("development_growth_factor", Character | Province | County),
    ("different_culture_opinion", Character),
    ("different_faith_county_opinion_mult", Character),
    (
        "different_faith_county_opinion_mult_even_if_baron",
        Character,
    ),
    ("different_faith_liege_opinion", Character),
    ("different_faith_opinion", Character),
    ("diplomacy", Character),
    ("diplomacy_per_piety_level", Character),
    ("diplomacy_per_prestige_level", Character),
    ("diplomacy_per_stress_level", Character),
    ("diplomacy_scheme_power", Character),
    ("diplomacy_scheme_resistance", Character),
    ("diplomatic_range_mult", Character),
    ("direct_vassal_opinion", Character),
    ("domain_limit", Character),
    ("domain_tax_different_faith_mult", Character),
    ("domain_tax_different_faith_mult_even_if_baron", Character),
    ("domain_tax_mult", Character),
    ("domain_tax_mult_even_if_baron", Character),
    ("domain_tax_same_faith_mult", Character),
    ("domain_tax_same_faith_mult_even_if_baron", Character),
    ("dread_baseline_add", Character),
    ("dread_decay_add", Character),
    ("dread_decay_mult", Character),
    ("dread_gain_mult", Character),
    ("dread_loss_mult", Character),
    ("dread_per_tyranny_add", Character),
    ("dread_per_tyranny_mult", Character),
    ("dynasty_house_opinion", Character),
    ("dynasty_opinion", Character),
    ("eligible_child_except_player_heir_opinion", Character),
    ("eligible_child_opinion", Character),
    ("embarkation_cost_mult", Character),
    ("enemy_hard_casualty_modifier", Character | Terrain),
    ("enemy_hostile_scheme_success_chance_add", Character),
    ("enemy_personal_scheme_success_chance_add", Character),
    ("enemy_terrain_advantage", Character),
    ("faith_conversion_piety_cost_add", Character),
    ("faith_conversion_piety_cost_mult", Character),
    ("faith_creation_piety_cost_add", Character),
    ("faith_creation_piety_cost_mult", Character),
    ("fellow_vassal_opinion", Character),
    ("fertility", Character),
    ("fort_level", Character | Province | County),
    ("garrison_size", Character | Province | County),
    ("general_opinion", Character),
    ("genetic_trait_strengthen_chance", Character),
    ("guest_opinion", Character),
    ("happy_powerful_vassal_levy_contribution_add", Character),
    ("happy_powerful_vassal_levy_contribution_mult", Character),
    ("happy_powerful_vassal_tax_contribution_add", Character),
    ("happy_powerful_vassal_tax_contribution_mult", Character),
    ("hard_casualty_modifier", Character | Terrain),
    ("hard_casualty_winter", Province),
    ("health", Character),
    ("holding_build_gold_cost", Character | Province | County),
    ("holding_build_piety_cost", Character | Province | County),
    ("holding_build_prestige_cost", Character | Province | County),
    ("holding_build_speed", Character | Province | County),
    ("holy_order_hire_cost_add", Character),
    ("holy_order_hire_cost_mult", Character),
    ("hostile_county_attrition", Character),
    ("hostile_county_attrition_raiding", Character),
    ("hostile_raid_time", Character | Province | County),
    ("hostile_scheme_power_add", Character),
    ("hostile_scheme_power_mult", Character),
    ("hostile_scheme_resistance_add", Character),
    ("hostile_scheme_resistance_mult", Character),
    ("ignore_different_faith_opinion", Character),
    ("ignore_negative_culture_opinion", Character),
    ("ignore_negative_opinion_of_culture", Character),
    ("ignore_opinion_of_different_faith", Character),
    ("inbreeding_chance", Character),
    ("independent_primary_defender_advantage_add", Character),
    ("independent_ruler_opinion", Character),
    ("intimidated_vassal_levy_contribution_add", Character),
    ("intimidated_vassal_levy_contribution_mult", Character),
    ("intimidated_vassal_tax_contribution_add", Character),
    ("intimidated_vassal_tax_contribution_mult", Character),
    ("intrigue", Character),
    ("intrigue_per_piety_level", Character),
    ("intrigue_per_prestige_level", Character),
    ("intrigue_per_stress_level", Character),
    ("intrigue_scheme_power", Character),
    ("intrigue_scheme_resistance", Character),
    ("knight_effectiveness_mult", Character),
    ("knight_effectiveness_per_dread", Character),
    ("knight_effectiveness_per_tyranny", Character),
    ("knight_limit", Character),
    ("learning", Character),
    ("learning_per_piety_level", Character),
    ("learning_per_prestige_level", Character),
    ("learning_per_stress_level", Character),
    ("learning_scheme_power", Character),
    ("learning_scheme_resistance", Character),
    ("led_by_owner_extra_advantage_add", Character),
    ("levy_attack", Character),
    ("levy_maintenance", Character),
    ("levy_pursuit", Character),
    ("levy_reinforcement_rate", Character | Province | County),
    ("levy_reinforcement_rate_different_faith", Character),
    (
        "levy_reinforcement_rate_different_faith_even_if_baron",
        Character,
    ),
    ("levy_reinforcement_rate_even_if_baron", Character),
    (
        "levy_reinforcement_rate_friendly_territory",
        Character | Province | County,
    ),
    ("levy_reinforcement_rate_same_faith", Character),
    (
        "levy_reinforcement_rate_same_faith_even_if_baron",
        Character,
    ),
    ("levy_screen", Character),
    ("levy_siege", Character),
    ("levy_size", Character | Province | County),
    ("levy_toughness", Character),
    ("liege_opinion", Character),
    ("life_expectancy", Character),
    ("long_reign_bonus_mult", Character),
    ("maa_damage_add", Character),
    ("maa_damage_mult", Character),
    ("maa_pursuit_add", Character),
    ("maa_pursuit_mult", Character),
    ("maa_screen_add", Character),
    ("maa_screen_mult", Character),
    ("maa_siege_value_add", Character),
    ("maa_siege_value_mult", Character),
    ("maa_toughness_add", Character),
    ("maa_toughness_mult", Character),
    ("martial", Character),
    ("martial_per_piety_level", Character),
    ("martial_per_prestige_level", Character),
    ("martial_per_stress_level", Character),
    ("martial_scheme_power", Character),
    ("martial_scheme_resistance", Character),
    ("max_combat_roll", Character),
    ("max_hostile_schemes_add", Character),
    ("max_loot_mult", Character),
    ("max_personal_schemes_add", Character),
    ("men_at_arms_cap", Character),
    ("men_at_arms_limit", Character),
    ("men_at_arms_maintenance", Character),
    ("men_at_arms_maintenance_per_dread_mult", Character),
    ("men_at_arms_recruitment_cost", Character),
    ("mercenary_count_mult", Culture),
    ("mercenary_hire_cost_add", Character),
    ("mercenary_hire_cost_mult", Character),
    ("min_combat_roll", Character),
    (
        "monthly_county_control_change_add",
        Character | Province | County,
    ),
    ("monthly_county_control_change_add_even_if_baron", Character),
    (
        "monthly_county_control_change_at_war_add",
        Character | Province | County,
    ),
    (
        "monthly_county_control_change_at_war_mult",
        Character | Province | County,
    ),
    (
        "monthly_county_control_change_factor",
        Character | Province | County,
    ),
    (
        "monthly_county_control_change_factor_even_if_baron",
        Character,
    ),
    ("monthly_court_grandeur_change_add", Character),
    ("monthly_court_grandeur_change_mult", Character),
    ("monthly_dread", Character),
    ("monthly_dynasty_prestige", Character),
    ("monthly_dynasty_prestige_mult", Character),
    ("monthly_income", Character | Province),
    ("monthly_income_mult", Character),
    ("monthly_income_per_stress_level_add", Character),
    ("monthly_income_per_stress_level_mult", Character),
    ("monthly_lifestyle_xp_gain_mult", Character),
    ("monthly_piety", Character),
    ("monthly_piety_from_buildings_mult", Character),
    ("monthly_piety_gain_mult", Character),
    ("monthly_piety_gain_per_dread_add", Character),
    ("monthly_piety_gain_per_dread_mult", Character),
    (
        "monthly_piety_gain_per_happy_powerful_vassal_add",
        Character,
    ),
    (
        "monthly_piety_gain_per_happy_powerful_vassal_mult",
        Character,
    ),
    ("monthly_piety_gain_per_knight_add", Character),
    ("monthly_piety_gain_per_knight_mult", Character),
    ("monthly_prestige", Character),
    ("monthly_prestige_from_buildings_mult", Character),
    ("monthly_prestige_gain_mult", Character),
    ("monthly_prestige_gain_per_dread_add", Character),
    ("monthly_prestige_gain_per_dread_mult", Character),
    (
        "monthly_prestige_gain_per_happy_powerful_vassal_add",
        Character,
    ),
    (
        "monthly_prestige_gain_per_happy_powerful_vassal_mult",
        Character,
    ),
    ("monthly_prestige_gain_per_knight_add", Character),
    ("monthly_prestige_gain_per_knight_mult", Character),
    ("monthly_tyranny", Character),
    ("monthly_war_income_add", Character),
    ("monthly_war_income_mult", Character),
    ("movement_speed", Character),
    ("movement_speed_land_raiding", Character),
    ("naval_movement_speed_mult", Character),
    ("negate_diplomacy_penalty_add", Character),
    ("negate_fertility_penalty_add", Character),
    ("negate_health_penalty_add", Character),
    ("negate_intrigue_penalty_add", Character),
    ("negate_learning_penalty_add", Character),
    ("negate_martial_penalty_add", Character),
    ("negate_prowess_penalty_add", Character),
    ("negate_stewardship_penalty_add", Character),
    ("negative_inactive_inheritance_chance", Character),
    ("negative_random_genetic_chance", Character),
    ("no_disembark_penalty", Character),
    ("no_prowess_loss_from_age", Character),
    ("no_water_crossing_penalty", Character),
    ("opinion_of_different_culture", Character),
    ("opinion_of_different_faith", Character),
    ("opinion_of_different_faith_liege", Character),
    ("opinion_of_female_rulers", Character),
    ("opinion_of_liege", Character),
    ("opinion_of_male_rulers", Character),
    ("opinion_of_parents", Character),
    ("opinion_of_same_culture", Character),
    ("opinion_of_same_faith", Character),
    ("opinion_of_vassal", Character),
    ("owned_hostile_scheme_success_chance_add", Character),
    ("owned_personal_scheme_success_chance_add", Character),
    ("owned_scheme_secrecy_add", Character),
    ("personal_scheme_power_add", Character),
    ("personal_scheme_power_mult", Character),
    ("personal_scheme_resistance_add", Character),
    ("personal_scheme_resistance_mult", Character),
    ("piety_level_impact_mult", Character),
    ("player_heir_opinion", Character),
    ("positive_inactive_inheritance_chance", Character),
    ("positive_random_genetic_chance", Character),
    ("powerful_vassal_opinion", Character),
    ("prestige_level_impact_mult", Character),
    ("prisoner_opinion", Character),
    ("prowess", Character),
    ("prowess_no_portrait", Character),
    ("prowess_per_piety_level", Character),
    ("prowess_per_prestige_level", Character),
    ("prowess_per_stress_level", Character),
    ("prowess_scheme_power", Character),
    ("prowess_scheme_resistance", Character),
    ("pursue_efficiency", Character | Terrain),
    ("raid_speed", Character),
    ("random_advantage", Character),
    ("realm_priest_opinion", Character),
    ("religious_head_opinion", Character),
    ("religious_vassal_opinion", Character),
    ("retreat_losses", Character | Terrain),
    ("revolting_siege_morale_loss_add", Character),
    ("revolting_siege_morale_loss_mult", Character),
    ("same_culture_holy_order_hire_cost_add", Character),
    ("same_culture_holy_order_hire_cost_mult", Character),
    ("same_culture_mercenary_hire_cost_add", Character),
    ("same_culture_mercenary_hire_cost_mult", Character),
    ("same_culture_opinion", Character),
    ("same_faith_opinion", Character),
    ("same_heritage_county_advantage_add", Character),
    ("scheme_discovery_chance_mult", Character),
    ("scheme_power", Scheme),
    ("scheme_resistance", Scheme),
    ("scheme_secrecy", Scheme),
    ("scheme_success_chance", Scheme),
    ("short_reign_duration_mult", Character),
    ("siege_morale_loss", Character),
    ("siege_phase_time", Character),
    ("spouse_opinion", Character),
    ("stationed_maa_damage_add", Province),
    ("stationed_maa_damage_mult", Province),
    ("stationed_maa_pursuit_add", Province),
    ("stationed_maa_pursuit_mult", Province),
    ("stationed_maa_screen_add", Province),
    ("stationed_maa_screen_mult", Province),
    ("stationed_maa_siege_value_add", Province),
    ("stationed_maa_siege_value_mult", Province),
    ("stationed_maa_toughness_add", Province),
    ("stationed_maa_toughness_mult", Province),
    ("stewardship", Character),
    ("stewardship_per_piety_level", Character),
    ("stewardship_per_prestige_level", Character),
    ("stewardship_per_stress_level", Character),
    ("stewardship_scheme_power", Character),
    ("stewardship_scheme_resistance", Character),
    ("stress_gain_mult", Character),
    ("stress_loss_mult", Character),
    ("stress_loss_per_piety_level", Character),
    ("stress_loss_per_prestige_level", Character),
    ("strife_opinion_gain_mult", Character),
    ("strife_opinion_loss_mult", Character),
    ("supply_capacity_add", Character),
    ("supply_capacity_mult", Character),
    ("supply_duration", Character),
    ("supply_limit", Character | Province | County),
    ("supply_limit_mult", Character | Province | County),
    ("supply_loss_winter", Province),
    ("tax_mult", Character | Province | County),
    ("title_creation_cost", Character),
    ("title_creation_cost_mult", Character),
    ("tolerance_advantage_mod", Character),
    ("travel_companion_opinion", Character),
    ("travel_danger", Character | Province | County),
    ("travel_safety_mult", TravelPlan),
    ("travel_safety", TravelPlan),
    ("travel_speed_mult", TravelPlan),
    ("travel_speed", TravelPlan),
    ("twin_opinion", Character),
    ("tyranny_gain_mult", Character),
    ("tyranny_loss_mult", Character),
    ("uncontrolled_province_advantage", Character),
    ("vassal_levy_contribution_add", Character),
    ("vassal_levy_contribution_mult", Character),
    ("vassal_limit", Character),
    ("vassal_opinion", Character),
    ("vassal_tax_contribution_add", Character),
    ("vassal_tax_contribution_mult", Character),
    ("vassal_tax_mult", Character),
    ("winter_advantage", Character),
    ("winter_movement_speed", Character),
    ("years_of_fertility", Character),
];
