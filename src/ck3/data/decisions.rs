use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{old_warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_cost, validate_duration, validate_modifiers_with_base};

#[derive(Clone, Debug)]
pub struct Ck3Decision {}

impl Ck3Decision {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Decision, key, block, Box::new(Self {}));
    }
}

impl DbKind for Ck3Decision {
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
        vd.field_item("picture", Item::File);
        vd.field_item("extra_picture", Item::File);
        vd.field_bool("major");
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
        vd.field_validated_blocks("cost", |b, data| validate_cost(b, data, &mut sc));
        check_cost(&block.get_field_blocks("cost"));
        vd.field_validated_blocks("minimum_cost", |b, data| validate_cost(b, data, &mut sc));
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
        vd.field("widget"); // TODO
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
                    old_warn(gold, ErrorKey::Conflict, msg);
                }
                seen_gold = true;
            }
            if let Some(prestige) = cost.get_field("prestige") {
                if seen_prestige {
                    let msg = "This value of the prestige cost overrides the previously set cost.";
                    old_warn(prestige, ErrorKey::Conflict, msg);
                }
                seen_prestige = true;
            }
            if let Some(piety) = cost.get_field("piety") {
                if seen_piety {
                    let msg = "This value of the piety cost overrides the previously set cost.";
                    old_warn(piety, ErrorKey::Conflict, msg);
                }
                seen_piety = true;
            }
        }
    }
}
