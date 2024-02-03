#![allow(non_upper_case_globals)]

use std::borrow::Cow;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::everything::Everything;
use crate::item::Item;
use crate::modif::ModifKinds;
use crate::report::{report, ErrorKey, Severity};
use crate::token::Token;

/// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
/// Returns None otherwise.
// LAST UPDATED CK3 VERSION 1.11.3
pub fn lookup_modif(name: &Token, data: &Everything, warn: Option<Severity>) -> Option<ModifKinds> {
    let name_lc = name.as_str().to_lowercase();

    if let result @ Some(_) = MODIF_MAP.get(&*name_lc).copied() {
        return result;
    }

    // Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    // Vassal stance opinions
    for sfx in [
        "_ai_boldness",
        "_ai_compassion",
        "_ai_energy",
        "_ai_greed",
        "_ai_honor",
        "_ai_rationality",
        "_ai_sociability",
        "_ai_vengefulness",
        "_ai_zeal",
        "_different_culture_opinion",
        "_different_faith_opinion",
        "_same_culture_opinion",
        "_same_faith_opinion",
    ] {
        if let Some(s) = name_lc.strip_suffix(sfx) {
            return modif_check(name, s, Item::VassalStance, ModifKinds::Character, data, warn);
        }
    }

    // government type opinions
    for sfx in &["_vassal_opinion", "_opinion_same_faith"] {
        if let Some(s) = name_lc.strip_suffix(sfx) {
            return modif_check(name, s, Item::GovernmentType, ModifKinds::Character, data, warn);
        }
    }

    // other opinions
    if let Some(s) = name_lc.strip_suffix("_opinion") {
        if let Some(sev) = warn {
            if !data.item_exists(Item::Culture, s)
                && !data.item_exists(Item::Faith, s)
                && !data.item_exists(Item::Religion, s)
                && !data.item_exists(Item::ReligionFamily, s)
                && !data.item_exists(Item::GovernmentType, s)
                && !data.item_exists(Item::VassalStance, s)
            {
                let msg = format!("could not find any {s}");
                let info = "Could be a culture, faith, religion, religion family, government type, or vassal stance";
                report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
            }
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
        if let Some(s) = name_lc.strip_suffix(sfx) {
            if let Some(sev) = warn {
                if !data.item_exists(Item::GovernmentType, s)
                    && !data.item_exists(Item::VassalStance, s)
                {
                    let msg = format!("could not find any {s}");
                    let info = "Could be a government type or vassal stance";
                    report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
                }
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
        if let Some(s) = name_lc.strip_suffix(sfx) {
            if let Some(s) = s.strip_prefix("stationed_") {
                return modif_check(name, s, Item::MenAtArmsBase, ModifKinds::Province, data, warn);
            }
            return modif_check(name, s, Item::MenAtArmsBase, ModifKinds::Character, data, warn);
        }
    }

    // men-at-arms types, non-stationed
    for sfx in &["_maintenance_mult", "_max_size_add", "_max_size_mult", "_recruitment_cost_mult"] {
        if let Some(s) = name_lc.strip_suffix(sfx) {
            return modif_check(name, s, Item::MenAtArmsBase, ModifKinds::Character, data, warn);
        }
    }

    // scheme types
    for sfx in &[
        "_scheme_power_add",
        "_scheme_power_mult",
        "_scheme_resistance_add",
        "_scheme_resistance_mult",
    ] {
        if let Some(s) = name_lc.strip_suffix(sfx) {
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
        if let Some(s) = name_lc.strip_suffix(sfx) {
            return modif_check(name, s, Item::Terrain, ModifKinds::Character, data, warn);
        }
    }

    // monthly_$LIFESTYLE$_xp_gain_mult
    if let Some(s) = name_lc.strip_prefix("monthly_") {
        if let Some(s) = s.strip_suffix("_xp_gain_mult") {
            return modif_check(name, s, Item::Lifestyle, ModifKinds::Character, data, warn);
        }
    }

    // The names of individual tracks in a multi-track trait start with `trait_track_` before the track name.
    // It's also possible to use the names of traits that have one or more tracks directly, without the trait_track_.
    // Presumably it applies to all of a trait's tracks in that case.
    // $LIFESTYLE$_xp_gain_mult needs to be handled here too.
    for sfx in &["_xp_degradation_mult", "_xp_gain_mult", "_xp_loss_mult"] {
        if let Some(s) = name_lc.strip_suffix(sfx) {
            if let Some(s) = s.strip_prefix("trait_track_") {
                return modif_check(name, s, Item::TraitTrack, ModifKinds::Character, data, warn);
            }
            // It can be a lifestyle or a trait.
            if let Some(sev) = warn {
                if !data.item_exists(Item::Lifestyle, s) && !data.item_exists(Item::Trait, s) {
                    let msg = "`{s}` was not found as a trait or lifestyle";
                    let info = format!("so the modifier {name} does not exist");
                    report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
                } else if data.item_exists(Item::Trait, s) && !data.traits.has_track(s) {
                    let msg = format!("trait {s} does not have an xp track");
                    let info = format!("so the modifier {name} does not exist");
                    report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
                }
            }
            return Some(ModifKinds::Character);
        }
    }

    // max_$SCHEME_TYPE$_schemes_add
    if let Some(s) = name_lc.strip_prefix("max_") {
        if let Some(s) = s.strip_suffix("_schemes_add") {
            return modif_check(name, s, Item::Scheme, ModifKinds::Character, data, warn);
        }
    }

    // scheme power against scripted relation
    if let Some(s) = name_lc.strip_prefix("scheme_power_against_") {
        for sfx in &["_add", "_mult"] {
            if let Some(s) = s.strip_suffix(sfx) {
                return modif_check(name, s, Item::Relation, ModifKinds::Character, data, warn);
            }
        }
    }

    // $TAX_SLOT_TYPE$_add
    if let Some(s) = name_lc.strip_suffix("_add") {
        return modif_check(name, s, Item::TaxSlotType, ModifKinds::Character, data, warn);
    }

    // geographical region or terrain
    for sfx in &["_development_growth", "_development_growth_factor"] {
        if let Some(s) = name_lc.strip_suffix(sfx) {
            if data.item_exists(Item::Region, s) {
                if let Some(sev) = warn {
                    if !data.item_has_property(Item::Region, s, "generates_modifiers") {
                        let msg = format!("region {s} does not have `generates_modifiers = yes`");
                        let info = format!("so the modifier {name} does not exist");
                        report(ErrorKey::MissingItem, sev)
                            .strong()
                            .msg(msg)
                            .info(info)
                            .loc(name)
                            .push();
                    }
                }
            } else if let Some(sev) = warn {
                if !data.item_exists(Item::Terrain, s) {
                    let msg = format!("could not find any {s}");
                    let info = "Could be a geographical region or terrain";
                    report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
                }
            }
            return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
        }
    }

    // holding type
    for sfx in &["_build_gold_cost", "_build_piety_cost", "_build_prestige_cost", "_build_speed"] {
        if let Some(s) = name_lc.strip_suffix(sfx) {
            if data.item_exists(Item::HoldingType, s) {
                return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
            }
            if let Some(s) = s.strip_suffix("_holding") {
                if data.item_exists(Item::HoldingType, s) {
                    return Some(ModifKinds::Character | ModifKinds::Province | ModifKinds::County);
                }
            }
            if let Some(sev) = warn {
                let msg = format!("could not find holding type {s}");
                let info = format!("so the modifier {name} does not exist");
                report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
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
        if let Some(s) = name_lc.strip_suffix(sfx) {
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

#[allow(clippy::unnecessary_wraps)]
fn modif_check(
    name: &Token,
    s: &str,
    itype: Item,
    mk: ModifKinds,
    data: &Everything,
    warn: Option<Severity>,
) -> Option<ModifKinds> {
    if let Some(sev) = warn {
        if !data.item_exists(itype, s) {
            let msg = format!("could not find {itype} {s}");
            let info = format!("so the modifier {name} does not exist");
            report(ErrorKey::MissingItem, sev).strong().msg(msg).info(info).loc(name).push();
        }
    }
    Some(mk)
}

/// Return the modifier localization. If the modifier is static,
/// i.e. a code defined modifier, it begins with `MOD_` and may have a different body in special cases.
/// If the modifier is dynamic, i.e. generated from script defined items, then its name is returned unchanged.
pub fn modif_loc(name: &Token) -> Cow<'static, str> {
    let name_lc = name.as_str().to_lowercase();

    if let Some(body) = SPECIAL_MODIF_LOC_MAP.get(&*name_lc).copied() {
        Cow::Borrowed(body)
    } else if MODIF_MAP.contains_key(&*name_lc) {
        Cow::Owned(format!("MOD_{}", name_lc.to_uppercase()))
    } else {
        Cow::Owned(name_lc)
    }
}

static MODIF_MAP: Lazy<FnvHashMap<&'static str, ModifKinds>> = Lazy::new(|| {
    let mut hash = FnvHashMap::default();
    for (s, kind) in MODIF_TABLE.iter().copied() {
        hash.insert(s, kind);
    }
    hash
});

/// LAST UPDATED CK3 VERSION 1.11.3
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, ModifKinds)] = &[
    ("accolade_glory_gain_mult", ModifKinds::Character),
    ("active_accolades", ModifKinds::Character),
    (
        "additional_fort_level",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("advantage", ModifKinds::Character),
    ("advantage_against_coreligionists", ModifKinds::Character),
    ("ai_amenity_spending", ModifKinds::Character),
    ("ai_amenity_target_baseline", ModifKinds::Character),
    ("ai_boldness", ModifKinds::Character),
    ("ai_compassion", ModifKinds::Character),
    ("ai_energy", ModifKinds::Character),
    ("ai_greed", ModifKinds::Character),
    ("ai_honor", ModifKinds::Character),
    ("ai_rationality", ModifKinds::Character),
    ("ai_sociability", ModifKinds::Character),
    ("ai_vengefulness", ModifKinds::Character),
    ("ai_war_chance", ModifKinds::Character),
    ("ai_war_cooldown", ModifKinds::Character),
    ("ai_zeal", ModifKinds::Character),
    ("army_damage_mult", ModifKinds::Character),
    ("army_maintenance_mult", ModifKinds::Character),
    ("army_pursuit_mult", ModifKinds::Character),
    ("army_screen_mult", ModifKinds::Character),
    ("army_siege_value_mult", ModifKinds::Character),
    ("army_toughness_mult", ModifKinds::Character),
    (
        "artifact_decay_reduction_mult",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("attacker_advantage", ModifKinds::Character),
    ("attraction_opinion", ModifKinds::Character),
    (
        "build_gold_cost",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "building_slot_add",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "build_piety_cost",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "build_prestige_cost",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("build_speed", ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County)),
    ("character_capital_county_monthly_development_growth_add", ModifKinds::Character),
    ("character_travel_safety", ModifKinds::Character),
    ("character_travel_safety_mult", ModifKinds::Character),
    ("character_travel_speed", ModifKinds::Character),
    ("character_travel_speed_mult", ModifKinds::Character),
    ("child_except_player_heir_opinion", ModifKinds::Character),
    ("child_opinion", ModifKinds::Character),
    ("clergy_opinion", ModifKinds::Character),
    ("close_relative_opinion", ModifKinds::Character),
    ("coastal_advantage", ModifKinds::Character),
    ("controlled_province_advantage", ModifKinds::Character),
    ("councillor_opinion", ModifKinds::Character),
    ("counter_efficiency", ModifKinds::Character.union(ModifKinds::Terrain)),
    ("counter_resistance", ModifKinds::Character.union(ModifKinds::Terrain)),
    ("county_opinion_add", ModifKinds::Character.union(ModifKinds::County)),
    ("county_opinion_add_even_if_baron", ModifKinds::Character),
    ("court_grandeur_baseline_add", ModifKinds::Character),
    ("courtier_and_guest_opinion", ModifKinds::Character),
    ("courtier_opinion", ModifKinds::Character),
    ("cowed_vassal_levy_contribution_add", ModifKinds::Character),
    ("cowed_vassal_levy_contribution_mult", ModifKinds::Character),
    ("cowed_vassal_tax_contribution_add", ModifKinds::Character),
    ("cowed_vassal_tax_contribution_mult", ModifKinds::Character),
    ("cultural_acceptance_gain_mult", ModifKinds::Culture),
    ("cultural_head_acceptance_gain_mult", ModifKinds::Character),
    ("cultural_head_fascination_add", ModifKinds::Character),
    ("cultural_head_fascination_mult", ModifKinds::Character),
    ("culture_tradition_max_add", ModifKinds::Culture),
    ("defender_advantage", ModifKinds::Character),
    (
        "defender_holding_advantage",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("defender_winter_advantage", ModifKinds::Province),
    (
        "development_growth",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "development_growth_factor",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("different_culture_opinion", ModifKinds::Character),
    ("different_faith_county_opinion_mult", ModifKinds::Character),
    ("different_faith_county_opinion_mult_even_if_baron", ModifKinds::Character),
    ("different_faith_liege_opinion", ModifKinds::Character),
    ("different_faith_opinion", ModifKinds::Character),
    ("diplomacy", ModifKinds::Character),
    ("diplomacy_per_piety_level", ModifKinds::Character),
    ("diplomacy_per_prestige_level", ModifKinds::Character),
    ("diplomacy_per_stress_level", ModifKinds::Character),
    ("diplomacy_scheme_power", ModifKinds::Character),
    ("diplomacy_scheme_resistance", ModifKinds::Character),
    ("diplomatic_range_mult", ModifKinds::Character),
    ("direct_vassal_opinion", ModifKinds::Character),
    ("domain_limit", ModifKinds::Character),
    ("domain_tax_different_faith_mult", ModifKinds::Character),
    ("domain_tax_different_faith_mult_even_if_baron", ModifKinds::Character),
    ("domain_tax_mult", ModifKinds::Character),
    ("domain_tax_mult_even_if_baron", ModifKinds::Character),
    ("domain_tax_same_faith_mult", ModifKinds::Character),
    ("domain_tax_same_faith_mult_even_if_baron", ModifKinds::Character),
    ("dread_baseline_add", ModifKinds::Character),
    ("dread_decay_add", ModifKinds::Character),
    ("dread_decay_mult", ModifKinds::Character),
    ("dread_gain_mult", ModifKinds::Character),
    ("dread_loss_mult", ModifKinds::Character),
    ("dread_per_tyranny_add", ModifKinds::Character),
    ("dread_per_tyranny_mult", ModifKinds::Character),
    ("dynasty_house_opinion", ModifKinds::Character),
    ("dynasty_opinion", ModifKinds::Character),
    ("eligible_child_except_player_heir_opinion", ModifKinds::Character),
    ("eligible_child_opinion", ModifKinds::Character),
    ("embarkation_cost_mult", ModifKinds::Character),
    ("enemy_hard_casualty_modifier", ModifKinds::Character.union(ModifKinds::Terrain)),
    ("enemy_hostile_scheme_success_chance_add", ModifKinds::Character),
    ("enemy_personal_scheme_success_chance_add", ModifKinds::Character),
    ("enemy_terrain_advantage", ModifKinds::Character),
    ("faith_conversion_piety_cost_add", ModifKinds::Character),
    ("faith_conversion_piety_cost_mult", ModifKinds::Character),
    ("faith_creation_piety_cost_add", ModifKinds::Character),
    ("faith_creation_piety_cost_mult", ModifKinds::Character),
    ("fellow_vassal_opinion", ModifKinds::Character),
    ("fertility", ModifKinds::Character),
    ("fort_level", ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County)),
    ("garrison_size", ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County)),
    ("general_opinion", ModifKinds::Character),
    ("genetic_trait_strengthen_chance", ModifKinds::Character),
    ("guest_opinion", ModifKinds::Character),
    ("happy_powerful_vassal_levy_contribution_add", ModifKinds::Character),
    ("happy_powerful_vassal_levy_contribution_mult", ModifKinds::Character),
    ("happy_powerful_vassal_tax_contribution_add", ModifKinds::Character),
    ("happy_powerful_vassal_tax_contribution_mult", ModifKinds::Character),
    ("hard_casualty_modifier", ModifKinds::Character.union(ModifKinds::Terrain)),
    ("hard_casualty_winter", ModifKinds::Province),
    ("health", ModifKinds::Character),
    (
        "holding_build_gold_cost",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "holding_build_piety_cost",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "holding_build_prestige_cost",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "holding_build_speed",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("holy_order_hire_cost_add", ModifKinds::Character),
    ("holy_order_hire_cost_mult", ModifKinds::Character),
    ("hostage_income_mult", ModifKinds::Character),
    ("hostage_piety_mult", ModifKinds::Character),
    ("hostage_prestige_mult", ModifKinds::Character),
    ("hostage_renown_mult", ModifKinds::Character),
    ("hostile_county_attrition", ModifKinds::Character),
    ("hostile_county_attrition_raiding", ModifKinds::Character),
    (
        "hostile_raid_time",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("hostile_scheme_power_add", ModifKinds::Character),
    ("hostile_scheme_power_mult", ModifKinds::Character),
    ("hostile_scheme_resistance_add", ModifKinds::Character),
    ("hostile_scheme_resistance_mult", ModifKinds::Character),
    ("ignore_different_faith_opinion", ModifKinds::Character),
    ("ignore_negative_culture_opinion", ModifKinds::Character),
    ("ignore_negative_opinion_of_culture", ModifKinds::Character),
    ("ignore_opinion_of_different_faith", ModifKinds::Character),
    ("inbreeding_chance", ModifKinds::Character),
    ("independent_primary_defender_advantage_add", ModifKinds::Character),
    ("independent_ruler_opinion", ModifKinds::Character),
    ("intimidated_vassal_levy_contribution_add", ModifKinds::Character),
    ("intimidated_vassal_levy_contribution_mult", ModifKinds::Character),
    ("intimidated_vassal_tax_contribution_add", ModifKinds::Character),
    ("intimidated_vassal_tax_contribution_mult", ModifKinds::Character),
    ("intrigue", ModifKinds::Character),
    ("intrigue_per_piety_level", ModifKinds::Character),
    ("intrigue_per_prestige_level", ModifKinds::Character),
    ("intrigue_per_stress_level", ModifKinds::Character),
    ("intrigue_scheme_power", ModifKinds::Character),
    ("intrigue_scheme_resistance", ModifKinds::Character),
    ("knight_effectiveness_mult", ModifKinds::Character),
    ("knight_effectiveness_per_dread", ModifKinds::Character),
    ("knight_effectiveness_per_tyranny", ModifKinds::Character),
    ("knight_limit", ModifKinds::Character),
    ("learning", ModifKinds::Character),
    ("learning_per_piety_level", ModifKinds::Character),
    ("learning_per_prestige_level", ModifKinds::Character),
    ("learning_per_stress_level", ModifKinds::Character),
    ("learning_scheme_power", ModifKinds::Character),
    ("learning_scheme_resistance", ModifKinds::Character),
    ("led_by_owner_extra_advantage_add", ModifKinds::Character),
    ("levy_attack", ModifKinds::Character),
    ("levy_maintenance", ModifKinds::Character),
    ("levy_pursuit", ModifKinds::Character),
    (
        "levy_reinforcement_rate",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("levy_reinforcement_rate_different_faith", ModifKinds::Character),
    ("levy_reinforcement_rate_different_faith_even_if_baron", ModifKinds::Character),
    ("levy_reinforcement_rate_even_if_baron", ModifKinds::Character),
    (
        "levy_reinforcement_rate_friendly_territory",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("levy_reinforcement_rate_same_faith", ModifKinds::Character),
    ("levy_reinforcement_rate_same_faith_even_if_baron", ModifKinds::Character),
    ("levy_screen", ModifKinds::Character),
    ("levy_siege", ModifKinds::Character),
    ("levy_size", ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County)),
    ("levy_toughness", ModifKinds::Character),
    ("liege_opinion", ModifKinds::Character),
    ("life_expectancy", ModifKinds::Character),
    ("long_reign_bonus_mult", ModifKinds::Character),
    ("maa_damage_add", ModifKinds::Character),
    ("maa_damage_mult", ModifKinds::Character),
    ("maa_pursuit_add", ModifKinds::Character),
    ("maa_pursuit_mult", ModifKinds::Character),
    ("maa_screen_add", ModifKinds::Character),
    ("maa_screen_mult", ModifKinds::Character),
    ("maa_siege_value_add", ModifKinds::Character),
    ("maa_siege_value_mult", ModifKinds::Character),
    ("maa_toughness_add", ModifKinds::Character),
    ("maa_toughness_mult", ModifKinds::Character),
    ("martial", ModifKinds::Character),
    ("martial_per_piety_level", ModifKinds::Character),
    ("martial_per_prestige_level", ModifKinds::Character),
    ("martial_per_stress_level", ModifKinds::Character),
    ("martial_scheme_power", ModifKinds::Character),
    ("martial_scheme_resistance", ModifKinds::Character),
    ("max_combat_roll", ModifKinds::Character),
    ("max_hostile_schemes_add", ModifKinds::Character),
    ("max_loot_mult", ModifKinds::Character),
    ("max_personal_schemes_add", ModifKinds::Character),
    ("men_at_arms_cap", ModifKinds::Character),
    ("men_at_arms_limit", ModifKinds::Character),
    ("men_at_arms_maintenance", ModifKinds::Character),
    ("men_at_arms_maintenance_per_dread_mult", ModifKinds::Character),
    ("men_at_arms_recruitment_cost", ModifKinds::Character),
    ("mercenary_count_mult", ModifKinds::Culture),
    ("mercenary_hire_cost_add", ModifKinds::Character),
    ("mercenary_hire_cost_mult", ModifKinds::Character),
    ("min_combat_roll", ModifKinds::Character),
    (
        "monthly_county_control_change_add",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("monthly_county_control_change_add_even_if_baron", ModifKinds::Character),
    (
        "monthly_county_control_change_at_war_add",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "monthly_county_control_change_at_war_mult",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    (
        "monthly_county_control_change_factor",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("monthly_county_control_change_factor_even_if_baron", ModifKinds::Character),
    ("monthly_court_grandeur_change_add", ModifKinds::Character),
    ("monthly_court_grandeur_change_mult", ModifKinds::Character),
    ("monthly_dread", ModifKinds::Character),
    ("monthly_dynasty_prestige", ModifKinds::Character),
    ("monthly_dynasty_prestige_mult", ModifKinds::Character),
    ("monthly_income", ModifKinds::Character.union(ModifKinds::Province)),
    ("monthly_income_mult", ModifKinds::Character),
    ("monthly_income_per_stress_level_add", ModifKinds::Character),
    ("monthly_income_per_stress_level_mult", ModifKinds::Character),
    ("monthly_lifestyle_xp_gain_mult", ModifKinds::Character),
    ("monthly_piety", ModifKinds::Character),
    ("monthly_piety_from_buildings_mult", ModifKinds::Character),
    ("monthly_piety_gain_mult", ModifKinds::Character),
    ("monthly_piety_gain_per_dread_add", ModifKinds::Character),
    ("monthly_piety_gain_per_dread_mult", ModifKinds::Character),
    ("monthly_piety_gain_per_happy_powerful_vassal_add", ModifKinds::Character),
    ("monthly_piety_gain_per_happy_powerful_vassal_mult", ModifKinds::Character),
    ("monthly_piety_gain_per_knight_add", ModifKinds::Character),
    ("monthly_piety_gain_per_knight_mult", ModifKinds::Character),
    ("monthly_prestige", ModifKinds::Character),
    ("monthly_prestige_from_buildings_mult", ModifKinds::Character),
    ("monthly_prestige_gain_mult", ModifKinds::Character),
    ("monthly_prestige_gain_per_dread_add", ModifKinds::Character),
    ("monthly_prestige_gain_per_dread_mult", ModifKinds::Character),
    ("monthly_prestige_gain_per_happy_powerful_vassal_add", ModifKinds::Character),
    ("monthly_prestige_gain_per_happy_powerful_vassal_mult", ModifKinds::Character),
    ("monthly_prestige_gain_per_knight_add", ModifKinds::Character),
    ("monthly_prestige_gain_per_knight_mult", ModifKinds::Character),
    ("monthly_tyranny", ModifKinds::Character),
    ("monthly_war_income_add", ModifKinds::Character),
    ("monthly_war_income_mult", ModifKinds::Character),
    ("movement_speed", ModifKinds::Character),
    ("movement_speed_land_raiding", ModifKinds::Character),
    ("naval_movement_speed_mult", ModifKinds::Character),
    ("negate_diplomacy_penalty_add", ModifKinds::Character),
    ("negate_fertility_penalty_add", ModifKinds::Character),
    ("negate_health_penalty_add", ModifKinds::Character),
    ("negate_intrigue_penalty_add", ModifKinds::Character),
    ("negate_learning_penalty_add", ModifKinds::Character),
    ("negate_martial_penalty_add", ModifKinds::Character),
    ("negate_prowess_penalty_add", ModifKinds::Character),
    ("negate_stewardship_penalty_add", ModifKinds::Character),
    ("negative_inactive_inheritance_chance", ModifKinds::Character),
    ("negative_random_genetic_chance", ModifKinds::Character),
    ("no_disembark_penalty", ModifKinds::Character),
    ("no_prowess_loss_from_age", ModifKinds::Character),
    ("no_water_crossing_penalty", ModifKinds::Character),
    ("opinion_of_different_culture", ModifKinds::Character),
    ("opinion_of_different_faith", ModifKinds::Character),
    ("opinion_of_different_faith_liege", ModifKinds::Character),
    ("opinion_of_female_rulers", ModifKinds::Character),
    ("opinion_of_liege", ModifKinds::Character),
    ("opinion_of_male_rulers", ModifKinds::Character),
    ("opinion_of_parents", ModifKinds::Character),
    ("opinion_of_same_culture", ModifKinds::Character),
    ("opinion_of_same_faith", ModifKinds::Character),
    ("opinion_of_vassal", ModifKinds::Character),
    ("owned_hostile_scheme_success_chance_add", ModifKinds::Character),
    ("owned_personal_scheme_success_chance_add", ModifKinds::Character),
    ("owned_scheme_secrecy_add", ModifKinds::Character),
    ("personal_scheme_power_add", ModifKinds::Character),
    ("personal_scheme_power_mult", ModifKinds::Character),
    ("personal_scheme_resistance_add", ModifKinds::Character),
    ("personal_scheme_resistance_mult", ModifKinds::Character),
    ("piety_level_impact_mult", ModifKinds::Character),
    ("player_heir_opinion", ModifKinds::Character),
    ("positive_inactive_inheritance_chance", ModifKinds::Character),
    ("positive_random_genetic_chance", ModifKinds::Character),
    ("powerful_vassal_opinion", ModifKinds::Character),
    ("prestige_level_impact_mult", ModifKinds::Character),
    ("prisoner_opinion", ModifKinds::Character),
    ("prowess", ModifKinds::Character),
    ("prowess_no_portrait", ModifKinds::Character),
    ("prowess_per_piety_level", ModifKinds::Character),
    ("prowess_per_prestige_level", ModifKinds::Character),
    ("prowess_per_stress_level", ModifKinds::Character),
    ("prowess_scheme_power", ModifKinds::Character),
    ("prowess_scheme_resistance", ModifKinds::Character),
    ("pursue_efficiency", ModifKinds::Character.union(ModifKinds::Terrain)),
    ("raid_speed", ModifKinds::Character),
    ("random_advantage", ModifKinds::Character),
    ("realm_priest_opinion", ModifKinds::Character),
    ("religious_head_opinion", ModifKinds::Character),
    ("religious_vassal_opinion", ModifKinds::Character),
    ("retreat_losses", ModifKinds::Character.union(ModifKinds::Terrain)),
    ("revolting_siege_morale_loss_add", ModifKinds::Character),
    ("revolting_siege_morale_loss_mult", ModifKinds::Character),
    ("same_culture_holy_order_hire_cost_add", ModifKinds::Character),
    ("same_culture_holy_order_hire_cost_mult", ModifKinds::Character),
    ("same_culture_mercenary_hire_cost_add", ModifKinds::Character),
    ("same_culture_mercenary_hire_cost_mult", ModifKinds::Character),
    ("same_culture_opinion", ModifKinds::Character),
    ("same_faith_opinion", ModifKinds::Character),
    ("same_heritage_county_advantage_add", ModifKinds::Character),
    ("scheme_discovery_chance_mult", ModifKinds::Character),
    ("scheme_power", ModifKinds::Scheme),
    ("scheme_resistance", ModifKinds::Scheme),
    ("scheme_secrecy", ModifKinds::Scheme),
    ("scheme_success_chance", ModifKinds::Scheme),
    ("short_reign_duration_mult", ModifKinds::Character),
    ("siege_morale_loss", ModifKinds::Character),
    ("siege_phase_time", ModifKinds::Character),
    ("spouse_opinion", ModifKinds::Character),
    ("stationed_maa_damage_add", ModifKinds::Province),
    ("stationed_maa_damage_mult", ModifKinds::Province),
    ("stationed_maa_pursuit_add", ModifKinds::Province),
    ("stationed_maa_pursuit_mult", ModifKinds::Province),
    ("stationed_maa_screen_add", ModifKinds::Province),
    ("stationed_maa_screen_mult", ModifKinds::Province),
    ("stationed_maa_siege_value_add", ModifKinds::Province),
    ("stationed_maa_siege_value_mult", ModifKinds::Province),
    ("stationed_maa_toughness_add", ModifKinds::Province),
    ("stationed_maa_toughness_mult", ModifKinds::Province),
    ("stewardship", ModifKinds::Character),
    ("stewardship_per_piety_level", ModifKinds::Character),
    ("stewardship_per_prestige_level", ModifKinds::Character),
    ("stewardship_per_stress_level", ModifKinds::Character),
    ("stewardship_scheme_power", ModifKinds::Character),
    ("stewardship_scheme_resistance", ModifKinds::Character),
    ("stress_gain_mult", ModifKinds::Character),
    ("stress_loss_mult", ModifKinds::Character),
    ("stress_loss_per_piety_level", ModifKinds::Character),
    ("stress_loss_per_prestige_level", ModifKinds::Character),
    ("strife_opinion_gain_mult", ModifKinds::Character),
    ("strife_opinion_loss_mult", ModifKinds::Character),
    ("supply_capacity_add", ModifKinds::Character),
    ("supply_capacity_mult", ModifKinds::Character),
    ("supply_duration", ModifKinds::Character),
    ("supply_limit", ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County)),
    (
        "supply_limit_mult",
        ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County),
    ),
    ("supply_loss_winter", ModifKinds::Province),
    ("tax_mult", ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County)),
    ("tax_slot_add", ModifKinds::Character),
    ("title_creation_cost", ModifKinds::Character),
    ("title_creation_cost_mult", ModifKinds::Character),
    ("tolerance_advantage_mod", ModifKinds::Character),
    ("travel_companion_opinion", ModifKinds::Character),
    ("travel_danger", ModifKinds::Character.union(ModifKinds::Province).union(ModifKinds::County)),
    ("travel_safety_mult", ModifKinds::TravelPlan),
    ("travel_safety", ModifKinds::TravelPlan),
    ("travel_speed_mult", ModifKinds::TravelPlan),
    ("travel_speed", ModifKinds::TravelPlan),
    ("twin_opinion", ModifKinds::Character),
    ("tyranny_gain_mult", ModifKinds::Character),
    ("tyranny_loss_mult", ModifKinds::Character),
    ("uncontrolled_province_advantage", ModifKinds::Character),
    ("vassal_levy_contribution_add", ModifKinds::Character),
    ("vassal_levy_contribution_mult", ModifKinds::Character),
    ("vassal_limit", ModifKinds::Character),
    ("vassal_opinion", ModifKinds::Character),
    ("vassal_tax_contribution_add", ModifKinds::Character),
    ("vassal_tax_contribution_mult", ModifKinds::Character),
    ("vassal_tax_mult", ModifKinds::Character),
    ("winter_advantage", ModifKinds::Character),
    ("winter_movement_speed", ModifKinds::Character),
    ("years_of_fertility", ModifKinds::Character),
];

static SPECIAL_MODIF_LOC_MAP: Lazy<FnvHashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut hash = FnvHashMap::default();
    for (s, loc) in SPECIAL_MODIF_LOC_TABLE.iter().copied() {
        hash.insert(s, loc);
    }
    hash
});

/// LAST UPDATED CK3 VERSION 1.11.4
/// Special cases for static modifs defined in `modifiers/modifiers_l_english.yml`
const SPECIAL_MODIF_LOC_TABLE: &[(&str, &str)] = &[
    // Negate penalty
    ("negate_diplomacy_penalty_add", "MOD_DIPLOMACY_NEGATE_PENALTY"),
    ("negate_intrigue_penalty_add", "MOD_INTRIGUE_NEGATE_PENALTY"),
    ("negate_learning_penalty_add", "MOD_LEARNING_NEGATE_PENALTY"),
    ("negate_martial_penalty_add", "MOD_MARTIAL_NEGATE_PENALTY"),
    ("negate_prowess_penalty_add", "MOD_PROWESS_NEGATE_PENALTY"),
    ("negate_stewardship_penalty_add", "MOD_STEWARDSHIP_NEGATE_PENALTY"),
    ("negate_fertility_penalty_add", "MOD_FERTILITY_NEGATE_PENALTY"),
    // Combat
    ("pursue_efficiency", "MOD_COMBAT_PURSUE_EFFICIENCY"),
    ("counter_efficiency", "MOD_COMBAT_COUNTER_EFFICIENCY"),
    ("counter_resistance", "MOD_COMBAT_COUNTER_RESISTANCE"),
    // Scheme
    ("scheme_success_chance", "MOD_SCHEME_SUCCESS"),
    ("owned_hostile_scheme_success_chance_add", "MOD_OWNED_HOSTILE_SCHEME_SUCCESS_ADD"),
    ("enemy_hostile_scheme_success_chance_add", "MOD_ENEMY_HOSTILE_SCHEME_SUCCESS_ADD"),
    ("owned_personal_scheme_success_chance_add", "MOD_OWNED_PERSONAL_SCHEME_SUCCESS_ADD"),
    ("enemy_personal_scheme_success_chance_add", "MOD_ENEMY_PERSONAL_SCHEME_SUCCESS_ADD"),
    // Advantage
    ("tolerance_advantage_mod", "MOD_FAITH_HOSTILITY_ADVANTAGE_MOD"),
    ("advantage_against_coreligionists", "MOD_CORELIGIONIST_ADVANTAGE_MOD"),
    ("led_by_owner_extra_advantage_add", "MOD_LEAD_BY_OWNER_ADVANTAGE"),
    ("same_heritage_county_advantage_add", "MOD_SAME_HERITAGE_COUNTY_ADVANTAGE"),
    ("independent_primary_defender_advantage_add", "MOD_INDEPENDENT_PRIMARY_DEFENDER_ADVANTAGE"),
    // Fort level
    ("fort_level", "MOD_HOLDING_FORT_LEVEL"),
    ("additional_fort_level", "MOD_ADDITIONAL_HOLDING_FORT_LEVEL"),
    // Construction
    ("build_speed", "MOD_CONSTRUCTION_SPEED"),
    ("build_gold_cost", "MOD_CONSTRUCTION_GOLD_COST"),
    ("build_piety_cost", "MOD_CONSTRUCTION_PIETY_COST"),
    ("build_prestige_cost", "MOD_CONSTRUCTION_PRESTIGE_COST"),
    ("holding_build_speed", "MOD_HOLDING_CONSTRUCTION_SPEED"),
    ("holding_build_gold_cost", "MOD_HOLDING_CONSTRUCTION_GOLD_COST"),
    ("holding_build_piety_cost", "MOD_HOLDING_CONSTRUCTION_PIETY_COST"),
    ("holding_build_prestige_cost", "MOD_HOLDING_CONSTRUCTION_PRESTIGE_COST"),
    // Building Slot
    ("building_slot_add", "MOD_NUM_BUILDING_SLOTS"),
    // County
    ("development_growth_factor", "MOD_MONTHLY_DEVELOPMENT_GROWTH_FACTOR"),
    ("development_growth", "MOD_MONTHLY_DEVELOPMENT_GROWTH"),
    (
        "character_capital_county_monthly_development_growth_add",
        "MOD_CHARACTER_CAPITAL_MONTHLY_DEVELOPMENT_GROWTH_ADD",
    ),
    ("monthly_county_control_change_add", "MOD_MONTHLY_COUNTY_CONTROL_GROWTH"),
    ("monthly_county_control_change_factor", "MOD_MONTHLY_COUNTY_CONTROL_GROWTH_FACTOR"),
    (
        "monthly_county_control_change_add_even_if_baron",
        "MOD_MONTHLY_COUNTY_CONTROL_GROWTH_EVEN_IF_BARON",
    ),
    (
        "monthly_county_control_change_factor_even_if_baron",
        "MOD_MONTHLY_COUNTY_CONTROL_GROWTH_FACTOR_EVEN_IF_BARON",
    ),
    ("monthly_county_control_change_at_war_add", "MOD_MONTHLY_COUNTY_CONTROL_GROWTH_AT_WAR"),
    (
        "monthly_county_control_change_at_war_mult",
        "MOD_MONTHLY_COUNTY_CONTROL_GROWTH_FACTOR_AT_WAR",
    ),
    ("different_faith_county_opinion_mult", "MOD_COUNTY_OPINION_DIFFERENT_FAITH_MULT"),
    (
        "different_faith_county_opinion_mult_even_if_baron",
        "MOD_COUNTY_OPINION_DIFFERENT_FAITH_MULT_EVEN_IF_BARON",
    ),
    // Culture
    ("mercenary_count_mult", "MOD_CULTURE_MERCENARY_MULT"),
    ("cultural_head_fascination_add", "MOD_CULTURAL_FASCINATION_INNOVATION_ADD"),
    ("cultural_head_fascination_mult", "MOD_CULTURAL_FASCINATION_INNOVATION_MULT"),
    ("culture_tradition_max_add", "MODE_CULTURE_TRADITION_MAX_ADD"), // sic
    // Court
    ("court_grandeur_baseline_add", "MOD_COURT_GRANDEUR_BASELINE"),
    // Tax Slot
    ("tax_slot_add", "MOD_NUM_TAX_SLOTS"),
];
