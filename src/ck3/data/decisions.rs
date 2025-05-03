use crate::block::{Block, BV};
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_duration, validate_modifiers_with_base};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Decision {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Decision, Decision::add)
}

impl Decision {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Decision, key, block, Box::new(Self {}));
    }
}

impl DbKind for Decision {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        if let Some(block) = block.get_field_block("widget") {
            // Add builtin scopes added by known widget controllers
            if let Some(controller) = block.get_field_value("controller") {
                if controller.is("decision_option_list_controller") {
                    for block in block.get_field_blocks("item") {
                        if let Some(token) = block.get_field_value("value") {
                            sc.define_name(token.as_str(), Scopes::Bool, token);
                        }
                    }
                } else if controller.is("create_holy_order") {
                    sc.define_name("ruler", Scopes::Character, controller);
                } else if controller.is("revoke_holy_order_lease") {
                    sc.define_name("barony", Scopes::LandedTitle, controller);
                }
            }
        }

        vd.req_field_warn("picture");
        vd.multi_field_validated_block("picture", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_key_block("trigger", |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_item("reference", Item::File);
            vd.field_item("soundeffect", Item::Sound);
        });
        vd.field_item("extra_picture", Item::File);
        vd.advice_field("major", "Replaced with decision_group_type");
        vd.field_item("decision_group_type", Item::DecisionGroup);
        vd.field_integer("sort_order");
        vd.field_bool("is_invisible");
        vd.field_bool("ai_goal");
        vd.field_integer("ai_check_interval");
        if block.get_field_bool("ai_goal").unwrap_or(false) {
            vd.advice_field("ai_check_interval", "not needed if ai_goal = yes");
        }
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_item("confirm_click_sound", Item::Sound);

        if !vd.field_validated_sc("selection_tooltip", &mut sc, validate_desc) {
            let loca = format!("{key}_tooltip");
            data.verify_exists_implied(Item::Localization, &loca, key);
            data.validate_localization_sc(&loca, &mut sc);
        }

        if !vd.field_validated_sc("title", &mut sc, validate_desc) {
            data.verify_exists(Item::Localization, key);
            data.validate_localization_sc(key.as_str(), &mut sc);
        }

        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            data.validate_localization_sc(&loca, &mut sc);
        }

        if !vd.field_validated_sc("confirm_text", &mut sc, validate_desc) {
            let loca = format!("{key}_confirm");
            data.verify_exists_implied(Item::Localization, &loca, key);
            data.validate_localization_sc(&loca, &mut sc);
        }

        vd.field_validated_block("is_shown", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("is_valid_showing_failures_only", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::FailuresOnly);
        });
        vd.field_validated_block("is_valid", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        // cost can have multiple definitions and they will be combined
        // however, two costs of the same type are not summed
        vd.multi_field_validated_block("cost", |b, data| validate_cost(b, data, &mut sc));
        check_cost(&block.get_field_blocks("cost"));
        vd.multi_field_validated_block("minimum_cost", |b, data| validate_cost(b, data, &mut sc));
        check_cost(&block.get_field_blocks("minimum_cost"));

        vd.field_validated_block("effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("ai_potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block("should_create_alert", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated("widget", |bv, data| {
            match bv {
                BV::Value(value) => {
                    let filename =
                        format!("gui/decision_view_widgets/decision_view_widget_{value}.gui");
                    data.verify_exists_implied(Item::File, &filename, value);
                }
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);
                    vd.field_validated_value("gui", |key, mut vd| {
                        let filename = format!("gui/decision_view_widgets/{vd}.gui");
                        data.verify_exists_implied(Item::File, &filename, key);
                        vd.accept();
                    });

                    // LAST UPDATED CK3 VERSION 1.12.3
                    vd.field_choice(
                        "controller",
                        &[
                            "default",
                            "decision_option_list_controller",
                            "create_holy_order",
                            "revoke_holy_order_lease", // undocumented
                        ],
                    );
                    vd.field_bool("show_from_start");

                    // Undocumented
                    vd.field_item("decision_to_second_step_button", Item::Localization);

                    match block.get_field_value("controller").map(Token::as_str) {
                        Some("decision_option_list_controller") => {
                            vd.multi_field_validated_block("item", |block, data| {
                                let mut vd = Validator::new(block, data);
                                vd.field_value("value");
                                vd.field_validated_block_rooted(
                                    "is_shown",
                                    Scopes::Character,
                                    |block, data, sc| {
                                        validate_trigger(block, data, sc, Tooltipped::No);
                                    },
                                );
                                vd.field_validated_block_rooted(
                                    "is_valid",
                                    Scopes::Character,
                                    |block, data, sc| {
                                        validate_trigger(block, data, sc, Tooltipped::FailuresOnly);
                                    },
                                );
                                vd.field_validated_rooted(
                                    "current_description",
                                    Scopes::Character,
                                    validate_desc,
                                );
                                vd.field_validated_rooted(
                                    "localization",
                                    Scopes::Character,
                                    validate_desc,
                                );
                                vd.field_bool("is_default");
                                vd.field_item("icon", Item::File);
                                vd.field_bool("flat");

                                vd.field_script_value_no_breakdown_rooted(
                                    "ai_chance",
                                    Scopes::Character,
                                );
                            });
                        }
                        Some("create_holy_order" | "revoke_holy_order_lease") => {
                            vd.field_validated_block_build_sc(
                                "barony_valid",
                                |key| {
                                    let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
                                    sc.define_name("ruler", Scopes::Character, key);
                                    sc
                                },
                                |block, data, sc| {
                                    validate_trigger(block, data, sc, Tooltipped::No);
                                },
                            );
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}

fn check_cost(blocks: &[&Block]) {
    let mut seen_gold = false;
    let mut seen_prestige = false;
    let mut seen_piety = false;
    if blocks.len() > 1 {
        for cost in blocks {
            if let Some(gold) = cost.get_field("gold") {
                if seen_gold {
                    let msg = "This value of the gold cost overrides the previously set cost.";
                    warn(ErrorKey::Conflict).msg(msg).loc(gold).push();
                }
                seen_gold = true;
            }
            if let Some(prestige) = cost.get_field("prestige") {
                if seen_prestige {
                    let msg = "This value of the prestige cost overrides the previously set cost.";
                    warn(ErrorKey::Conflict).msg(msg).loc(prestige).push();
                }
                seen_prestige = true;
            }
            if let Some(piety) = cost.get_field("piety") {
                if seen_piety {
                    let msg = "This value of the piety cost overrides the previously set cost.";
                    warn(ErrorKey::Conflict).msg(msg).loc(piety).push();
                }
                seen_piety = true;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DecisionGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DecisionGroup, DecisionGroup::add)
}

impl DecisionGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DecisionGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for DecisionGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("decision_group_type_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_integer("sort_order");
        vd.field_list("gui_tags");
        vd.field_bool("important_decision_group");
    }
}
