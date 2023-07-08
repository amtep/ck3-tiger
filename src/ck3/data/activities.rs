use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::ck3::data::scripted_animations::validate_scripted_animation;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{old_warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_cost_with_renown, validate_duration, validate_modifiers_with_base};

#[derive(Clone, Debug)]
pub struct ActivityType {}

impl ActivityType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("options") {
            for (key, block) in block.iter_definitions() {
                db.add_flag(Item::ActivityOptionCategory, key.clone());
                for (key, _) in block.iter_definitions() {
                    db.add_flag(Item::ActivityOption, key.clone());
                }
            }
        }
        if let Some(block) = block.get_field_block("phases") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::ActivityPhase, key.clone());
            }
        }
        if let Some(block) = block.get_field_block("special_guests") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::SpecialGuest, key.clone());
            }
        }
        // window_characters seem to count as special guests too
        if let Some(block) = block.get_field_block("window_characters") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::SpecialGuest, key.clone());
            }
        }
        if let Some(vec) = block.get_field_list("guest_subsets") {
            for token in vec {
                db.add_flag(Item::GuestSubset, token.clone());
            }
        }
        db.add(Item::ActivityType, key, block, Box::new(Self {}));
    }
}

impl DbKind for ActivityType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let has_special_option = block.has_key("special_option_category");

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_owner");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_destination_selection");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_selection_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_guest_help_text");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_predicted_cost");
        data.verify_exists_implied(Item::Localization, &loca, key);

        if block.has_key("province_description") {
            let mut sc = ScopeContext::new(Scopes::Province, key);
            sc.define_name("host", Scopes::Character, key);
            if has_special_option {
                sc.define_name("special_option", Scopes::Flag, key);
            }
            vd.field_validated_sc("province_description", &mut sc, validate_desc);
        } else {
            let loca = format!("{key}_province_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if block.has_key("host_description") {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            vd.field_validated_sc("host_description", &mut sc, validate_desc);
        } else {
            let loca = format!("{key}_host_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        // undocumented
        if block.has_key("guest_description") {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            vd.field_validated_sc("guest_description", &mut sc, validate_desc);
        } else {
            let loca = format!("{key}_guest_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        let ch_host_activity_sc = |key: &Token| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("host", Scopes::Character, key);
            sc.define_name("activity", Scopes::Activity, key);
            sc
        };
        let ac_host_activity_sc = |key: &Token| {
            let mut sc = ScopeContext::new(Scopes::Activity, key);
            sc.define_name("host", Scopes::Character, key);
            sc.define_name("activity", Scopes::Activity, key);
            sc
        };

        if block.has_key("conclusion_description") {
            let mut sc = ch_host_activity_sc(key);
            vd.field_validated_sc("conclusion_description", &mut sc, validate_desc);
        } else {
            let loca = format!("{key}_conclusion_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        let mut join_chance_sc = ScopeContext::new(Scopes::Character, key);
        join_chance_sc.define_name("host", Scopes::Character, key);
        join_chance_sc.define_name("minimal_travel_time", Scopes::Value, key);
        // docs say activity_start_diff_days
        join_chance_sc.define_name("estimated_arrival_diff_days", Scopes::Value, key);
        join_chance_sc.define_list("special_guests", Scopes::Character, key);

        let mut special_guests_sc = ScopeContext::new(Scopes::Character, key);
        if has_special_option {
            special_guests_sc.define_name("special_option", Scopes::Flag, key);
        }
        special_guests_sc.define_list("special_guests", Scopes::Character, key);

        if let Some(block) = block.get_field_block("special_guests") {
            for (key, _) in block.iter_definitions() {
                join_chance_sc.define_name(key.as_str(), Scopes::Character, key);
                special_guests_sc.define_name(key.as_str(), Scopes::Character, key);
            }
        }
        if let Some(block) = block.get_field_block("options") {
            for (_, block) in block.iter_definitions() {
                for (key, _) in block.iter_definitions() {
                    join_chance_sc.define_name(key.as_str(), Scopes::Flag, key);
                }
            }
        }

        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.field_validated_block("is_shown", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("can_start", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_start_showing_failures_only", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::FailuresOnly);
        });

        vd.field_script_value("num_pickable_phases", &mut sc);
        vd.field_script_value("max_pickable_phases_per_province", &mut sc);
        vd.field_validated_block_sc("wait_time_before_start", &mut sc, validate_duration);
        vd.field_validated_block_sc("max_guest_arrival_delay_time", &mut sc, validate_duration);
        vd.field_numeric("max_route_deviation_mult");

        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_bool("is_grand_activity");
        vd.field_bool("is_single_location");
        vd.field_choice("planner_type", &["province", "holder"]);

        vd.field_script_value_no_breakdown("ai_will_do", &mut sc);
        vd.field_integer("ai_check_interval");
        vd.field_script_value_no_breakdown("ai_select_num_provinces", &mut sc);

        vd.field_validated_key_block("is_valid", |key, block, data| {
            let mut sc = ac_host_activity_sc(key);
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_invalidated", |key, block, data| {
            let mut sc = ac_host_activity_sc(key);
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_host_death", |key, block, data| {
            let mut sc = ac_host_activity_sc(key);
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        let filters = &["capital", "domain", "realm", "top_realm", "holy_sites", "all"];
        vd.field_choice("province_filter", filters);
        vd.field_choice("ai_province_filter", filters);

        let mut sc = ScopeContext::new(Scopes::Province, key);
        sc.define_name("host", Scopes::Character, key);
        vd.field_script_value_no_breakdown("province_score", &mut sc);

        if has_special_option {
            sc.define_name("special_option", Scopes::Flag, key);
        }
        vd.field_validated_block("is_location_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        sc.define_name("score", Scopes::Value, key);
        vd.field_script_value_no_breakdown("ai_will_select_province", &mut sc);

        vd.field_integer("max_province_icons");

        vd.field_item("special_option_category", Item::ActivityOptionCategory);
        let special_option_category = block.get_field_value("special_option_category");

        vd.field_validated_block("options", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, block) in vd.unknown_block_fields() {
                // option categories
                let mut vd = Validator::new(block, data);
                let mut is_special_option = false;
                if let Some(special) = special_option_category {
                    if key.is(special.as_str()) {
                        is_special_option = true;
                    }
                }
                for (key, block) in vd.unknown_block_fields() {
                    validate_option(key, block, data, has_special_option, is_special_option);
                }
            }
        });

        vd.field_validated_block("phases", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, block) in vd.unknown_block_fields() {
                validate_phase(key, block, data, has_special_option);
            }
        });

        vd.field_validated_key_block("cost", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            if has_special_option {
                sc.define_name("special_option", Scopes::Flag, key);
            }
            sc.define_name("province", Scopes::Province, key);
            sc.define_list("provinces", Scopes::Province, key);
            validate_cost_with_renown(block, data, &mut sc);
        });

        let mut sc = ScopeContext::new(Scopes::Character, key);
        vd.field_validated_block_sc("ui_predicted_cost", &mut sc, validate_cost_with_renown);

        vd.field_integer("max_guests");
        vd.field_bool("allow_zero_guest_invites");

        vd.field_validated_block("guest_invite_rules", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("rules", |block, data| {
                validate_rules(block, data);
            });
            vd.field_validated_block("defaults", |block, data| {
                // TODO: the rules items in defaults should not be in the rules block above
                validate_rules(block, data);
            });
        });

        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("host", Scopes::Character, key);
        if has_special_option {
            sc.define_name("special_option", Scopes::Flag, key);
        }
        vd.field_validated_block("can_be_activity_guest", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_list("guest_subsets");

        vd.field_validated_block("special_guests", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, block) in vd.unknown_block_fields() {
                validate_special_guest(
                    key,
                    block,
                    data,
                    has_special_option,
                    &mut special_guests_sc,
                );
            }
        });

        vd.field_validated_block("locales", |block, data| {
            let mut vd = Validator::new(block, data);
            // TODO: can we validate the key against anything?
            for (_, block) in vd.unknown_block_fields() {
                let mut vd = Validator::new(block, data);
                vd.field_validated_key_block("is_available", |key, block, data| {
                    let mut sc = ac_host_activity_sc(key);
                    validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
                });
                vd.field_list_items("locales", Item::ActivityLocale);
            }
        });

        let mut sc = ch_host_activity_sc(key);
        vd.field_validated_block_sc("locale_cooldown", &mut sc, validate_duration);
        vd.field_validated_block_sc("auto_select_locale_cooldown", &mut sc, validate_duration);
        for field in &[
            "on_enter_travel_state",
            "on_enter_passive_state",
            "on_enter_active_state",
            "on_leave_travel_state",
            "on_leave_passive_state",
            "on_leave_active_state",
            "on_travel_state_pulse",
            "on_passive_state_pulse",
            "on_active_state_pulse",
            "on_complete",
        ] {
            vd.field_validated_key_block(field, |key, block, data| {
                let mut sc = ch_host_activity_sc(key);
                validate_normal_effect(block, data, &mut sc, Tooltipped::No);
            });
        }
        vd.field_validated_key_block("on_start", |key, block, data| {
            let mut sc = ch_host_activity_sc(key);
            sc.change_root(Scopes::Activity, key);
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        sc.change_root(Scopes::None, key);
        vd.field_validated_block_sc("early_locale_opening_duration", &mut sc, validate_duration);

        vd.field_bool("open_invite");

        vd.field_validated_block("host_intents", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("intents", Item::ActivityIntent);
            vd.field_item("default", Item::ActivityIntent);
            vd.field_list_items("player_defaults", Item::ActivityIntent);
        });
        vd.field_validated_block("guest_intents", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("intents", Item::ActivityIntent);
            vd.field_item("default", Item::ActivityIntent);
            vd.field_list_items("player_defaults", Item::ActivityIntent);
        });

        vd.field_validated_block_sc(
            "guest_join_chance",
            &mut join_chance_sc,
            validate_modifiers_with_base,
        );

        vd.field_validated_block("activity_window_widgets", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, _) in vd.unknown_value_fields() {
                let pathname = format!("gui/activity_window_widgets/{key}.gui");
                data.verify_exists_implied(Item::File, &pathname, key);
                // TODO: what is value?
            }
        });

        vd.field_validated_block("activity_planner_widgets", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, _) in vd.unknown_value_fields() {
                let pathname = format!("gui/activity_window_widgets/{key}.gui");
                data.verify_exists_implied(Item::File, &pathname, key);
                // TODO: what is value?
            }
        });

        let mut sc = ScopeContext::new(Scopes::Activity, key);
        sc.define_name("host", Scopes::Character, key);
        sc.define_name("activity", Scopes::Activity, key);
        vd.field_validated_bvs("map_entity", |bv, data| match bv {
            BV::Value(token) => data.verify_exists(Item::Entity, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_validated_block("trigger", |block, data| {
                    validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
                });
                vd.field_item("reference", Item::Entity);
            }
        });
        vd.field_validated_blocks("background", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |block, data| {
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_item("texture", Item::File);
            vd.field_item("environment", Item::Environment);
            vd.field_item("ambience", Item::Sound);
            vd.field_item("music", Item::Music);
        });
        vd.field_validated_blocks("locale_background", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |block, data| {
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_item("texture", Item::File);
            vd.field_item("environment", Item::Environment);
            vd.field_item("ambience", Item::Sound);
            vd.field_item("music", Item::Music);
        });
        vd.field_validated_block("window_characters", |block, data| {
            let mut vd = Validator::new(block, data);
            for (key, block) in vd.unknown_block_fields() {
                validate_window_characters(key, block, data);
            }
        });
        vd.field_validated_key_block("travel_entourage_selection", |key, block, data| {
            validate_tes(key, block, data, has_special_option);
        });

        // undocumented
        vd.field_validated_block("pulse_actions", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("entries", Item::PulseAction);
            vd.field_numeric_range("chance_of_no_event", 0.0, 100.0);
        });
    }
}

fn validate_rules(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (_, bv) in vd.integer_keys() {
        if let Some(token) = bv.expect_value() {
            data.verify_exists(Item::GuestInviteRule, token);
        }
    }
}

pub fn validate_tes(key: &Token, block: &Block, data: &Everything, has_special_option: bool) {
    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("host", Scopes::Character, key);
    sc.define_name("owner", Scopes::Character, key);
    if has_special_option {
        sc.define_name("special_option", Scopes::Flag, key);
    }
    let mut vd = Validator::new(block, data);
    vd.field_script_value("weight", &mut sc);
    let mut sc = ScopeContext::new(Scopes::None, key);
    vd.field_script_value("max", &mut sc);
    vd.field_script_value("ai_max", &mut sc);
    vd.field_integer("invite_rule_order");
}

fn validate_window_characters(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let loca = format!("activity_window_character_{key}");
    data.verify_exists_implied(Item::Localization, &loca, key);
    vd.field_item("camera", Item::PortraitCamera);
    let mut sc = ScopeContext::new(Scopes::Activity, key);
    sc.define_name("activity", Scopes::Activity, key);
    sc.define_name("host", Scopes::Character, key);
    sc.define_name("player", Scopes::Character, key);
    sc.define_list("characters", Scopes::Character, key);
    vd.field_validated_block("effect", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });

    let mut sc = ScopeContext::new(Scopes::Activity, key);
    sc.define_name("activity", Scopes::Activity, key);
    sc.define_name("host", Scopes::Character, key);
    sc.define_name("player", Scopes::Character, key);
    sc.define_name("character", Scopes::Character, key);
    vd.field_validated_sc("scripted_animation", &mut sc, validate_scripted_animation);
    vd.field_item("animation", Item::PortraitAnimation);
}

fn validate_phase(key: &Token, block: &Block, data: &Everything, has_special_option: bool) {
    let mut vd = Validator::new(block, data);
    vd.field_bool("is_predefined");
    let mut sc = ScopeContext::new(Scopes::None, key);
    vd.field_script_value("number_of_picks", &mut sc);
    // TODO: "you should have unique order nr for your phases, if you have more than one phase"
    vd.field_integer("order");
    vd.field_choice(
        "location_source",
        &["pickable", "first_picked_phase", "last_picked_phase", "scripted"],
    );
    if block.field_value_is("location_source", "scripted") {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        if has_special_option {
            sc.define_name("special_option", Scopes::Flag, key);
        }
        vd.field_validated_block("select_scripted_location", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
    } else {
        vd.ban_field("select_scripted_location", || "location_source = scripted");
    }
    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("province", Scopes::Province, key);
    if has_special_option {
        sc.define_name("special_option", Scopes::Flag, key);
    }
    vd.field_script_value_no_breakdown("ai_will_do", &mut sc);

    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("province", Scopes::Province, key);
    if has_special_option {
        sc.define_name("special_option", Scopes::Flag, key);
    }
    vd.field_validated_block("is_shown", |block, data| {
        validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("can_pick", |block, data| {
        validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
    });
    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("province", Scopes::Province, key);
    sc.define_name("host", Scopes::Character, key);
    vd.field_validated_block("is_valid", |block, data| {
        validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
    });

    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("activity", Scopes::Activity, key);
    sc.define_name("host", Scopes::Character, key);
    vd.field_validated_block("on_enter_phase", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("on_phase_active", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("on_end", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("on_monthly_pulse", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("on_weekly_pulse", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });
    sc.define_name("province", Scopes::Province, key);
    vd.field_validated_block("on_invalidated", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });

    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("province", Scopes::Province, key);
    sc.define_name("previous_province", Scopes::Province, key);
    if has_special_option {
        sc.define_name("special_option", Scopes::Flag, key);
    }
    vd.field_validated_block_sc("cost", &mut sc, validate_cost_with_renown);
}

fn validate_special_guest(
    key: &Token,
    block: &Block,
    data: &Everything,
    has_special_option: bool,
    special_guests_sc: &mut ScopeContext,
) {
    let mut vd = Validator::new(block, data);
    data.verify_exists(Item::Localization, key);
    let loca = format!("{key}_desc");
    data.verify_exists_implied(Item::Localization, &loca, key);
    let loca = format!("{key}_for_host");
    data.verify_exists_implied(Item::Localization, &loca, key);
    let loca = format!("{key}_desc_for_host");
    data.verify_exists_implied(Item::Localization, &loca, key);
    let mut sc = ScopeContext::new(Scopes::Character, key);
    if has_special_option {
        sc.define_name("special_option", Scopes::Flag, key);
    }
    vd.field_validated_block("is_shown", |block, data| {
        validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_bool("is_required");

    vd.field_validated_block("select_character", |block, data| {
        // TODO: "interface effects"
        validate_normal_effect(block, data, special_guests_sc, Tooltipped::No);
    });

    special_guests_sc.define_name("host", Scopes::Character, key);
    vd.field_validated_block("can_pick", |block, data| {
        validate_normal_trigger(block, data, special_guests_sc, Tooltipped::No);
    });
    vd.field_validated_block("on_invite", |block, data| {
        validate_normal_effect(block, data, special_guests_sc, Tooltipped::No);
    });

    let mut sc = ScopeContext::new(Scopes::Character, key);
    vd.field_script_value_no_breakdown("ai_will_do", &mut sc);
}

#[derive(Clone, Debug)]
pub struct ActivityLocale {}

impl ActivityLocale {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ActivityLocale, key, block, Box::new(Self {}));
    }
}

impl DbKind for ActivityLocale {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("host", Scopes::Character, key);
        sc.define_name("activity", Scopes::Activity, key);
        vd.field_validated_block("is_available", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_script_value("chance", &mut sc);
        vd.field_validated_block("on_enter_locale", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_script_value_no_breakdown("ai_will_do", &mut sc);
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_validated_key_bvs("visuals", validate_visuals);
    }
}

fn validate_visuals(key: &Token, bv: &BV, data: &Everything) {
    match bv {
        BV::Value(_) => (), // TODO
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            let mut sc = ScopeContext::new(Scopes::Activity, key);
            sc.define_name("activity", Scopes::Activity, key);
            sc.define_name("host", Scopes::Character, key);
            vd.field_validated_block("trigger", |block, data| {
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_value("reference"); // TODO
        }
    }
}

#[derive(Clone, Debug)]
pub struct GuestInviteRule {}

impl GuestInviteRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GuestInviteRule, key, block, Box::new(Self {}));
    }
}

impl DbKind for GuestInviteRule {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);

        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_list("characters", Scopes::Character, key);
        vd.field_validated_block("effect", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct PulseAction {}

impl PulseAction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PulseAction, key, block, Box::new(Self {}));
    }
}

impl DbKind for PulseAction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let loca = format!("{key}_title");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let icon = vd.field_value("icon").unwrap_or(key);
        let pathname = format!("gfx/interface/icons/activity_pulse_actions/{icon}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);

        let mut sc = ScopeContext::new(Scopes::Activity, key);
        sc.define_name("activity", Scopes::Activity, key);
        sc.define_name("host", Scopes::Character, key);
        sc.define_name("province", Scopes::Province, key);
        vd.field_validated_block("is_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_script_value("weight", &mut sc);
        vd.field_validated_block("effect", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct ActivityIntent {}

impl ActivityIntent {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ActivityIntent, key, block, Box::new(Self {}));
    }
}

impl DbKind for ActivityIntent {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let icon = vd.field_value("icon").unwrap_or(key);
        let pathname = format!("gfx/interface/icons/activity_intents/{icon}.dds");
        data.verify_exists_implied(Item::File, &pathname, icon);

        vd.field_bool("auto_complete");

        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("magnificence", Scopes::Value, key);
        sc.define_name("special_option", Scopes::Flag, key);
        vd.field_validated_block("is_shown", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("is_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("target", Scopes::Character, key);
        sc.define_name("special_option", Scopes::Flag, key);
        vd.field_validated_block("is_target_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        let mut sc = ScopeContext::new(Scopes::Character, key);
        vd.field_validated_block("on_invalidated", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });
        sc.define_name("target", Scopes::Character, key);
        vd.field_validated_block("on_target_invalidated", |block, data| {
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("activity", Scopes::Activity, key);
        vd.field_script_value_no_breakdown("ai_will_do", &mut sc);

        vd.field_blocks("ai_targets"); // TODO, see also interactions
        vd.field_blocks("ai_target_quick_trigger"); // TODO, see also interactions

        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("target", Scopes::Character, key);
        sc.define_name("special_option", Scopes::Flag, key);
        vd.field_script_value_no_breakdown("ai_target_score", &mut sc);

        let mut sc = ScopeContext::new(Scopes::Character, key);
        vd.field_validated_sc("scripted_animation", &mut sc, validate_scripted_animation);
    }
}

fn validate_option(
    key: &Token,
    block: &Block,
    data: &Everything,
    has_special_option: bool,
    is_special_option: bool,
) {
    let mut vd = Validator::new(block, data);
    data.verify_exists(Item::Localization, key);
    let loca = format!("{key}_desc");
    data.verify_exists_implied(Item::Localization, &loca, key);
    if is_special_option {
        let pathname = format!("gfx/interface/illustrations/activity_types/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);
        let pathname = format!("gfx/interface/icons/activity_types/{key}_icon.dds");
        data.verify_exists_implied(Item::File, &pathname, key);
    }
    let mut sc = ScopeContext::new(Scopes::Character, key);
    if has_special_option {
        sc.define_name("special_option", Scopes::Flag, key);
    }
    vd.field_validated_block("is_shown", |block, data| {
        validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("is_valid", |block, data| {
        validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
    });
    vd.field_script_value_no_breakdown("ai_will_do", &mut sc);

    let mut sc = ScopeContext::new(Scopes::Activity, key);
    sc.define_name("activity", Scopes::Activity, key);
    sc.define_name("host", Scopes::Character, key);
    vd.field_validated_block("on_start", |block, data| {
        validate_normal_effect(block, data, &mut sc, Tooltipped::No);
    });

    vd.field_validated("default", |bv, data| {
        match bv {
            BV::Value(token) => {
                if !token.is("yes") {
                    let msg = "expected `default = yes`";
                    old_warn(token, ErrorKey::Validation, msg);
                }
            }
            BV::Block(block) => {
                // TODO: what is the scope context?
                let mut sc = ScopeContext::new(Scopes::Character, key);
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            }
        }
    });

    vd.field_list_items("blocked_intents", Item::ActivityIntent);

    let mut sc = ScopeContext::new(Scopes::Character, key);
    vd.field_validated_block_sc("cost", &mut sc, validate_cost_with_renown);

    vd.field_validated_key_block("travel_entourage_selection", |key, block, data| {
        validate_tes(key, block, data, has_special_option);
    });
}
