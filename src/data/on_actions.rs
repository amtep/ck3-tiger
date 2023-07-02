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
        sc.set_strict_scopes(false);
        for (name, root) in ON_ACTION_SCOPES {
            if self.key.is(name) {
                sc = ScopeContext::new(*root, &self.key);
                set_additional_scopes(&mut sc, &self.key);
                sc.set_strict_scopes(true);
                break;
            }
        }
        if let Some(relation) = self.key.as_str().strip_suffix("_quarterly_pulse") {
            if data.item_exists(Item::Relation, relation) {
                sc.define_name("quarter", Scopes::Value, &self.key); // undocumented
                sc.set_strict_scopes(true);
            }
        } else {
            for pfx in &[
                "on_set_relation_",
                "on_remove_relation_",
                "on_death_relation_",
            ] {
                if let Some(relation) = self.key.as_str().strip_prefix(pfx) {
                    data.verify_exists_implied(Item::Relation, relation, &self.key);
                    sc.define_name("target", Scopes::Character, &self.key); // undocumented
                    sc.set_strict_scopes(true);
                }
            }
        }
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

/// LAST UPDATED VERSION 1.9.2.1
fn set_additional_scopes(sc: &mut ScopeContext, name: &Token) {
    match name.as_str() {
        "on_accolade_rank_change" => sc.define_name("positive", Scopes::Bool, name),
        "on_accolade_glory_change" => sc.define_name("glory", Scopes::Value, name),
        "on_active_accolade_succession" | "on_inactive_accolade_succession" => {
            sc.define_name("new_owner", Scopes::Character, name)
        }
        "on_accolade_acclaimed_death" | "on_accolade_acclaimed_removal" => {
            sc.define_name("old_acclaimed_knight", Scopes::Character, name);
            sc.define_name("new_accolade_type", Scopes::Bool, name);
        }
        "on_accolade_deactivated" => sc.define_name("owner", Scopes::Character, name),
        "on_accolade_new_acclaimed_knight" => {
            sc.define_name("glory", Scopes::Value, name);
            sc.define_name("new_accolade_type", Scopes::Bool, name);
            sc.define_name("new_acclaimed_knight", Scopes::Character, name);
        }

        "on_alliance_added" | "on_alliance_removed" => {
            sc.define_name("first", Scopes::Character, name);
            sc.define_name("second", Scopes::Character, name);
        }
        "on_alliance_broken" => {
            sc.define_name("first", Scopes::Character, name);
            sc.define_name("second", Scopes::Character, name);
            sc.define_list("first", Scopes::Character, name);
            sc.define_list("second", Scopes::Character, name);
        }

        "on_army_monthly" | "on_army_enter_province" => sc.define_name("army", Scopes::Army, name),
        "on_county_occupied" | "on_siege_completion" => {
            sc.define_name("county", Scopes::LandedTitle, name);
            sc.define_name("barony", Scopes::LandedTitle, name);
            sc.define_name("previous_controller", Scopes::Character, name);
            sc.define_name("war", Scopes::War, name);
            if name.is("on_siege_completion") {
                sc.define_list("occupied_baronies", Scopes::LandedTitle, name);
            }
        }
        "on_siege_looting" => {
            sc.define_name("county", Scopes::LandedTitle, name);
            sc.define_name("barony", Scopes::LandedTitle, name);
            sc.define_name("previous_controller", Scopes::Character, name);
        }
        "on_raid_action_start" | "on_raid_action_completion" | "on_raid_action_weekly" => {
            sc.define_name("raider", Scopes::Character, name);
            sc.define_name("barony", Scopes::LandedTitle, name);
            sc.define_name("county", Scopes::LandedTitle, name);
        }
        "on_raid_loot_delivered" => sc.define_name("raider", Scopes::Character, name),
        "on_defeat_raid_army" => {
            sc.define_name("raider", Scopes::Character, name);
            sc.define_name("receiver", Scopes::Character, name);
        }

        "on_birth_mother" | "on_birth_father" | "on_birth_real_father" | "on_birth_child" => {
            sc.define_name("child", Scopes::Character, name);
            sc.define_name("mother", Scopes::Character, name);
            sc.define_name("real_father", Scopes::Character, name);
            sc.define_name("father", Scopes::Character, name);
            if name.is("on_birth_child") {
                sc.define_name("is_bastard", Scopes::Bool, name);
            }
        }
        "on_pregnancy_mother" | "on_pregnancy_father" | "on_pregnancy_ended_mother" => {
            sc.define_name("mother", Scopes::Character, name);
            sc.define_name("real_father", Scopes::Character, name);
            sc.define_name("father", Scopes::Character, name);
        }

        "on_combat_end_winner" | "on_combat_end_loser" => {
            sc.define_name("wipe", Scopes::Bool, name)
        }

        "on_councillor_left" => {
            sc.define_name("old_employer", Scopes::Character, name);
            sc.define_name("council_task", Scopes::CouncilTask, name);
            sc.define_name("councillor", Scopes::Character, name);
        }

        "on_county_faith_change" => sc.define_name("old_faith", Scopes::Faith, name),
        "on_county_culture_change" => sc.define_name("old_culture", Scopes::Culture, name),

        "on_absent_from_royal_court" => sc.define_name("value", Scopes::Value, name),
        "on_court_grandeur_level_changed" => {
            sc.define_name("old_value", Scopes::Value, name);
            sc.define_name("new_value", Scopes::Value, name);
        }

        "on_courtier_decided_to_move_to_pool" | "on_courtier_ready_to_move_to_pool" => {
            sc.define_name("courtier", Scopes::Character, name);
            sc.define_name("liege", Scopes::Character, name);
            sc.define_list("characters", Scopes::Character, name);
        }
        "on_guest_arrived_from_pool" | "on_guest_ready_to_move_to_pool" => {
            sc.define_name("guest", Scopes::Character, name);
            sc.define_name("host", Scopes::Character, name);
            sc.define_list("characters", Scopes::Character, name);
            if name.is("on_guest_ready_to_move_to_pool") {
                sc.define_name("destination", Scopes::Province, name); // TODO: verify
            }
        }
        "on_join_court" => {
            sc.define_name("new_employer", Scopes::Character, name);
            sc.define_name("old_employer", Scopes::Character, name); // may be unset
        }
        "on_leave_court" => sc.define_name("old_employer", Scopes::Character, name),

        "on_tradition_removed" | "on_tradition_added" => {
            sc.define_name("tradition", Scopes::Flag, name)
        } // TODO: verify scope type
        "on_culture_created" => sc.define_name("founder", Scopes::Character, name),

        "on_county_auto_granted_to_liege_culture" | "on_county_auto_granted_to_local_culture" => {
            sc.define_name("actor", Scopes::Character, name);
            sc.define_name("landed_title", Scopes::LandedTitle, name);
        }

        "on_death" => sc.define_name("killer", Scopes::Character, name), // may be unset

        "on_entered_diarchy" => sc.define_name("reason", Scopes::Flag, name),
        "on_left_diarchy" => sc.define_name("old_diarch", Scopes::Character, name),
        "on_diarch_change" => {
            sc.define_name("reason", Scopes::Flag, name);
            sc.define_name("old_diarch", Scopes::Character, name);
        }
        "on_diarch_designation" => sc.define_name(
            "former_designated_diarch",
            Scopes::Character | Scopes::None,
            name,
        ),

        "on_holy_order_new_lease" => {
            sc.define_name("patron", Scopes::Character, name);
            sc.define_name("barony", Scopes::LandedTitle, name);
        }
        "on_holy_order_hired" => {
            sc.define_name("patron", Scopes::Character, name);
            sc.define_name("actor", Scopes::Character, name);
        }
        "on_holy_order_destroyed" => {
            sc.define_name("title", Scopes::LandedTitle, name);
            sc.define_name("leader", Scopes::Character, name);
        }

        "on_hook_used" => sc.define_name("target", Scopes::Character, name),

        "on_artifact_changed_owner" | "on_artifact_succession" => {
            sc.define_name("owner", Scopes::Character, name);
            sc.define_name("old_owner", Scopes::Character, name);
            if name.is("on_artifact_succession") {
                sc.define_name("old_primary", Scopes::Character, name);
            }
        }
        // TODO: docs say that on_artifact_durability_low is different from all of these. Verify.
        "on_artifact_broken_through_decay"
        | "on_artifact_broken_through_effect"
        | "on_artifact_durability_very_low" => sc.define_name("owner", Scopes::Character, name),
        "on_artifact_claim_gained" | "on_artifact_claim_lost" => {
            sc.define_name("owner", Scopes::Character, name);
            sc.define_name("artifact", Scopes::Artifact, name);
        }

        "on_commander_combat_finished" | "on_army_combat_finished" => {
            sc.define_name("combat_side", Scopes::CombatSide, name);
            sc.define_name("victory", Scopes::Bool, name);
            if name.is("on_army_combat_finished") {
                sc.define_list("commanders", Scopes::Character, name);
                sc.define_list("knights", Scopes::Character, name);
            }
        }

        "on_marriage" => sc.define_name("spouse", Scopes::Character, name),
        "on_divorce" => {
            sc.define_name("spouse", Scopes::Character, name);
            sc.define_name("reason", Scopes::Flag, name);
        }
        "on_concubinage" => sc.define_name("concubine", Scopes::Character, name),
        "on_concubinage_end" => {
            sc.define_name("concubine", Scopes::Character, name);
            sc.define_name("reason", Scopes::Flag, name);
        }
        "on_betrothal_broken" => {
            sc.define_name("second", Scopes::Character, name);
            sc.define_name("reason", Scopes::Flag, name);
        }

        "on_imprison" | "on_release_from_prison" => {
            sc.define_name("imprisoner", Scopes::Character, name)
        }

        "on_faith_created" | "on_faith_conversion" | "on_character_faith_change" => {
            sc.define_name("old_faith", Scopes::Faith, name)
        }
        "on_great_holy_war_participant_replaced" => {
            sc.define_name("great_holy_war", Scopes::GreatHolyWar, name);
            sc.define_name("replacement", Scopes::Character, name);
        }

        "yearly_struggle_playable_pulse" | "five_year_struggle_playable_pulse" => {
            sc.define_name("struggle", Scopes::Struggle, name)
        }

        "on_title_destroyed" => sc.define_name("landed_title", Scopes::LandedTitle, name),
        "on_title_gain" | "on_title_gain_inheritance" | "on_title_gain_usurpation" => {
            sc.define_name("title", Scopes::LandedTitle, name);
            sc.define_name("previous_holder", Scopes::Character, name);
            sc.define_name("transfer_type", Scopes::Flag, name);
        }
        "on_title_lost" => {
            sc.define_name("title", Scopes::LandedTitle, name);
            sc.define_name("new_holder", Scopes::Character, name);
            sc.define_name("transfer_type", Scopes::Flag, name);
        }
        "on_explicit_claim_gain" => {
            sc.define_name("title", Scopes::LandedTitle, name);
            sc.define_name("transfer_type", Scopes::Flag | Scopes::None, name);
        }
        "on_explicit_claim_lost" | "on_rank_up" | "on_rank_down" => {
            sc.define_name("title", Scopes::LandedTitle, name)
        }
        "on_vassal_gained" => {
            sc.define_name("vassal", Scopes::Character, name);
            sc.define_name("old_liege", Scopes::Character, name); // may be null character
            sc.define_name("transfer_type", Scopes::Flag | Scopes::None, name);
        }
        "on_baron_found_or_created_for_title" => {
            sc.define_name("liege", Scopes::Character, name);
            sc.define_name("title", Scopes::LandedTitle, name);
        }

        "on_travel_activity_arrival_too_late" => {
            sc.define_name("travel_plan", Scopes::TravelPlan, name)
        }
        "on_travel_activity_estimated_arrival_too_late" => {
            sc.define_name("travel_plan", Scopes::TravelPlan, name);
            sc.define_name("estimated_arrival_diff_days", Scopes::Value, name);
        }
        "on_travel_leader_removed" => {
            sc.define_name("travel_plan", Scopes::TravelPlan, name);
            sc.define_name("old_travel_leader", Scopes::Character, name);
        }

        "on_war_transferred" => {
            sc.define_name("war", Scopes::War, name);
            sc.define_name("defender", Scopes::Character, name);
        }
        "on_join_war_as_secondary" => sc.define_name("war", Scopes::War, name),
        "on_war_started"
        | "on_war_won_attacker"
        | "on_war_won_defender"
        | "on_war_white_peace"
        | "on_war_invalidated" => {
            sc.define_name("attacker", Scopes::Character, name);
            sc.define_name("defender", Scopes::Character, name);
            sc.define_name("claimant", Scopes::Character, name); // might not be defined
            sc.define_name("war", Scopes::War, name); // undocumented
        }

        _ => {}
    };
}

/// LAST UPDATED VERSION 1.9.0.4
const ON_ACTION_SCOPES: &[(&str, Scopes)] = &[
    ("on_artifact_changed_owner", Scopes::Artifact),
    ("on_artifact_succession", Scopes::Artifact),
    ("on_artifact_broken_through_decay", Scopes::Artifact),
    ("on_artifact_broken_through_effect", Scopes::Artifact),
    ("on_artifact_claim_gained", Scopes::Character),
    ("on_artifact_claim_lost", Scopes::Character),
    ("on_artifact_durability_low", Scopes::Character), // TODO: verify the doc here
    ("on_artifact_durability_very_low", Scopes::Artifact),
    ("yearly_struggle_playable_pulse", Scopes::Character),
    ("five_year_struggle_playable_pulse", Scopes::Character),
    ("on_building_completed", Scopes::Province),
    ("on_army_monthly", Scopes::Character),
    ("on_county_occupied", Scopes::Character),
    ("on_siege_completion", Scopes::Character),
    ("on_siege_looting", Scopes::Character),
    ("on_army_enter_province", Scopes::Character),
    ("on_raid_action_start", Scopes::Army),
    ("on_raid_action_completion", Scopes::Army),
    ("on_raid_action_weekly", Scopes::Army),
    ("on_raid_loot_delivered", Scopes::Army),
    ("on_defeat_raid_army", Scopes::Army),
    ("on_hook_used", Scopes::Character),
    ("on_trigger_court_events", Scopes::Character),
    ("on_absent_from_royal_court", Scopes::Character),
    ("on_imprison", Scopes::Character),
    ("on_release_from_prison", Scopes::Character),
    ("on_faith_created", Scopes::Character),
    ("on_faith_conversion", Scopes::Character),
    ("on_character_faith_change", Scopes::Character),
    ("on_faith_monthly", Scopes::Faith),
    (
        "on_potential_great_holy_war_invalidation",
        Scopes::GreatHolyWar,
    ),
    ("on_great_holy_war_invalidation", Scopes::GreatHolyWar),
    ("on_great_holy_war_countdown_end", Scopes::GreatHolyWar),
    ("on_great_holy_war_participant_replaced", Scopes::Character),
    ("on_join_court", Scopes::Character),
    ("on_leave_court", Scopes::Character),
    ("on_county_faith_change", Scopes::LandedTitle),
    ("on_county_culture_change", Scopes::LandedTitle),
    ("on_war_transferred", Scopes::Character),
    ("on_join_war_as_secondary", Scopes::Character),
    ("on_war_started", Scopes::CasusBelli),
    ("on_war_won_attacker", Scopes::CasusBelli),
    ("on_war_won_defender", Scopes::CasusBelli),
    ("on_war_white_peace", Scopes::CasusBelli),
    ("on_war_invalidated", Scopes::CasusBelli),
    ("on_title_destroyed", Scopes::Character),
    ("on_title_gain", Scopes::Character),
    ("on_title_gain_inheritance", Scopes::Character),
    ("on_title_gain_usurpation", Scopes::Character),
    ("on_title_lost", Scopes::Character),
    ("on_explicit_claim_gain", Scopes::Character),
    ("on_explicit_claim_lost", Scopes::Character),
    ("on_rank_up", Scopes::Character),
    ("on_rank_down", Scopes::Character),
    ("on_vassal_gained", Scopes::Character),
    ("on_baron_found_or_created_for_title", Scopes::Character),
    ("on_holy_order_new_lease", Scopes::HolyOrder),
    ("on_holy_order_hired", Scopes::HolyOrder),
    ("on_holy_order_destroyed", Scopes::Faith),
    ("on_perks_refunded", Scopes::Character),
    ("on_ruler_designer_finished", Scopes::Character),
    ("on_alliance_added", Scopes::None),
    ("on_alliance_removed", Scopes::None),
    ("on_alliance_broken", Scopes::None),
    ("on_dynasty_created", Scopes::Dynasty), // undocumented
    ("on_courtier_decided_to_move_to_pool", Scopes::Character),
    ("on_courtier_ready_to_move_to_pool", Scopes::Character),
    ("on_guest_arrived_from_pool", Scopes::Character),
    ("on_guest_ready_to_move_to_pool", Scopes::Character),
    ("on_accolade_rank_change", Scopes::Accolade),
    ("on_accolade_glory_change", Scopes::Accolade),
    ("on_accolade_created", Scopes::Accolade),
    ("on_active_accolade_succession", Scopes::Accolade),
    ("on_inactive_accolade_succession", Scopes::Accolade),
    ("on_accolade_acclaimed_death", Scopes::Accolade),
    ("on_accolade_acclaimed_removal", Scopes::Accolade),
    ("on_accolade_successor_death", Scopes::Accolade),
    ("on_accolade_successor_removal", Scopes::Accolade),
    ("on_accolade_deactivated", Scopes::Accolade),
    ("on_accolade_new_acclaimed_knight", Scopes::Accolade),
    ("on_combat_end_winner", Scopes::CombatSide),
    ("on_combat_end_loser", Scopes::CombatSide),
    ("on_marriage", Scopes::Character),
    ("on_divorce", Scopes::Character),
    ("on_concubinage", Scopes::Character),
    ("on_concubinage_end", Scopes::Character),
    ("on_betrothal_broken", Scopes::Character),
    ("on_game_start", Scopes::None),
    ("on_game_start_after_lobby", Scopes::None),
    ("on_councillor_left", Scopes::Character),
    ("on_stress_level_reduced", Scopes::Character),
    ("on_stress_level_1", Scopes::Character),
    ("on_stress_level_2", Scopes::Character),
    ("on_stress_level_3", Scopes::Character),
    ("on_stress_level_4", Scopes::Character),
    ("on_game_start_with_tutorial", Scopes::None),
    ("on_court_language_changed", Scopes::Character),
    ("on_commander_combat_finished", Scopes::Character),
    ("on_army_combat_finished", Scopes::Character),
    ("on_entered_diarchy", Scopes::Character),
    ("on_left_diarchy", Scopes::Character),
    ("on_diarch_change", Scopes::Character),
    ("on_diarch_designation", Scopes::Character),
    ("on_birth_mother", Scopes::Character),
    ("on_birth_father", Scopes::Character),
    ("on_birth_real_father", Scopes::Character),
    ("on_birth_child", Scopes::Character),
    ("on_pregnancy_mother", Scopes::Character),
    ("on_pregnancy_father", Scopes::Character),
    ("on_pregnancy_ended_mother", Scopes::Character),
    ("yearly_global_pulse", Scopes::None),
    ("yearly_playable_pulse", Scopes::Character),
    ("three_year_playable_pulse", Scopes::Character),
    ("five_year_playable_pulse", Scopes::Character),
    ("quarterly_playable_pulse", Scopes::Character),
    ("random_yearly_playable_pulse", Scopes::Character),
    ("random_yearly_everyone_pulse", Scopes::Character),
    ("five_year_everyone_pulse", Scopes::Character),
    ("three_year_pool_pulse", Scopes::Character),
    ("on_birthday", Scopes::Character),
    ("yearly_culture_pulse", Scopes::Culture),
    ("three_yearly_culture_pulse", Scopes::Culture),
    ("on_culture_era_changed", Scopes::Culture),
    ("on_character_culture_change", Scopes::Character),
    ("on_tradition_removed", Scopes::Culture),
    ("on_tradition_added", Scopes::Culture),
    ("on_culture_created", Scopes::Culture),
    ("on_county_auto_granted_to_liege_culture", Scopes::Culture),
    ("on_county_auto_granted_to_local_culture", Scopes::Culture),
    ("on_death", Scopes::Character),
    ("on_natural_death_second_chance", Scopes::Character),
    ("on_travel_plan_movement", Scopes::Character),
    ("on_travel_plan_arrival", Scopes::Character),
    ("on_travel_plan_start", Scopes::Character),
    ("on_travel_plan_complete", Scopes::Character),
    ("on_travel_plan_abort", Scopes::Character),
    ("on_travel_plan_cancel", Scopes::Character),
    // TODO figure out the root for these
    // ("on_travel_activity_complete",
    // ("on_travel_activity_invalidated",
    ("on_travel_leader_removed", Scopes::Character),
    ("on_court_type_changed", Scopes::Character),
    ("on_player_royal_court_first_gained", Scopes::Character),
    ("on_court_grandeur_level_changed", Scopes::Character),
];
