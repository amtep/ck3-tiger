use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_cost, validate_duration};

#[derive(Clone, Debug)]
pub struct CasusBelli {}

impl CasusBelli {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CasusBelli, key, block, Box::new(Self {}));
    }
}

impl DbKind for CasusBelli {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);

        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::CasusBelli, key);
        sc.define_name("attacker", Scopes::Character, key);
        sc.define_name("defender", Scopes::Character, key);

        if block.has_key("is_allowed_claim_title") {
            sc.define_name("claimant", Scopes::Character, key);
        }

        vd.field_item("group", Item::CasusBelliGroup);
        let icon = vd.field_value("icon").unwrap_or(key);
        if let Some(icon_path) =
            data.get_defined_string_warn(key, "NGameIcons|CASUS_BELLI_TYPE_ICON_PATH")
        {
            let path = format!("{icon_path}/{icon}.dds");
            data.verify_exists_implied(Item::File, &path, icon);
        }

        vd.field_validated_block_rooted(
            "attacker_ticking_warscore_delay",
            Scopes::Character,
            validate_duration,
        );
        vd.field_validated_block_rooted(
            "defender_ticking_warscore_delay",
            Scopes::Character,
            validate_duration,
        );
        vd.field_numeric("attacker_ticking_warscore");
        vd.field_numeric("defender_ticking_warscore");
        vd.field_numeric_range("attacker_wargoal_percentage", 0.0, 1.0);
        vd.field_numeric_range("defender_wargoal_percentage", 0.0, 1.0);
        vd.field_numeric("attacker_score_from_occupation_scale");
        vd.field_numeric("defender_score_from_occupation_scale");
        vd.field_numeric("attacker_score_from_battles_scale");
        vd.field_numeric("defender_score_from_battles_scale");
        vd.field_numeric("max_attacker_score_from_battles");
        vd.field_numeric("max_defender_score_from_battles");
        vd.field_numeric("max_attacker_score_from_occupation");
        vd.field_numeric("max_defender_score_from_occupation");
        vd.field_bool("full_occupation_by_defender_gives_victory");
        vd.field_bool("full_occupation_by_attacker_gives_victory");
        vd.field_bool("landless_attacker_needs_armies");

        vd.field_numeric("occupation_participation_mult");
        vd.field_numeric("siege_participation_mult");
        vd.field_numeric("battle_participation_mult");

        vd.field_validated_block_sc("cost", &mut sc, validate_cost);

        vd.field_bool("attacker_capital_gives_war_score");
        vd.field_bool("defender_capital_gives_war_score");
        vd.field_bool("imprisonment_by_attacker_give_war_score");
        vd.field_bool("imprisonment_by_defender_give_war_score");

        for field in
            &["on_declaration", "on_victory", "on_white_peace", "on_defeat", "on_invalidated"]
        {
            vd.field_validated_block(field, |block, data| {
                // TODO: check which are tooltipped
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
        }

        for field in
            &["on_victory_desc", "on_defeat_desc", "on_white_peace_desc", "on_invalidated_desc"]
        {
            vd.field_validated_sc(field, &mut sc, validate_desc);
        }

        vd.field_validated_block("should_invalidate", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("mutually_exclusive_titles", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_bool("combine_into_one");

        vd.field_validated_key_block("allowed_for_character", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("attacker", Scopes::Character, key);
            sc.define_name("defender", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block(
            "allowed_for_character_display_regardless",
            |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                sc.define_name("attacker", Scopes::Character, key);
                sc.define_name("defender", Scopes::Character, key);
                validate_trigger(block, data, &mut sc, Tooltipped::Yes);
            },
        );

        vd.field_validated_key_block("allowed_against_character", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("attacker", Scopes::Character, key);
            sc.define_name("defender", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block(
            "allowed_against_character_display_regardless",
            |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                sc.define_name("attacker", Scopes::Character, key);
                sc.define_name("defender", Scopes::Character, key);
                validate_trigger(block, data, &mut sc, Tooltipped::Yes);
            },
        );

        vd.field_validated_key_block("valid_to_start", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("attacker", Scopes::Character, key);
            sc.define_name("defender", Scopes::Character, key);
            sc.define_name("target", Scopes::LandedTitle, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("valid_to_start_display_regardless", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("attacker", Scopes::Character, key);
            sc.define_name("defender", Scopes::Character, key);
            sc.define_name("target", Scopes::LandedTitle, key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_key_block("is_allowed_claim_title", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
            sc.define_name("attacker", Scopes::Character, key);
            sc.define_name("defender", Scopes::Character, key);
            sc.define_name("claimant", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        // "claim" and "de_jure" choices are undocumented
        let choices = &[
            "none",
            "neighbor_land",
            "neighbor_water",
            "neighbor_land_or_water",
            "de_jure_claim",
            "title_claim",
            "all",
            "claim",
            "de_jure",
            "independence_domain", // undocumented
        ];
        vd.field_choice("target_titles", choices);
        let choices = &["all", "barony", "county", "duchy", "kingdom", "empire"];
        vd.field_choice("target_title_tier", choices);
        vd.field_bool("target_de_jure_regions_above");
        vd.field_bool("use_de_jure_wargoal_only");

        // undocumented
        vd.field_item("cb_name", Item::Localization);
        vd.field_item("cb_name_no_target", Item::Localization);

        vd.field_item("war_name", Item::Localization);
        vd.field_item("my_war_name", Item::Localization);
        vd.field_item("war_name_base", Item::Localization);
        vd.field_item("my_war_name_base", Item::Localization);

        vd.field_integer("truce_days"); // not used in vanilla

        vd.field_values("ignore_effect"); // TODO

        let choices = &["invalidate", "inherit", "inherit_faction"];
        vd.field_choice("on_primary_attacker_death", choices);
        vd.field_choice("on_primary_defender_death", choices);
        vd.field_choice("transfer_behavior", &["invalidate", "transfer"]);
        vd.field_bool("check_attacker_inheritance_validity");
        vd.field_bool("check_defender_inheritance_validity");
        vd.field_bool("attacker_allies_inherit");
        vd.field_bool("defender_allies_inherit");

        vd.field_integer("interface_priority");
        vd.field_integer("max_ai_diplo_distance_to_title");
        vd.field_bool("ai_only_against_liege");
        vd.field_bool("ai_only_against_neighbors");
        vd.field_validated_block_rooted(
            "ai_can_target_all_titles",
            Scopes::Character,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_bool("ai");

        vd.field_bool("white_peace_possible");
        vd.field_bool("check_all_defenders_for_ticking_war_score");
        vd.field_bool("ticking_war_score_targets_entire_realm");

        vd.field_bool("gui_attacker_faith_might_join");
        vd.field_bool("gui_defender_faith_might_join");
        vd.field_bool("defender_faith_can_join");
        vd.field_bool("is_great_holy_war");
        vd.field_bool("is_holy_war"); // undocumented
        vd.field_bool("target_top_liege_if_outside_realm");
        vd.field_bool("should_check_for_interface_availability");

        vd.field_script_value("ai_score", &mut sc);
        vd.field_script_value("ai_score_mult", &mut sc);

        // undocumented
        vd.field_bool("should_show_war_goal_subview");
    }
}

#[derive(Clone, Debug)]
pub struct CasusBelliGroup {}

impl CasusBelliGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CasusBelliGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for CasusBelliGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_rooted(
            "allowed_for_character",
            Scopes::Character,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_bool("should_check_for_interface_availability");
        vd.field_bool("can_only_start_via_script");
    }
}
