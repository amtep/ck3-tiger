use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct DecisionCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::DecisionCategory, DecisionCategory::add)
}

impl DecisionCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DecisionCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for DecisionCategory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_decision(key, block, data, true);
    }
}

#[derive(Clone, Debug)]
pub struct Decision {
    category: Token,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Decision, Decision::add)
}

impl Decision {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        // Check depth to avoid scanning common/decisions/categories/ here
        if key.loc.pathname().iter().count() == 3 {
            for (decision, block) in block.drain_definitions_warn() {
                db.add(Item::Decision, decision, block, Box::new(Self { category: key.clone() }));
            }
        }
    }
}

impl DbKind for Decision {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::DecisionCategory, &self.category);
        validate_decision(key, block, data, false);
    }
}

fn validate_icon(mut vd: ValueValidator, data: &Everything, is_category: bool) {
    if !vd.maybe_item(Item::Sprite) {
        let category = if is_category { "category_" } else { "" };
        let pathname = format!("gfx/interface/decisions/decision_{}{}.dds", category, vd.value());
        data.verify_exists_implied(Item::File, &pathname, vd.value());
        vd.accept();
    }
}

fn validate_decision(key: &Token, block: &Block, data: &Everything, is_category: bool) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);

    data.verify_exists(Item::Localization, key);

    vd.field_integer("priority");
    vd.multi_field_validated("icon", |bv, data| match bv {
        BV::Value(value) => {
            let vd = ValueValidator::new(value, data);
            validate_icon(vd, data, is_category);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.req_field("key");
            vd.field_validated_value("key", |_, vd| {
                validate_icon(vd, data, is_category);
            });
            vd.field_validated_block("trigger", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
        }
    });
    vd.field_item("picture", Item::Sprite);
    vd.field_bool("visble_when_empty");
    vd.field_bool("cancel_if_not_visible");
    vd.field_validated_block("allowed", |block, data| {
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
    // TODO: set FROM in sc for these
    vd.field_validated_block("visible", |block, data| {
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("available", |block, data| {
        validate_trigger(block, data, &mut sc, Tooltipped::Yes);
    });
    vd.field_validated_block("target_root_trigger", |block, data| {
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated_block("target_trigger", |block, data| {
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.advice_field("state_trigger", "docs say state_trigger but it's state_target");
    vd.field_validated_value("state_target", |_, mut vd| {
        vd.maybe_bool();
        vd.maybe_is("any");
        vd.maybe_is("any_owned_state");
        vd.maybe_is("any_controlled_state");
        vd.item(Item::Continent);
    });
    vd.field_item("scripted_gui", Item::ScriptedGui);

    if !is_category {
        vd.field_bool("is_good");
        vd.field_bool("fire_only_once");
        vd.field_bool("selectable_mission");
        vd.field_integer("days_mission_timeout");
        vd.field_validated_block("activation", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("complete_effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("custom_cost_trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_localization("custom_cost_text", &mut sc);
        vd.field_numeric("ai_hint_pp_cost");
        vd.field_integer("days_remove");
        vd.field_validated_block("cancel_trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("cancel_effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("remove_effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("timeout_effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_choice(
            "on_map_mode",
            &["map_only", "decision_view_only", "map_and_decisions_view"],
        );
        vd.field_value("target_array"); // TODO
    }
}
