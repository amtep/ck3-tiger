use fnv::{FnvHashMap, FnvHashSet};
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::effect::validate_normal_effect;
use crate::errorkey::ErrorKey;
use crate::errors::{error_info, warn_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_duration, validate_modifiers_with_base};

#[derive(Clone, Debug, Default)]
pub struct OnActions {
    on_actions: FnvHashMap<String, OnAction>,
}

impl OnActions {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.on_actions.get_mut(key.as_str()) {
            on_action_special_append(&mut other.block, block);
        } else {
            self.on_actions
                .insert(key.to_string(), OnAction::new(key, block));
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.on_actions.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&OnAction> {
        self.on_actions.get(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.on_actions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for OnActions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/on_action")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(mut block) = PdxFile::read(entry, fullpath) else { return };
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

fn on_action_special_append(first: &mut Block, mut second: Block) {
    const SPECIAL_FIELDS: &[&str] = &[
        "events",
        "random_events",
        "first_valid",
        "on_actions",
        "random_on_action",
        "first_valid_on_action",
    ];
    let mut seen: FnvHashSet<String> = FnvHashSet::default();
    for (k, cmp, bv) in second.drain() {
        if let Some(key) = k {
            if let BV::Block(mut block) = bv {
                // For the special fields, append the first one we see to the first block's corresponding field.
                if SPECIAL_FIELDS.contains(&key.as_str()) && !seen.contains(&key.to_string()) {
                    seen.insert(key.to_string());
                    if first.add_to_field_block(key.as_str(), &mut block) {
                        continue;
                    }
                }
                first.add_key_value(key, cmp, BV::Block(block));
            } else {
                first.add_key_value(key, cmp, bv);
            }
        } else {
            first.add_value(bv);
        }
    }
}

#[derive(Clone, Debug)]
pub struct OnAction {
    key: Token,
    block: Block,
}

impl OnAction {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::non_primitive(), &self.key);
        for (name, root, named_scopes) in ON_ACTION_SCOPES {
            if self.key.is(name) {
                sc = ScopeContext::new(*root, &self.key);
                for (name, scope) in named_scopes.iter() {
                    sc.define_name(name, *scope, &self.key);
                }
                break;
            }
        }
        sc.set_strict_scopes(false);
        vd.field_validated_block("trigger", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block_sc("weight_multiplier", &mut sc, validate_modifiers_with_base);
        let mut count = 0;
        vd.field_validated_key_blocks("events", |key, b, data| {
            let mut vd = Validator::new(b, data);
            vd.field_validated_blocks_sc("delay", &mut sc, validate_duration);
            for token in vd.values() {
                data.verify_exists(Item::Event, token);
                data.events.check_scope(token, &mut sc);
            }
            count += 1;
            if count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try combining them into one block";
                warn_info(key, ErrorKey::Validation, &msg, info);
            }
        });
        count = 0;
        vd.field_validated_key_blocks("random_events", |key, b, data| {
            let mut vd = Validator::new(b, data);
            vd.field_numeric("chance_to_happen"); // TODO: 0 - 100
            vd.field_script_value("chance_of_no_event", &mut sc);
            vd.field_validated_blocks_sc("delay", &mut sc, validate_duration); // undocumented
            for (_key, token) in vd.integer_values() {
                if token.is("0") {
                    continue;
                }
                data.verify_exists(Item::Event, token);
                data.events.check_scope(token, &mut sc);
            }
            count += 1;
            if count == 2 {
                let msg = format!("multiple `{key}` blocks in one on_action do not work");
                let info = "try putting each into its own on_action and firing those separately";
                error_info(key, ErrorKey::Validation, &msg, info);
            }
        });
        count = 0;
        vd.field_validated_key_blocks("first_valid", |key, b, data| {
            let mut vd = Validator::new(b, data);
            for token in vd.values() {
                data.verify_exists(Item::Event, token);
                data.events.check_scope(token, &mut sc);
            }
            count += 1;
            if count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try putting each into its own on_action and firing those separately";
                warn_info(key, ErrorKey::Validation, &msg, info);
            }
        });
        count = 0;
        vd.field_validated_key_blocks("on_actions", |key, b, data| {
            let mut vd = Validator::new(b, data);
            vd.field_validated_blocks_sc("delay", &mut sc, validate_duration);
            for token in vd.values() {
                data.verify_exists(Item::OnAction, token);
            }
            count += 1;
            if count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try combining them into one block";
                warn_info(key, ErrorKey::Validation, &msg, info);
            }
        });
        count = 0;
        vd.field_validated_key_blocks("random_on_action", |key, b, data| {
            let mut vd = Validator::new(b, data);
            vd.field_numeric("chance_to_happen"); // TODO: 0 - 100
            vd.field_script_value("chance_of_no_event", &mut sc);
            for (_key, token) in vd.integer_values() {
                if token.is("0") {
                    continue;
                }
                data.verify_exists(Item::OnAction, token);
            }
            count += 1;
            if count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try putting each into its own on_action and firing those separately";
                warn_info(key, ErrorKey::Validation, &msg, info);
            }
        });
        count = 0;
        vd.field_validated_key_blocks("first_valid_on_action", |key, b, data| {
            let mut vd = Validator::new(b, data);
            for token in vd.values() {
                data.verify_exists(Item::OnAction, token);
            }
            count += 1;
            if count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try putting each into its own on_action and firing those separately";
                warn_info(key, ErrorKey::Validation, &msg, info);
            }
        });
        vd.field_validated_block("effect", |b, data| {
            validate_normal_effect(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_item("fallback", Item::OnAction);
    }
}

/// LAST UPDATED VERSION 1.9.0.4
const ON_ACTION_SCOPES: &[(&str, Scopes, &[(&str, Scopes)])] = &[
    // TODO Special:
    // `<relation>_quarterly_pulse`
    // `on_set_relation_<relation>`
    // `on_remove_relation_<relation>`
    // `on_death_relation_<relation>`
    ("on_artifact_changed_owner", Scopes::Artifact, &[]),
    ("on_artifact_succession", Scopes::Artifact, &[]),
    ("on_artifact_broken_through_decay", Scopes::Artifact, &[]),
    ("on_artifact_broken_through_effect", Scopes::Artifact, &[]),
    ("on_artifact_claim_gained", Scopes::Character, &[]),
    ("on_artifact_claim_lost", Scopes::Character, &[]),
    ("on_artifact_durability_low", Scopes::Character, &[]), // TODO: verify the doc here
    ("on_artifact_durability_very_low", Scopes::Artifact, &[]),
    ("yearly_struggle_playable_pulse", Scopes::Character, &[]),
    ("five_year_struggle_playable_pulse", Scopes::Character, &[]),
    ("on_building_completed", Scopes::Province, &[]),
    ("on_army_monthly", Scopes::Army, &[]),
    ("on_county_occupied", Scopes::Character, &[]),
    ("on_siege_completion", Scopes::Character, &[]),
    ("on_siege_looting", Scopes::Character, &[]),
    ("on_army_enter_province", Scopes::Character, &[]),
    ("on_raid_action_start", Scopes::Army, &[]),
    ("on_raid_action_completion", Scopes::Army, &[]),
    ("on_raid_action_weekly", Scopes::Army, &[]),
    ("on_raid_loot_delivered", Scopes::Army, &[]),
    ("on_defeat_raid_army", Scopes::Army, &[]),
    ("on_hook_used", Scopes::Character, &[]),
    ("on_trigger_court_events", Scopes::Character, &[]),
    ("on_absent_from_royal_court", Scopes::Character, &[]),
    ("on_imprison", Scopes::Character, &[]),
    ("on_release_from_prison", Scopes::Character, &[]),
    ("on_faith_created", Scopes::Character, &[]),
    ("on_faith_conversion", Scopes::Character, &[]),
    ("on_character_faith_change", Scopes::Character, &[]),
    ("on_faith_monthly", Scopes::Faith, &[]),
    (
        "on_potential_great_holy_war_invalidation",
        Scopes::GreatHolyWar,
        &[],
    ),
    ("on_great_holy_war_invalidation", Scopes::GreatHolyWar, &[]),
    ("on_great_holy_war_countdown_end", Scopes::GreatHolyWar, &[]),
    (
        "on_great_holy_war_participant_replaced",
        Scopes::Character,
        &[],
    ),
    ("on_join_court", Scopes::Character, &[]),
    ("on_leave_court", Scopes::Character, &[]),
    ("on_county_faith_change", Scopes::LandedTitle, &[]),
    ("on_county_culture_change", Scopes::LandedTitle, &[]),
    ("on_war_transferred", Scopes::Character, &[]),
    ("on_join_war_as_secondary", Scopes::Character, &[]),
    // TODO figure out the root for these
    //    ("on_war_started" , &[]),
    //    ("on_war_won_attacker" , &[]),
    //    ("on_war_won_defender" , &[]),
    //    ("on_war_white_peace" , &[]),
    //    ("on_war_invalidated" , &[]),
    ("on_title_destroyed", Scopes::Character, &[]),
    ("on_title_gain", Scopes::Character, &[]),
    ("on_title_gain_inheritance", Scopes::Character, &[]),
    ("on_title_gain_usurpation", Scopes::Character, &[]),
    ("on_title_lost", Scopes::Character, &[]),
    ("on_explicit_claim_gain", Scopes::Character, &[]),
    ("on_explicit_claim_lost", Scopes::Character, &[]),
    ("on_rank_up", Scopes::Character, &[]),
    ("on_rank_down", Scopes::Character, &[]),
    ("on_vassal_gained", Scopes::Character, &[]),
    (
        "on_baron_found_or_created_for_title",
        Scopes::Character,
        &[],
    ),
    ("on_holy_order_new_lease", Scopes::HolyOrder, &[]),
    ("on_holy_order_hired", Scopes::HolyOrder, &[]),
    ("on_holy_order_destroyed", Scopes::Faith, &[]),
    ("on_perks_refunded", Scopes::Character, &[]),
    ("on_ruler_designer_finished", Scopes::Character, &[]),
    ("on_alliance_added", Scopes::None, &[]),
    ("on_alliance_removed", Scopes::None, &[]),
    ("on_alliance_broken", Scopes::None, &[]),
    ("on_dynasty_created", Scopes::Dynasty, &[]), // undocumented
    (
        "on_courtier_decided_to_move_to_pool",
        Scopes::Character,
        &[],
    ),
    ("on_courtier_ready_to_move_to_pool", Scopes::Character, &[]),
    ("on_guest_arrived_from_pool", Scopes::Character, &[]),
    ("on_guest_ready_to_move_to_pool", Scopes::Character, &[]),
    ("on_accolade_rank_change", Scopes::Accolade, &[]),
    ("on_accolade_glory_change", Scopes::Accolade, &[]),
    ("on_accolade_created", Scopes::Accolade, &[]),
    ("on_active_accolade_succession", Scopes::Accolade, &[]),
    ("on_inactive_accolade_succession", Scopes::Accolade, &[]),
    ("on_accolade_acclaimed_death", Scopes::Accolade, &[]),
    ("on_accolade_acclaimed_removal", Scopes::Accolade, &[]),
    ("on_accolade_successor_death", Scopes::Accolade, &[]),
    ("on_accolade_successor_removal", Scopes::Accolade, &[]),
    ("on_accolade_deactivated", Scopes::Accolade, &[]),
    ("on_accolade_new_acclaimed_knight", Scopes::Accolade, &[]),
    ("on_combat_end_winner", Scopes::CombatSide, &[]),
    ("on_combat_end_loser", Scopes::CombatSide, &[]),
    ("on_marriage", Scopes::Character, &[]),
    ("on_divorce", Scopes::Character, &[]),
    ("on_concubinage", Scopes::Character, &[]),
    ("on_concubinage_end", Scopes::Character, &[]),
    ("on_betrothal_broken", Scopes::Character, &[]),
    ("on_game_start", Scopes::None, &[]),
    ("on_game_start_after_lobby", Scopes::None, &[]),
    ("on_councillor_left", Scopes::Character, &[]),
    ("on_stress_level_reduced", Scopes::Character, &[]),
    ("on_stress_level_1", Scopes::Character, &[]),
    ("on_stress_level_2", Scopes::Character, &[]),
    ("on_stress_level_3", Scopes::Character, &[]),
    ("on_stress_level_4", Scopes::Character, &[]),
    ("on_game_start_with_tutorial", Scopes::None, &[]),
    ("on_court_language_changed", Scopes::Character, &[]),
    ("on_commander_combat_finished", Scopes::Character, &[]),
    ("on_army_combat_finished", Scopes::Character, &[]),
    ("on_entered_diarchy", Scopes::Character, &[]),
    ("on_left_diarchy", Scopes::Character, &[]),
    ("on_diarch_change", Scopes::Character, &[]),
    ("on_diarch_designation", Scopes::Character, &[]),
    ("on_birth_mother", Scopes::Character, &[]),
    ("on_birth_father", Scopes::Character, &[]),
    ("on_birth_real_father", Scopes::Character, &[]),
    ("on_birth_child", Scopes::Character, &[]),
    ("on_pregnancy_mother", Scopes::Character, &[]),
    ("on_pregnancy_father", Scopes::Character, &[]),
    ("on_pregnancy_ended_mother", Scopes::Character, &[]),
    ("yearly_global_pulse", Scopes::None, &[]),
    ("yearly_playable_pulse", Scopes::Character, &[]),
    ("three_year_playable_pulse", Scopes::Character, &[]),
    ("five_year_playable_pulse", Scopes::Character, &[]),
    ("quarterly_playable_pulse", Scopes::Character, &[]),
    ("random_yearly_playable_pulse", Scopes::Character, &[]),
    ("random_yearly_everyone_pulse", Scopes::Character, &[]),
    ("five_year_everyone_pulse", Scopes::Character, &[]),
    ("three_year_pool_pulse", Scopes::Character, &[]),
    ("on_birthday", Scopes::Character, &[]),
    ("yearly_culture_pulse", Scopes::Culture, &[]),
    ("three_yearly_culture_pulse", Scopes::Culture, &[]),
    ("on_culture_era_changed", Scopes::Culture, &[]),
    ("on_character_culture_change", Scopes::Character, &[]),
    ("on_tradition_removed", Scopes::Culture, &[]),
    ("on_tradition_added", Scopes::Culture, &[]),
    ("on_culture_created", Scopes::Culture, &[]),
    (
        "on_county_auto_granted_to_liege_culture",
        Scopes::Culture,
        &[],
    ),
    (
        "on_county_auto_granted_to_local_culture",
        Scopes::Culture,
        &[],
    ),
    ("on_death", Scopes::Character, &[]),
    ("on_natural_death_second_chance", Scopes::Character, &[]),
    ("on_travel_plan_movement", Scopes::Character, &[]),
    ("on_travel_plan_arrival", Scopes::Character, &[]),
    ("on_travel_plan_start", Scopes::Character, &[]),
    ("on_travel_plan_complete", Scopes::Character, &[]),
    ("on_travel_plan_abort", Scopes::Character, &[]),
    ("on_travel_plan_cancel", Scopes::Character, &[]),
    // TODO figure out the root for these
    // ("on_travel_activity_complete",
    // ("on_travel_activity_invalidated",
    ("on_travel_leader_removed", Scopes::Character, &[]),
    ("on_court_type_changed", Scopes::Character, &[]),
    ("on_player_royal_court_first_gained", Scopes::Character, &[]),
    ("on_court_grandeur_level_changed", Scopes::Character, &[]),
];
