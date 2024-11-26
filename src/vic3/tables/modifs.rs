#![allow(non_upper_case_globals)]

use std::borrow::Cow;

use once_cell::sync::Lazy;

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

    // building_employment_$PopType$_add
    // building_employment_$PopType$_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("building_employment_") {
        for &sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::PopType, &part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
    }
    // building_group_$BuildingGroup$_$PopType$_fertility_mult
    // building_group_$BuildingGroup$_$PopType$_mortality_mult
    // building_group_$BuildingGroup$_$PopType$_standard_of_living_add
    // TODO: allowed_collectivization_add is not enabled for all bg
    // building_group_$BuildingGroup$_allowed_collectivization_add
    // building_group_$BuildingGroup$_employee_mult
    // building_group_$BuildingGroup$_fertility_mult
    // building_group_$BuildingGroup$_infrastructure_usage_mult
    // building_group_$BuildingGroup$_mortality_mult
    // building_group_$BuildingGroup$_standard_of_living_add
    // building_group_$BuildingGroup$_throughput_mult (obsolete)
    // building_group_$BuildingGroup$_unincorporated_throughput_add
    // building_group_$BuildingGroup$_throughput_add
    // building_group_$BuildingGroup$_tax_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("building_group_") {
        for &sfx in &["_fertility_mult", "_mortality_mult", "_standard_of_living_add"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                // This is tricky because both BuildingGroup and PopType can have `_` in them.
                for (i, _) in part.rmatch_indices_unchecked('_') {
                    if data.item_exists_lc(Item::PopType, &part.slice(i + 1..)) {
                        maybe_warn(Item::BuildingGroup, &part.slice(..i), name, data, warn);
                        return Some(ModifKinds::Building);
                    }
                }
                // Check if it's the kind without $PopType$
                maybe_warn(Item::BuildingGroup, &part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
        for &sfx in &[
            "_allowed_collectivization_add",
            "_infrastructure_usage_mult",
            "_employee_mult",
            "_tax_mult",
            "_unincorporated_throughput_add",
            "_throughput_add",
        ] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::BuildingGroup, &part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
        if let Some(part) = part.strip_suffix_unchecked("_throughput_mult") {
            maybe_warn(Item::BuildingGroup, &part, name, data, warn);
            if let Some(sev) = warn {
                let msg = format!("`{name}` was removed in 1.5");
                let info = "it was replaced with `_add`";
                report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
            }
            return Some(ModifKinds::Building);
        }
    }

    // $BuildingType$_throughput_mult (obsolete)
    if let Some(part) = name_lc.strip_suffix_unchecked("_throughput_mult") {
        maybe_warn(Item::BuildingType, &part, name, data, warn);
        if let Some(sev) = warn {
            let msg = format!("`{name}` was removed in 1.5");
            let info = "it was replaced with `_add`";
            report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
        }
        return Some(ModifKinds::Building);
    }
    // $BuildingType$_throughput_add
    if let Some(part) = name_lc.strip_suffix_unchecked("_throughput_add") {
        maybe_warn(Item::BuildingType, &part, name, data, warn);
        return Some(ModifKinds::Building);
    }

    // building_$PopType$_fertility_mult
    // building_$PopType$_mortality_mult
    // building_$PopType$_shares_add
    // building_$PopType$_shares_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("building_") {
        for &sfx in &["_fertility_mult", "_mortality_mult", "_shares_add", "_shares_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::PopType, &part, name, data, warn);
                return Some(ModifKinds::Building);
            }
        }
    }

    // building_input_$Goods$_add (obsolete)
    if let Some(part) = name_lc.strip_prefix_unchecked("building_input_") {
        if let Some(part) = part.strip_suffix_unchecked("_add") {
            maybe_warn(Item::Goods, &part, name, data, warn);
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
    if let Some(part) = name_lc.strip_prefix_unchecked("goods_input_") {
        for &sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::Goods, &part, name, data, warn);
                return Some(ModifKinds::Goods);
            }
        }
    }

    // building_output_$Goods$_add (obsolete)
    // building_output_$Goods$_mult (obsolete)
    if let Some(part) = name_lc.strip_prefix_unchecked("building_output_") {
        // TODO: some goods don't have the _mult version. Figure out why.
        for &sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::Goods, &part, name, data, warn);
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
    if let Some(part) = name_lc.strip_prefix_unchecked("goods_output_") {
        // TODO: some goods don't have the _mult version. Figure out why.
        for &sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::Goods, &part, name, data, warn);
                return Some(ModifKinds::Goods);
            }
        }
    }

    // character_$BattleCondition$_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("character_") {
        if let Some(part) = part.strip_suffix_unchecked("_mult") {
            maybe_warn(Item::BattleCondition, &part, name, data, warn);
            return Some(ModifKinds::Character);
        }
    }

    // TODO: this is only for a few institutions
    // country_institution_cost_$Institution$_add
    // country_institution_cost_$Institution$_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("country_institution_cost_") {
        for &sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::Institution, &part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // TODO: this is only for 1 institution
    // country_institution_impact_$Institution$_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("country_institution_impact_") {
        if let Some(part) = part.strip_suffix_unchecked("_mult") {
            maybe_warn(Item::Institution, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // TODO: this is only for 2 institutions
    // country_institution_size_change_speed_$Institution$_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("country_institution_size_change_speed_") {
        if let Some(part) = part.strip_suffix_unchecked("_mult") {
            maybe_warn(Item::Institution, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // country_can_impose_same_$LawGroup$_in_power_bloc_bool
    if let Some(part) = name_lc.strip_prefix_unchecked("country_can_impose_same_") {
        if let Some(part) = part.strip_suffix_unchecked("_in_power_bloc_bool") {
            maybe_warn(Item::LawGroup, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // country_$PopType$_pol_str_mult
    // country_$PopType$_voting_power_add
    if let Some(part) = name_lc.strip_prefix_unchecked("country_") {
        for &sfx in &["_pol_str_mult", "_voting_power_add"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::PopType, &part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // country_$Institution$_max_investment_add
    if let Some(part) = name_lc.strip_prefix_unchecked("country_") {
        if let Some(part) = part.strip_suffix_unchecked("_max_investment_add") {
            maybe_warn(Item::Institution, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // country_subsidies_$BuildingGroup$
    if let Some(part) = name_lc.strip_prefix_unchecked("country_subsidies_") {
        maybe_warn(Item::BuildingGroup, &part, name, data, warn);
        if let Some(sev) = warn {
            let msg = format!("`{name}` was removed in 1.7");
            report(ErrorKey::Removed, sev).msg(msg).loc(name).push();
        }
        return Some(ModifKinds::Country);
    }

    // country_enactment_success_chance_$Law$_add
    if let Some(part) = name_lc.strip_prefix_unchecked("country_enactment_success_chance_") {
        if let Some(part) = part.strip_suffix_unchecked("_add") {
            maybe_warn(Item::LawType, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // country_allow_assimilation_$AcceptanceStatus$_bool
    // country_allow_conversion_$AcceptanceStatus$_bool
    // country_allow_voting_$AcceptanceStatus$_bool
    // country_disallow_government_work_$AcceptanceStatus$_bool
    // country_disallow_military_work_$AcceptanceStatus$_bool
    for &pfx in &[
        "country_allow_assimilation_",
        "country_allow_conversion_",
        "country_allow_voting_",
        "country_disallow_government_work_",
        "country_disallow_military_work_",
    ] {
        if let Some(part) = name_lc.strip_prefix_unchecked(pfx) {
            if let Some(part) = part.strip_suffix_unchecked("_bool") {
                maybe_warn(Item::AcceptanceStatus, &part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // country_disallow_$Law$_bool
    if let Some(part) = name_lc.strip_prefix_unchecked("country_disallow_") {
        if let Some(part) = part.strip_suffix_unchecked("_bool") {
            maybe_warn(Item::LawType, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // country_assimilation_$AcceptanceStatus$_mult
    // country_loyalism_increases_$AcceptanceStatus$_mult
    // country_political_strength_$AcceptanceStatus$_mult
    // country_qualification_growth_$AcceptanceStatus$_mult
    // country_radicalism_increases_$AcceptanceStatus$_mult
    // country_voting_power_$AcceptanceStatus$_mult
    // country_wage_$AcceptanceStatus$_mult
    for &pfx in &[
        "country_assimilation_",
        "country_loyalism_increases_",
        "country_political_strength_",
        "country_qualification_growth_",
        "country_radicalism_increases_",
        "country_voting_power_",
        "country_wage_",
    ] {
        if let Some(part) = name_lc.strip_prefix_unchecked(pfx) {
            if let Some(part) = part.strip_suffix_unchecked("_mult") {
                maybe_warn(Item::AcceptanceStatus, &part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // country_standard_of_living_$AcceptanceStatus$_add
    if let Some(part) = name_lc.strip_prefix_unchecked("country_standard_of_living_") {
        if let Some(part) = part.strip_suffix_unchecked("_add") {
            maybe_warn(Item::AcceptanceStatus, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // country_$SocialClass$_acceptance_max_add
    // country_$SocialClass$_acceptance_min_add
    // country_$SocialClass$_cultural_acceptance_add
    // country_$SocialClass$_cultural_acceptance_mult
    // country_$SocialClass$_education_access_add
    // country_$SocialClass$_education_access_mult
    // country_$SocialClass$_qualification_growth_add
    // country_$SocialClass$_qualification_growth_mult
    // country_$SocialClass$_qualification_growth_other_class
    // country_$SocialClass$_qualification_growth_same_class
    if let Some(part) = name_lc.strip_prefix_unchecked("country_") {
        for &sfx in &[
            "_acceptance_max_add",
            "_acceptance_min_add",
            "_cultural_acceptance_add",
            "_cultural_acceptance_mult",
            "_education_access_add",
            "_education_access_mult",
            "_qualification_growth_add",
            "_qualification_growth_mult",
            "_qualification_growth_other_class",
            "_qualification_growth_same_class",
        ] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::SocialClass, &part, name, data, warn);
                return Some(ModifKinds::Country);
            }
        }
    }

    // TODO: this is only for a few laws
    // country_enactment_time_$Law$_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("country_enactment_time_") {
        if let Some(part) = part.strip_suffix_unchecked("_mult") {
            maybe_warn(Item::LawType, &part, name, data, warn);
            return Some(ModifKinds::Country);
        }
    }

    // interest_group_$InterestGroup$_approval_add
    // interest_group_$InterestGroup$_pol_str_mult
    // interest_group_$InterestGroup$_pop_attraction_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("interest_group_") {
        for &sfx in &["_approval_add", "_pol_str_mult", "_pop_attraction_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::InterestGroup, &part, name, data, warn);
                return Some(ModifKinds::InterestGroup);
            }
        }
    }

    // state_$Culture$_standard_of_living_add
    // state_$Religion$_standard_of_living_add
    if let Some(part) = name_lc.strip_prefix_unchecked("state_") {
        if let Some(part) = part.strip_suffix_unchecked("_standard_of_living_add") {
            if let Some(sev) = warn {
                if !data.item_exists_lc(Item::Religion, &part)
                    && !data.item_exists_lc(Item::Culture, &part)
                {
                    let msg = format!("{part} not found as culture or religion");
                    let info = format!("so the modifier {name} does not exist");
                    report(ErrorKey::MissingItem, sev).msg(msg).info(info).loc(name).push();
                }
            }
            return Some(ModifKinds::State);
        }
    }

    // state_$PopType$_consumption_multiplier_add
    // state_$PopType$_dependent_wage_mult
    // state_$PopType$_internal_migration_disallowed_bool
    // state_$PopType$_investment_pool_contribution_add
    // state_$PopType$_investment_pool_efficiency_mult
    // state_$PopType$_mass_migration_disallowed_bool
    // state_$PopType$_mortality_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("state_") {
        for &sfx in &[
            "_consumption_multiplier_add",
            "_dependent_wage_mult",
            "_internal_migration_disallowed_bool",
            "_investment_pool_contribution_add",
            "_investment_pool_efficiency_mult",
            "_mass_migration_disallowed_bool",
            "_mortality_mult",
        ] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::PopType, &part, name, data, warn);
                return Some(ModifKinds::State);
            }
        }
    }

    // state_pop_support_$PoliticalMovement$_add
    // state_pop_support_$PoliticalMovement$_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("state_pop_support_") {
        for &sfx in &["_add", "_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                if data.item_exists_lc(Item::LawType, &part) {
                    if let Some(sev) = warn {
                        let msg = "support for law types has been replaced by support for political movements";
                        report(ErrorKey::Removed, sev).msg(msg).loc(name).push();
                    }
                } else {
                    maybe_warn(Item::PoliticalMovement, &part, name, data, warn);
                }
                return Some(ModifKinds::State);
            }
        }
    }

    // state_$Building$_max_level_add
    if let Some(part) = name_lc.strip_prefix_unchecked("state_") {
        if let Some(part) = part.strip_suffix_unchecked("_max_level_add") {
            maybe_warn(Item::BuildingType, &part, name, data, warn);

            if let Some(sev) = warn {
                if data.item_exists(Item::BuildingType, part.as_str())
                    && !data.item_has_property(Item::BuildingType, part.as_str(), "max_level")
                {
                    let msg = format!("building {part} does not have `has_max_level = yes`");
                    let info = format!("so the modifier {name} does not exist");
                    report(ErrorKey::MissingItem, sev)
                        .strong()
                        .msg(msg)
                        .info(info)
                        .loc(name)
                        .push();
                }
            }
            return Some(ModifKinds::State);
        }
    }

    // state_harvest_condition_$HarvestConditionType$_duration_mult
    // state_harvest_condition_$HarvestConditionType$_impact_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("state_harvest_condition_") {
        for &sfx in &["_duration_mult", "_impact_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::HarvestConditionType, &part, name, data, warn);
                return Some(ModifKinds::State);
            }
        }
    }

    // state_loyalism_increases_$AcceptanceStatus$_mult
    // state_radicalism_increases_$AcceptanceStatus$_mult
    for &pfx in &["state_loyalism_increases_", "state_radicalism_increases_"] {
        if let Some(part) = name_lc.strip_prefix_unchecked(pfx) {
            if let Some(part) = part.strip_suffix_unchecked("_mult") {
                maybe_warn(Item::AcceptanceStatus, &part, name, data, warn);
                return Some(ModifKinds::State);
            }
        }
    }

    // unit_defense_$TerrainKey$_add
    // unit_defense_$TerrainKey$_mult
    // unit_offense_$TerrainKey$_mult
    // unit_offense_$TerrainKey$_mult
    for &pfx in &["unit_defense_", "unit_offense_"] {
        if let Some(part) = name_lc.strip_prefix_unchecked(pfx) {
            for &sfx in &["_add", "_mult"] {
                if let Some(part) = part.strip_suffix_unchecked(sfx) {
                    maybe_warn(Item::TerrainKey, &part, name, data, warn);
                    return Some(ModifKinds::Unit);
                }
            }
        }
    }

    // TODO: not all of these exist for all unit types
    // unit_$CombatUnit$_offense_mult
    // unit_$CombatUnit$_offense_add
    if let Some(part) = name_lc.strip_prefix_unchecked("unit_") {
        for &sfx in &["_offense_add", "_offense_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::CombatUnit, &part, name, data, warn);
                return Some(ModifKinds::Unit);
            }
        }
    }

    // power_bloc_invite_acceptance_$CountryRank$_add
    if let Some(part) = name_lc.strip_prefix_unchecked("power_bloc_invite_acceptance_") {
        if let Some(part) = part.strip_suffix_unchecked("_add") {
            maybe_warn(Item::CountryRank, &part, name, data, warn);
            return Some(ModifKinds::PowerBloc);
        }
    }

    // power_bloc_mandate_progress_per_$CountryRank$_member_add
    // power_bloc_mandate_progress_per_$CountryRank$_member_mult
    if let Some(part) = name_lc.strip_prefix_unchecked("power_bloc_mandate_progress_per_") {
        for &sfx in &["_member_add", "_member_mult"] {
            if let Some(part) = part.strip_suffix_unchecked(sfx) {
                maybe_warn(Item::CountryRank, &part, name, data, warn);
                return Some(ModifKinds::PowerBloc);
            }
        }
    }

    // TODO: modifiers from terrain labels

    // User-defined modifs are accepted in Vic3.
    // They must have a ModifierType entry to be accepted by the game engine,
    // so if that exists then accept the modif.
    if data.item_exists_lc(Item::ModifierTypeDefinition, &name_lc) {
        return Some(ModifKinds::all());
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

/// Return the modifier localization keys.
/// It's usually just the name, but there are known exceptions.
pub fn modif_loc(name: &Token, data: &Everything) -> (Cow<'static, str>, Cow<'static, str>) {
    let name_lc = Lowercase::new(name.as_str());

    if MODIF_MAP.contains_key(&name_lc) {
        let desc_loc = format!("{name_lc}_desc");
        return (name_lc.into_cow(), Cow::Owned(desc_loc));
    }

    if let Some(part) = name_lc.strip_prefix_unchecked("state_") {
        if let Some(part) = part.strip_suffix_unchecked("_standard_of_living_add") {
            if data.item_exists_lc(Item::Religion, &part) {
                return (
                    Cow::Borrowed("STATE_RELIGION_SOL_MODIFIER"),
                    Cow::Borrowed("STATE_RELIGION_SOL_MODIFIER_DESC"),
                );
            } else if data.item_exists_lc(Item::Culture, &part) {
                return (
                    Cow::Borrowed("STATE_CULTURE_SOL_MODIFIER"),
                    Cow::Borrowed("STATE_CULTURE_SOL_MODIFIER_DESC"),
                );
            }
            // We need some kind of default for missing items, and cultures are more common.
            return (
                Cow::Borrowed("STATE_RELIGION_SOL_MODIFIER"),
                Cow::Borrowed("STATE_RELIGION_SOL_MODIFIER_DESC"),
            );
        }
    }
    // TODO: should the loca key be lowercased?
    let desc_loc = format!("{name}_desc");
    return (Cow::Borrowed(name.as_str()), Cow::Owned(desc_loc));
}

static MODIF_MAP: Lazy<TigerHashMap<Lowercase<'static>, ModifKinds>> = Lazy::new(|| {
    let mut hash = TigerHashMap::default();
    for (s, kind) in MODIF_TABLE.iter().copied() {
        hash.insert(Lowercase::new_unchecked(s), kind);
    }
    hash
});

/// LAST UPDATED VIC3 VERSION 1.8.1
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, ModifKinds)] = &[
    ("battle_casualties_mult", ModifKinds::Battle),
    ("battle_combat_width_mult", ModifKinds::Battle),
    ("battle_defense_owned_province_mult", ModifKinds::Battle),
    ("battle_offense_owned_province_mult", ModifKinds::Battle),
    ("building_cash_reserves_mult", ModifKinds::Building),
    ("building_economy_of_scale_level_cap_add", ModifKinds::Building),
    ("building_goods_input_mult", ModifKinds::Building),
    ("building_level_bureaucracy_cost_add", ModifKinds::Building),
    ("building_minimum_wage_mult", ModifKinds::Building),
    ("building_mobilization_cost_mult", ModifKinds::Building),
    ("building_nationalization_cost_mult", ModifKinds::Building),
    ("building_nationalization_investment_return_add", ModifKinds::Building),
    ("building_nationalization_radicals_mult", ModifKinds::Building),
    ("building_subsistence_output_add", ModifKinds::Building),
    ("building_subsistence_output_mult", ModifKinds::Building),
    ("building_throughput_add", ModifKinds::Building),
    ("building_training_rate_add", ModifKinds::Building),
    ("building_training_rate_mult", ModifKinds::Building),
    ("building_unincorporated_subsistence_output_mult", ModifKinds::Building),
    ("building_unincorporated_throughput_add", ModifKinds::Building),
    ("building_working_conditions_mult", ModifKinds::Building),
    ("character_advancement_speed_add", ModifKinds::Character),
    ("character_command_limit_add", ModifKinds::Character),
    ("character_command_limit_mult", ModifKinds::Character),
    ("character_convoy_protection_mult", ModifKinds::Character),
    ("character_convoy_raiding_mult", ModifKinds::Character),
    ("character_expedition_events_explorer_mult", ModifKinds::Character),
    ("character_health_add", ModifKinds::Character),
    ("character_interception_add", ModifKinds::Character),
    ("character_popularity_add", ModifKinds::Character),
    ("character_supply_route_cost_mult", ModifKinds::Character),
    ("country_acceptance_culture_base_add", ModifKinds::Country),
    ("country_acceptance_primary_culture_add", ModifKinds::Country),
    ("country_acceptance_religion_base_add", ModifKinds::Country),
    ("country_acceptance_shared_cultural_trait_add", ModifKinds::Country),
    ("country_acceptance_shared_heritage_and_cultural_trait_add", ModifKinds::Country),
    ("country_acceptance_shared_heritage_trait_add", ModifKinds::Country),
    ("country_acceptance_shared_religious_trait_add", ModifKinds::Country),
    ("country_acceptance_state_religion_add", ModifKinds::Country),
    ("country_agitator_slots_add", ModifKinds::Country),
    ("country_ahead_of_time_research_penalty_mult", ModifKinds::Country),
    ("country_all_buildings_protected_bool", ModifKinds::Country),
    ("country_allow_enacting_decrees_in_subject_bool", ModifKinds::Country),
    ("country_allow_multiple_alliances_bool", ModifKinds::Country),
    ("country_allow_national_collectivization_bool", ModifKinds::Country),
    ("country_allow_trade_routes_without_interest_bool", ModifKinds::Country),
    ("country_assimilation_delta_threshold_add", ModifKinds::Country),
    ("country_authority_add", ModifKinds::Country),
    ("country_authority_mult", ModifKinds::Country),
    ("country_authority_per_subject_add", ModifKinds::Country),
    ("country_bolster_attraction_mult", ModifKinds::Country),
    ("country_bolster_cost_mult", ModifKinds::Country),
    ("country_bureaucracy_add", ModifKinds::Country),
    ("country_bureaucracy_investment_cost_factor_mult", ModifKinds::Country),
    ("country_bureaucracy_mult", ModifKinds::Country),
    ("country_can_form_construction_company_bool", ModifKinds::Country),
    ("country_can_impose_same_lawgroup_army_model_in_power_bloc_bool", ModifKinds::Country),
    ("country_cannot_be_target_for_law_imposition_bool", ModifKinds::Country),
    ("country_cannot_cancel_law_enactment_bool", ModifKinds::Country),
    ("country_cannot_enact_laws_bool", ModifKinds::Country),
    ("country_cannot_start_law_enactment_bool", ModifKinds::Country),
    ("country_company_construction_efficiency_bonus_add", ModifKinds::Country),
    ("country_company_pay_to_establish_bool", ModifKinds::Country),
    ("country_company_throughput_bonus_add", ModifKinds::Country),
    ("country_construction_add", ModifKinds::Country),
    ("country_construction_goods_cost_mult", ModifKinds::Country),
    ("country_consumption_tax_cost_mult", ModifKinds::Country),
    ("country_conversion_delta_threshold_add", ModifKinds::Country),
    ("country_convoy_contribution_to_market_owner_add", ModifKinds::Country),
    ("country_convoys_capacity_add", ModifKinds::Country),
    ("country_convoys_capacity_mult", ModifKinds::Country),
    ("country_damage_relations_speed_mult", ModifKinds::Country),
    ("country_diplomatic_play_maneuvers_add", ModifKinds::Country),
    ("country_disable_investment_pool_bool", ModifKinds::Country),
    ("country_disable_nationalization_bool", ModifKinds::Country),
    ("country_disable_nationalization_without_compensation_bool", ModifKinds::Country),
    ("country_disable_privatization_bool", ModifKinds::Country),
    ("country_disallow_aggressive_plays_bool", ModifKinds::Country),
    ("country_disallow_agitator_invites_bool", ModifKinds::Country),
    ("country_economic_dependence_on_overlord_add", ModifKinds::Country),
    ("country_expedition_events_explorer_mult", ModifKinds::Country),
    ("country_expenses_add", ModifKinds::Country),
    ("country_force_privatization_bool", ModifKinds::Country),
    ("country_foreign_collectivization_bool", ModifKinds::Country),
    ("country_free_trade_routes_add", ModifKinds::Country),
    ("country_gold_reserve_limit_mult", ModifKinds::Country),
    ("country_government_buildings_protected_bool", ModifKinds::Country),
    ("country_government_dividends_efficiency_add", ModifKinds::Country),
    ("country_government_dividends_reinvestment_add", ModifKinds::Country),
    ("country_government_dividends_waste_add", ModifKinds::Country),
    ("country_government_wages_mult", ModifKinds::Country),
    ("country_higher_diplomatic_acceptance_same_religion_bool", ModifKinds::Country),
    ("country_higher_leverage_from_economic_dependence_bool", ModifKinds::Country),
    ("country_ignores_landing_craft_penalty_bool", ModifKinds::Country),
    ("country_improve_relations_speed_mult", ModifKinds::Country),
    ("country_infamy_decay_mult", ModifKinds::Country),
    ("country_infamy_generation_mult", ModifKinds::Country),
    ("country_infamy_generation_against_unrecognized_mult", ModifKinds::Country),
    ("country_influence_add", ModifKinds::Country),
    ("country_influence_mult", ModifKinds::Country),
    ("country_initiator_war_goal_maneuver_cost_mult", ModifKinds::Country),
    ("country_institution_size_change_speed_mult", ModifKinds::Country),
    ("country_join_power_bloc_member_in_defensive_plays_bool", ModifKinds::Country),
    ("country_join_power_bloc_member_in_plays_bool", ModifKinds::Country),
    ("country_law_enactment_imposition_success_add", ModifKinds::Country),
    ("country_law_enactment_max_setbacks_add", ModifKinds::Country),
    ("country_law_enactment_stall_mult", ModifKinds::Country),
    ("country_law_enactment_success_add", ModifKinds::Country),
    ("country_law_enactment_time_mult", ModifKinds::Country),
    ("country_leader_has_law_enactment_success_mult", ModifKinds::Country),
    ("country_legitimacy_base_add", ModifKinds::Country),
    ("country_legitimacy_govt_leader_clout_add", ModifKinds::Country),
    ("country_legitimacy_govt_size_add", ModifKinds::Country),
    ("country_legitimacy_govt_total_clout_add", ModifKinds::Country),
    ("country_legitimacy_govt_total_votes_add", ModifKinds::Country),
    ("country_legitimacy_headofstate_add", ModifKinds::Country),
    ("country_legitimacy_ideological_incoherence_mult", ModifKinds::Country),
    ("country_legitimacy_min_add", ModifKinds::Country),
    ("country_leverage_generation_add", ModifKinds::Country),
    ("country_leverage_generation_mult", ModifKinds::Country),
    ("country_leverage_resistance_add", ModifKinds::Country),
    ("country_leverage_resistance_mult", ModifKinds::Country),
    ("country_liberty_desire_add", ModifKinds::Country),
    ("country_liberty_desire_decrease_mult", ModifKinds::Country),
    ("country_liberty_desire_increase_mult", ModifKinds::Country),
    ("country_liberty_desire_of_subjects_mult", ModifKinds::Country),
    ("country_loan_interest_rate_add", ModifKinds::Country),
    ("country_loan_interest_rate_mult", ModifKinds::Country),
    ("country_lobby_leverage_generation_mult", ModifKinds::Country),
    ("country_loyalists_from_legitimacy_mult", ModifKinds::Country),
    ("country_mass_migration_attraction_mult", ModifKinds::Country),
    ("country_max_companies_add", ModifKinds::Country),
    ("country_max_declared_interests_add", ModifKinds::Country),
    ("country_max_declared_interests_mult", ModifKinds::Country),
    ("country_max_weekly_construction_progress_add", ModifKinds::Country),
    ("country_migration_restrictiveness_add", ModifKinds::Country),
    ("country_military_goods_cost_mult", ModifKinds::Country),
    ("country_military_tech_research_speed_mult", ModifKinds::Country),
    ("country_military_tech_spread_mult", ModifKinds::Country),
    ("country_military_wages_mult", ModifKinds::Country),
    ("country_minting_add", ModifKinds::Country),
    ("country_minting_mult", ModifKinds::Country),
    ("country_must_have_movement_to_enact_laws_bool", ModifKinds::Country),
    ("country_nationalization_cost_non_members_mult", ModifKinds::Country),
    ("country_non_state_religion_wages_mult", ModifKinds::Country),
    ("country_opposition_ig_approval_add", ModifKinds::Country),
    ("country_overlord_income_transfer_mult", ModifKinds::Country),
    ("country_pact_leverage_generation_add", ModifKinds::Country),
    ("country_pact_leverage_generation_mult", ModifKinds::Country),
    ("country_party_whip_impact_mult", ModifKinds::Country),
    ("country_port_connection_cost_mult", ModifKinds::Country),
    ("country_prestige_add", ModifKinds::Country),
    ("country_prestige_from_army_power_projection_mult", ModifKinds::Country),
    ("country_prestige_from_navy_power_projection_mult", ModifKinds::Country),
    ("country_prestige_mult", ModifKinds::Country),
    ("country_private_construction_allocation_mult", ModifKinds::Country),
    ("country_production_tech_research_speed_mult", ModifKinds::Country),
    ("country_production_tech_spread_mult", ModifKinds::Country),
    ("country_radicals_from_conquest_mult", ModifKinds::Country),
    ("country_radicals_from_legitimacy_mult", ModifKinds::Country),
    ("country_reduced_liberty_desire_same_religion_bool", ModifKinds::Country),
    ("country_resource_depletion_chance_mult", ModifKinds::Country),
    ("country_resource_discovery_chance_mult", ModifKinds::Country),
    ("country_revolution_clock_time_add", ModifKinds::Country),
    ("country_revolution_progress_add", ModifKinds::Country),
    ("country_revolution_progress_mult", ModifKinds::Country),
    ("country_secession_clock_time_add", ModifKinds::Country),
    ("country_secession_progress_add", ModifKinds::Country),
    ("country_secession_progress_mult", ModifKinds::Country),
    ("country_society_tech_research_speed_mult", ModifKinds::Country),
    ("country_society_tech_spread_mult", ModifKinds::Country),
    ("country_state_religion_wages_mult", ModifKinds::Country),
    ("country_subject_income_transfer_heathen_mult", ModifKinds::Country),
    ("country_subject_income_transfer_mult", ModifKinds::Country),
    ("country_suppression_attraction_mult", ModifKinds::Country),
    ("country_suppression_cost_mult", ModifKinds::Country),
    ("country_tax_income_add", ModifKinds::Country),
    ("country_tech_group_research_speed_mult", ModifKinds::Country),
    ("country_tech_research_speed_mult", ModifKinds::Country),
    ("country_tech_spread_add", ModifKinds::Country),
    ("country_tech_spread_mult", ModifKinds::Country),
    ("country_tension_decay_mult", ModifKinds::Country),
    ("country_trade_route_competitiveness_mult", ModifKinds::Country),
    ("country_trade_route_cost_mult", ModifKinds::Country),
    ("country_trade_route_quantity_mult", ModifKinds::Country),
    ("country_voting_power_base_add", ModifKinds::Country),
    ("country_voting_power_from_literacy_add", ModifKinds::Country),
    ("country_voting_power_wealth_threshold_add", ModifKinds::Country),
    ("country_war_exhaustion_casualties_mult", ModifKinds::Country),
    ("country_weekly_innovation_add", ModifKinds::Country),
    ("country_weekly_innovation_max_add", ModifKinds::Country),
    ("country_weekly_innovation_mult", ModifKinds::Country),
    ("interest_group_approval_add", ModifKinds::InterestGroup),
    ("interest_group_in_government_approval_add", ModifKinds::InterestGroup),
    ("interest_group_in_government_attraction_mult", ModifKinds::InterestGroup),
    ("interest_group_in_opposition_agitator_popularity_add", ModifKinds::InterestGroup),
    ("interest_group_in_opposition_approval_add", ModifKinds::InterestGroup),
    ("interest_group_pol_str_factor", ModifKinds::InterestGroup),
    ("interest_group_pol_str_mult", ModifKinds::InterestGroup),
    ("interest_group_pop_attraction_mult", ModifKinds::InterestGroup),
    ("market_disallow_trade_routes_bool", ModifKinds::Market),
    ("market_land_trade_capacity_add", ModifKinds::Market),
    ("market_max_exports_add", ModifKinds::Market),
    ("market_max_imports_add", ModifKinds::Market),
    ("military_formation_attrition_risk_add", ModifKinds::MilitaryFormation),
    ("military_formation_attrition_risk_mult", ModifKinds::MilitaryFormation),
    ("military_formation_mobilization_speed_add", ModifKinds::MilitaryFormation),
    ("military_formation_mobilization_speed_mult", ModifKinds::MilitaryFormation),
    ("military_formation_movement_speed_add", ModifKinds::MilitaryFormation),
    ("military_formation_movement_speed_mult", ModifKinds::MilitaryFormation),
    ("military_formation_organization_gain_add", ModifKinds::MilitaryFormation),
    ("military_formation_organization_gain_mult", ModifKinds::MilitaryFormation),
    ("political_movement_character_attraction_mult", ModifKinds::PoliticalMovement),
    ("political_movement_pop_attraction_mult", ModifKinds::PoliticalMovement),
    ("political_movement_radicalism_add", ModifKinds::PoliticalMovement),
    ("political_movement_radicalism_from_enactment_approval_mult", ModifKinds::PoliticalMovement),
    (
        "political_movement_radicalism_from_enactment_disapproval_mult",
        ModifKinds::PoliticalMovement,
    ),
    ("power_bloc_allow_foreign_investment_lower_rank_bool", ModifKinds::PowerBloc),
    ("power_bloc_allow_wider_migration_area_bool", ModifKinds::PowerBloc),
    ("power_bloc_cohesion_add", ModifKinds::PowerBloc),
    ("power_bloc_cohesion_mult", ModifKinds::PowerBloc),
    ("power_bloc_cohesion_per_member_add", ModifKinds::PowerBloc),
    ("power_bloc_customs_union_bool", ModifKinds::PowerBloc),
    ("power_bloc_disallow_embargo_bool", ModifKinds::PowerBloc),
    ("power_bloc_disallow_war_bool", ModifKinds::PowerBloc),
    ("power_bloc_income_transfer_to_leader_factor", ModifKinds::PowerBloc),
    ("power_bloc_invite_acceptance_add", ModifKinds::PowerBloc),
    ("power_bloc_leader_can_add_wargoal_bool", ModifKinds::PowerBloc),
    ("power_bloc_leader_can_force_state_religion_bool", ModifKinds::PowerBloc),
    ("power_bloc_leader_can_make_subjects_bool", ModifKinds::PowerBloc),
    ("power_bloc_leader_can_regime_change_bool", ModifKinds::PowerBloc),
    ("power_bloc_leverage_generation_mult", ModifKinds::PowerBloc),
    ("power_bloc_mandate_progress_mult", ModifKinds::PowerBloc),
    ("power_bloc_religion_trade_route_competitiveness_mult", ModifKinds::PowerBloc),
    ("power_bloc_target_sway_cost_mult", ModifKinds::PowerBloc),
    ("power_bloc_trade_route_cost_mult", ModifKinds::PowerBloc),
    ("state_assimilation_mult", ModifKinds::State),
    ("state_birth_rate_mult", ModifKinds::State),
    ("state_bureaucracy_population_base_cost_factor_mult", ModifKinds::State),
    ("state_colony_growth_creation_factor", ModifKinds::State),
    ("state_colony_growth_speed_mult", ModifKinds::State),
    ("state_conscription_rate_add", ModifKinds::State),
    ("state_conscription_rate_mult", ModifKinds::State),
    ("state_construction_mult", ModifKinds::State),
    ("state_conversion_mult", ModifKinds::State),
    ("state_decree_cost_mult", ModifKinds::State),
    ("state_dependent_political_participation_add", ModifKinds::State),
    ("state_dependent_wage_add", ModifKinds::State),
    ("state_dependent_wage_mult", ModifKinds::State),
    ("state_disallow_incorporation_bool", ModifKinds::State),
    ("state_education_access_add", ModifKinds::State),
    ("state_education_access_wealth_add", ModifKinds::State),
    ("state_expected_sol_from_literacy", ModifKinds::State),
    ("state_expected_sol_mult", ModifKinds::State),
    ("state_food_security_add", ModifKinds::State),
    ("state_infrastructure_add", ModifKinds::State),
    ("state_infrastructure_from_automobiles_consumption_add", ModifKinds::State),
    ("state_infrastructure_from_population_add", ModifKinds::State),
    ("state_infrastructure_from_population_max_add", ModifKinds::State),
    ("state_infrastructure_from_population_max_mult", ModifKinds::State),
    ("state_infrastructure_from_population_mult", ModifKinds::State),
    ("state_infrastructure_mult", ModifKinds::State),
    ("state_lower_strata_expected_sol_add", ModifKinds::State),
    ("state_lower_strata_standard_of_living_add", ModifKinds::State),
    ("state_loyalists_from_political_movements_mult", ModifKinds::State),
    ("state_middle_strata_expected_sol_add", ModifKinds::State),
    ("state_middle_strata_standard_of_living_add", ModifKinds::State),
    ("state_migration_pull_add", ModifKinds::State),
    ("state_migration_pull_mult", ModifKinds::State),
    ("state_migration_pull_unincorporated_mult", ModifKinds::State),
    ("state_migration_push_mult", ModifKinds::State),
    ("state_migration_quota_mult", ModifKinds::State),
    ("state_market_access_price_impact", ModifKinds::State),
    ("state_mortality_mult", ModifKinds::State),
    ("state_mortality_turmoil_mult", ModifKinds::State),
    ("state_mortality_wealth_mult", ModifKinds::State),
    ("state_non_homeland_colony_growth_speed_mult", ModifKinds::State),
    ("state_non_homeland_mortality_mult", ModifKinds::State),
    ("state_peasants_education_access_add", ModifKinds::State),
    ("state_political_strength_from_wealth_mult", ModifKinds::State),
    ("state_political_strength_from_welfare_mult", ModifKinds::State),
    ("state_pollution_generation_add", ModifKinds::State),
    ("state_pollution_reduction_health_mult", ModifKinds::State),
    ("state_pop_pol_str_add", ModifKinds::State),
    ("state_pop_pol_str_mult", ModifKinds::State),
    ("state_pop_qualifications_mult", ModifKinds::State),
    ("state_radicals_and_loyalists_from_sol_change_mult", ModifKinds::State),
    ("state_radicals_from_political_movements_mult", ModifKinds::State),
    ("state_slave_import_mult", ModifKinds::State),
    ("state_standard_of_living_add", ModifKinds::State),
    ("state_tax_capacity_add", ModifKinds::State),
    ("state_tax_capacity_mult", ModifKinds::State),
    ("state_tax_collection_mult", ModifKinds::State),
    ("state_tax_waste_add", ModifKinds::State),
    ("state_turmoil_effects_mult", ModifKinds::State),
    ("state_unincorporated_starting_wages_mult", ModifKinds::State),
    ("state_upper_strata_expected_sol_add", ModifKinds::State),
    ("state_upper_strata_standard_of_living_add", ModifKinds::State),
    ("state_urbanization_per_level_add", ModifKinds::State),
    ("state_urbanization_per_level_mult", ModifKinds::State),
    ("state_welfare_payments_add", ModifKinds::State),
    ("state_welfare_payments_mult", ModifKinds::State),
    ("state_working_adult_ratio_add", ModifKinds::State),
    ("tariff_export_add", ModifKinds::Tariff),
    ("tariff_export_outside_power_bloc_mult", ModifKinds::Tariff),
    ("tariff_import_add", ModifKinds::Tariff),
    ("tariff_import_outside_power_bloc_mult", ModifKinds::Tariff),
    ("tax_consumption_add", ModifKinds::Tax),
    ("tax_dividends_add", ModifKinds::Tax),
    ("tax_heathen_add", ModifKinds::Tax),
    ("tax_income_add", ModifKinds::Tax),
    ("tax_land_add", ModifKinds::Tax),
    ("tax_per_capita_add", ModifKinds::Tax),
    ("unit_advancement_speed_mult", ModifKinds::Unit),
    ("unit_army_defense_add", ModifKinds::Unit),
    ("unit_army_defense_mult", ModifKinds::Unit),
    ("unit_army_experience_gain_add", ModifKinds::Unit),
    ("unit_army_experience_gain_mult", ModifKinds::Unit),
    ("unit_army_offense_add", ModifKinds::Unit),
    ("unit_army_offense_mult", ModifKinds::Unit),
    ("unit_convoy_defense_mult", ModifKinds::Unit),
    ("unit_convoy_raiding_interception_mult", ModifKinds::Unit),
    ("unit_convoy_raiding_mult", ModifKinds::Unit),
    ("unit_convoy_requirements_mult", ModifKinds::Unit),
    ("unit_defense_add", ModifKinds::Unit),
    ("unit_defense_mult", ModifKinds::Unit),
    ("unit_devastation_mult", ModifKinds::Unit),
    ("unit_experience_gain_add", ModifKinds::Unit),
    ("unit_experience_gain_mult", ModifKinds::Unit),
    ("unit_kill_rate_add", ModifKinds::Unit),
    ("unit_morale_damage_mult", ModifKinds::Unit),
    ("unit_morale_loss_add", ModifKinds::Unit),
    ("unit_morale_loss_mult", ModifKinds::Unit),
    ("unit_morale_recovery_mult", ModifKinds::Unit),
    ("unit_navy_defense_add", ModifKinds::Unit),
    ("unit_navy_defense_mult", ModifKinds::Unit),
    ("unit_navy_experience_gain_add", ModifKinds::Unit),
    ("unit_navy_experience_gain_mult", ModifKinds::Unit),
    ("unit_navy_offense_add", ModifKinds::Unit),
    ("unit_navy_offense_mult", ModifKinds::Unit),
    ("unit_occupation_mult", ModifKinds::Unit),
    ("unit_offense_add", ModifKinds::Unit),
    ("unit_offense_mult", ModifKinds::Unit),
    ("unit_provinces_captured_mult", ModifKinds::Unit),
    ("unit_provinces_lost_mult", ModifKinds::Unit),
    ("unit_recovery_rate_add", ModifKinds::Unit),
    ("unit_supply_consumption_mult", ModifKinds::Unit),
];

static MODIF_REMOVED_MAP: Lazy<TigerHashMap<Lowercase<'static>, &'static str>> = Lazy::new(|| {
    let mut hash = TigerHashMap::default();
    for (s, info) in MODIF_REMOVED_TABLE.iter().copied() {
        hash.insert(Lowercase::new_unchecked(s), info);
    }
    hash
});

const MODIF_REMOVED_TABLE: &[(&str, &str)] = &[
    ("building_input_mult", "replaced in 1.5 with building_goods_input_mult"),
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
    ("character_command_limit_combat_unit_conscript_add", "removed in 1.6"),
    ("character_command_limit_combat_unit_flotilla_add", "removed in 1.6"),
    ("character_command_limit_combat_unit_regular_add", "removed in 1.6"),
    ("building_government_shares_add", "removed in 1.7"),
    ("building_production_mult", "removed in 1.7"),
    ("building_throughput_oil_mult", "removed in 1.7"),
    ("building_workforce_shares_add", "removed in 1.7"),
    ("country_all_buildings_protected", "renamed to country_all_buildings_protected_bool in 1.7"),
    ("country_allow_multiple_alliances", "renamed to country_allow_multiple_alliances_bool in 1.7"),
    ("country_cannot_enact_laws", "renamed to country_cannot_enact_laws_bool in 1.7"),
    ("country_decree_cost_mult", "removed in 1.7"),
    ("country_disable_investment_pool", "renamed to country_disable_investment_pool_bool in 1.7"),
    (
        "country_disallow_aggressive_plays",
        "renamed to country_disallow_aggressive_plays_bool in 1.7",
    ),
    (
        "country_disallow_agitator_invites",
        "renamed to country_disallow_agitator_invites_bool in 1.7",
    ),
    (
        "country_disallow_discriminated_migration",
        "renamed to country_disallow_discriminated_migration_bool in 1.7",
    ),
    ("country_disallow_migration", "renamed to country_disallow_migration_bool in 1.7"),
    (
        "country_government_buildings_protected",
        "renamed to country_government_buildings_protected_bool in 1.7",
    ),
    (
        "country_ignores_landing_craft_penalty",
        "renamed to country_ignores_landing_craft_penalty_bool in 1.7",
    ),
    ("country_mandate_subsidies", "removed in 1.7"),
    (
        "country_must_have_movement_to_enact_laws",
        "renamed to country_must_have_movement_to_enact_laws_bool in 1.7",
    ),
    ("country_private_buildings_protected", "removed in 1.7"),
    ("country_promotion_ig_attraction_mult", "removed in 1.7"),
    ("country_subsidies_all", "removed in 1.7"),
    ("market_disallow_trade_routes", "renamed to market_disallow_trade_routes_bool in 1.7"),
    ("state_disallow_incorporation", "renamed to state_disallow_incorporation_bool in 1.7"),
    ("state_port_range_add", "removed in 1.7"),
    ("state_unincorporated_standard_of_living_add", "removed in 1.7"),
    ("state_urbanization_add", "removed in 1.7"),
    ("state_urbanization_mult", "removed in 1.7"),
    ("unit_mobilization_speed_mult", "removed in 1.7"),
    ("country_leverage_resistance_per_population_add", "removed in 1.7.1"),
    ("character_morale_cap_add", "removed in 1.8"),
    ("country_bolster_ig_attraction_mult", "removed in 1.8"),
    ("country_disallow_discriminated_migration_bool", "removed in 1.8"),
    ("country_disallow_migration_bool", "removed in 1.8"),
    ("country_force_collectivization_bool", "removed in 1.8"),
    ("country_suppression_ig_attraction_mult", "removed in 1.8"),
    ("political_movement_enact_support_mult", "removed in 1.8"),
    ("political_movement_preserve_support_mult", "removed in 1.8"),
    ("political_movement_radicalism_mult", "removed in 1.8"),
    ("political_movement_restore_support_mult", "removed in 1.8"),
    ("political_movement_support_add", "removed in 1.8"),
    ("political_movement_support_mult", "removed in 1.8"),
    ("state_accepted_birth_rate_mult", "removed in 1.8"),
    (
        "state_colony_growth_creation_mult",
        "replaced with state_colony_growth_creation_factor in 1.8",
    ),
    ("state_loyalists_from_sol_change_accepted_culture_mult", "removed in 1.8"),
    ("state_loyalists_from_sol_change_accepted_religion_mult", "removed in 1.8"),
    ("state_loyalists_from_sol_change_mult", "removed in 1.8"),
    ("state_middle_expected_sol", "removed in 1.8"),
    ("state_middle_standard_of_living_add", "removed in 1.8"),
    ("state_minimum_wealth_add", "removed in 1.8"),
    ("state_political_strength_from_discrimination_mult", "removed in 1.8"),
    ("state_poor_expected_sol", "removed in 1.8"),
    ("state_poor_standard_of_living_add", "removed in 1.8"),
    ("state_radicals_from_discrimination_mult", "removed in 1.8"),
    ("state_radicals_from_sol_change_accepted_culture_mult", "removed in 1.8"),
    ("state_radicals_from_sol_change_accepted_religion_mult", "removed in 1.8"),
    ("state_radicals_from_sol_change_mult", "removed in 1.8"),
    ("state_rich_expected_sol", "removed in 1.8"),
    ("state_rich_standard_of_living_add", "removed in 1.8"),
];
