use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CouncilPosition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CouncilPosition, CouncilPosition::add)
}

impl CouncilPosition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CouncilPosition, key, block, Box::new(Self {}));
    }
}

impl DbKind for CouncilPosition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("councillor", Scopes::Character, key);
        sc.define_name("councillor_liege", Scopes::Character, key);

        // The base key has to exist even if name = a triggered desc
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_possessive");
        data.verify_exists_implied(Item::Localization, &loca, key);
        vd.field_validated_sc("name", &mut sc, validate_desc);
        vd.field_validated_sc("tooltip", &mut sc, validate_desc);

        vd.field_item("skill", Item::Skill);
        vd.field_validated_sc("auto_fill", &mut sc, validate_yes_no_trigger);
        vd.field_validated_sc("inherit", &mut sc, validate_yes_no_trigger);
        vd.field_validated_sc("can_fire", &mut sc, validate_yes_no_trigger);
        vd.field_validated_sc("can_reassign", &mut sc, validate_yes_no_trigger);
        vd.field_validated_sc("can_change_once", &mut sc, validate_yes_no_trigger);

        let mut count = 0;
        vd.multi_field_validated_block_rooted("modifier", Scopes::Character, |block, data, sc| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization);
            vd.field_script_value("scale", sc);
            validate_modifs(block, data, ModifKinds::Character, vd);
            count += 1;
            if count > 5 {
                let msg = "no more than 5 modifier blocks can be specified here";
                warn(ErrorKey::Validation).msg(msg).loc(block).push();
            }
        });
        count = 0;
        vd.multi_field_validated_block_rooted(
            "council_owner_modifier",
            Scopes::Character,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.field_item("name", Item::Localization);
                vd.field_script_value("scale", sc);
                validate_modifs(block, data, ModifKinds::Character, vd);
                count += 1;
                if count > 5 {
                    let msg = "no more than 5 council_owner_modifier blocks can be specified here";
                    warn(ErrorKey::Validation).msg(msg).loc(block).push();
                }
            },
        );

        vd.field_trigger("valid_position", Tooltipped::No, &mut sc);
        vd.field_trigger("valid_character", Tooltipped::No, &mut sc);

        vd.field_effect("on_get_position", Tooltipped::No, &mut sc);
        vd.field_effect("on_lose_position", Tooltipped::No, &mut sc);
        vd.field_effect("on_fired_from_position", Tooltipped::No, &mut sc);

        vd.advice_field("use_for_scheme_power", "replaced with use_for_scheme_phase_duration");
        vd.field_bool("use_for_scheme_phase_duration");
        vd.field_bool("use_for_scheme_resistance");

        vd.field_item("portrait_animation", Item::PortraitAnimation);
        vd.field_validated_block("barbershop_data", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_numeric_exactly("position", 2);
            vd.field_bool("click_to_front");
        });

        // undocumented

        vd.field_bool("fill_from_pool");
        vd.field_script_value("councillor_cooldown_days", &mut sc);
        vd.field_item("pool_character_config", Item::PoolSelector);
    }
}

#[derive(Clone, Debug)]
pub struct CouncilTask {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CouncilTask, CouncilTask::add)
}

impl CouncilTask {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CouncilTask, key, block, Box::new(Self {}));
    }
}

impl DbKind for CouncilTask {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("councillor", Scopes::Character, key);
        sc.define_name("councillor_liege", Scopes::Character, key);
        if let Some(token) = block.get_field_value("task_type") {
            if token.is("task_type_county") {
                sc.define_name("province", Scopes::Province, token);
                sc.define_name("county", Scopes::LandedTitle, token);
            } else if token.is("task_type_court") {
                sc.define_name("target_character", Scopes::Character, token);
            }
        }

        vd.field_validated_value("default_task", |key, mut vd| {
            vd.bool();
            if vd.value().is("yes") {
                if !block.field_value_is("task_type", "task_type_general") {
                    let msg = "`default_task` is only available for `task_type_general` tasks";
                    err(ErrorKey::Validation).msg(msg).loc(key).push();
                }
                if !block.field_value_is("task_progress", "task_progress_infinite") {
                    let msg = "`default_task` is only available for `task_progress_infinite` tasks";
                    err(ErrorKey::Validation).msg(msg).loc(key).push();
                }
            }
        });
        vd.field_item("position", Item::CouncilPosition);
        vd.field_choice("task_type", &["task_type_general", "task_type_county", "task_type_court"]);
        if block.field_value_is("task_type", "task_type_county") {
            vd.field_choice(
                "county_target",
                &["all", "realm", "domain", "neighbor_land", "neighbor_land_or_water"],
            );
            vd.field_choice(
                "ai_county_target",
                &["all", "realm", "domain", "neighbor_land", "neighbor_land_or_water"],
            );
            if let Some(token) = block.get_field_value("county_target") {
                if token.is("neighbor_land_or_water") {
                    let msg = "`neighbor_land_or_water` is only for `ai_county_target`";
                    warn(ErrorKey::Validation).msg(msg).loc(token).push();
                }
            }
            vd.field_script_value("ai_target_score", &mut sc);
        } else {
            vd.ban_field("county_target", || "task_type_county");
            vd.ban_field("ai_county_target", || "task_type_county");
            vd.ban_field("ai_target_score", || "task_type_county");
        }

        vd.field_choice(
            "task_progress",
            &["task_progress_infinite", "task_progress_percentage", "task_progress_value"],
        );
        if block.field_value_is("task_progress", "task_progress_value") {
            vd.field_script_value("task_current_value", &mut sc);
            vd.field_script_value("task_max_value", &mut sc);
        } else {
            vd.ban_field("task_current_value", || "task_progress_value");
            vd.ban_field("task_max_value", || "task_progress_value");
        }
        vd.field_bool("restart_on_finish");
        vd.field_bool("highlight_own_realm");

        vd.multi_field_validated_block_sc("asset", &mut sc, validate_asset);

        vd.multi_field_validated_block_rooted(
            "councillor_modifier",
            Scopes::Character,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.field_item("name", Item::Localization);
                vd.field_script_value("scale", sc);
                validate_modifs(block, data, ModifKinds::Character, vd);
            },
        );
        vd.multi_field_validated_block_rooted(
            "council_owner_modifier",
            Scopes::Character,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.field_item("name", Item::Localization);
                vd.field_script_value("scale", sc);
                validate_modifs(block, data, ModifKinds::Character, vd);
            },
        );
        if block.field_value_is("task_type", "task_type_county") {
            vd.multi_field_validated_block_rooted(
                "county_modifier",
                Scopes::Character,
                |block, data, sc| {
                    let mut vd = Validator::new(block, data);
                    vd.field_item("name", Item::Localization);
                    vd.field_script_value("scale", sc);
                    validate_modifs(block, data, ModifKinds::County, vd);
                },
            );
        } else {
            vd.ban_field("county_modifier", || "task_type_county");
        }

        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("is_valid_showing_failures_only", Tooltipped::FailuresOnly, &mut sc);
        vd.field_effect("on_start_task", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_finish_task", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_cancel_task", Tooltipped::No, &mut sc);
        vd.field_effect("on_monthly", Tooltipped::No, &mut sc);
        vd.field_action("monthly_on_action", &sc);

        if let Some(token) = block.get_field_value("task_type") {
            if token.is("task_type_county") {
                vd.field_trigger("potential_county", Tooltipped::Yes, &mut sc);
                vd.field_trigger("valid_county", Tooltipped::Yes, &mut sc);
                vd.field_effect("on_start_task_county", Tooltipped::Yes, &mut sc);
                vd.field_effect("on_finish_task_county", Tooltipped::Yes, &mut sc);
                vd.field_effect("on_cancel_task_county", Tooltipped::Yes, &mut sc);
                vd.field_effect("on_monthly_county", Tooltipped::No, &mut sc);
            } else {
                vd.ban_field("potential_county", || "task_type_county");
                vd.ban_field("valid_county", || "task_type_county");
                vd.ban_field("on_start_task_county", || "task_type_county");
                vd.ban_field("on_finish_task_county", || "task_type_county");
                vd.ban_field("on_cancel_task_county", || "task_type_county");
                vd.ban_field("on_monthly_county", || "task_type_county");
            }
            if token.is("task_type_court") {
                vd.field_trigger("potential_target_court", Tooltipped::Yes, &mut sc);
                vd.field_trigger("valid_target_court", Tooltipped::Yes, &mut sc);
                vd.field_effect("on_start_task_court", Tooltipped::Yes, &mut sc);
                vd.field_effect("on_finish_task_court", Tooltipped::Yes, &mut sc);
                vd.field_effect("on_cancel_task_court", Tooltipped::No, &mut sc);
                vd.field_effect("on_monthly_court", Tooltipped::No, &mut sc);
            } else {
                vd.ban_field("potential_court", || "task_type_court");
                vd.ban_field("valid_court", || "task_type_court");
                vd.ban_field("on_start_task_court", || "task_type_court");
                vd.ban_field("on_finish_task_court", || "task_type_court");
                vd.ban_field("on_cancel_task_court", || "task_type_court");
                vd.ban_field("on_monthly_court", || "task_type_court");
            }
        }

        // task_accept_culture is a hardcoded exception
        if !block.field_value_is("task_progress", "task_progress_infinite")
            || key.is("task_accept_culture")
        {
            vd.field_script_value("progress", &mut sc); // documented as a mtth though
            vd.field_script_value("full_progress", &mut sc);
        } else {
            vd.ban_field("progress", || "task_progress_percent or task_progress_value");
            vd.ban_field("full_progress", || "task_progress_percent or task_progress_value");
        }
        vd.field_item("custom_other_loc", Item::Localization);
        vd.field_validated_sc("effect_desc", &mut sc, validate_desc);

        // undocumented
        vd.field_script_value("ai_will_do", &mut sc);
        vd.field_item("skill", Item::Skill);
    }
}

fn validate_yes_no_trigger(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    match bv {
        BV::Value(token) => {
            if !token.is("yes") && !token.is("no") {
                err(ErrorKey::Validation).msg("expected yes or no or trigger").loc(token).push();
            }
        }
        BV::Block(block) => {
            validate_trigger(block, data, sc, Tooltipped::No);
        }
    }
}

fn validate_asset(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.field_trigger("trigger", Tooltipped::No, sc);
    vd.field_item("icon", Item::File);
    vd.field_item("background", Item::File);
    vd.field_item("frame", Item::File);
    vd.field_item("glow", Item::File);
}
