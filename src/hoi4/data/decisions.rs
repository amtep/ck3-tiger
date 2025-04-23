use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::{Builder, Validator, ValueValidator};

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
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        for (decision, block) in block.drain_definitions_warn() {
            db.add(Item::Decision, decision, block, Box::new(Self { category: key.clone() }));
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

    if !vd.field_item("name", Item::Localization) {
        data.verify_exists(Item::Localization, key);
    }

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
            vd.field_trigger("trigger", Scopes::Country, Tooltipped::No);
        }
    });
    vd.field_item("picture", Item::Sprite);
    vd.field_bool("visible_when_empty");
    vd.field_bool("cancel_if_not_visible");
    vd.field_trigger("allowed", Scopes::Country, Tooltipped::No);
    let has_state_target = block.get_field_value("state_target").is_some_and(|v| !v.is("no"));
    let sc_builder: &Builder = &move |key| {
        let mut sc = ScopeContext::new(Scopes::Country, key);
        let scope =
            if has_state_target { Scopes::CombinedCountryAndState } else { Scopes::Country };
        sc.push_as_from(scope, key);
        sc
    };
    vd.field_trigger("visible", sc_builder, Tooltipped::No);
    vd.field_trigger("available", sc_builder, Tooltipped::Yes);
    vd.field_validated_block("targets", |block, data| {
        let mut vd = Validator::new(block, data);
        if has_state_target {
            vd.multi_field_item("state", Item::State);
            for value in vd.values() {
                data.verify_exists(Item::State, value);
            }
        } else {
            for value in vd.values() {
                if !value.is("host") {
                    data.verify_exists(Item::CountryTag, value);
                }
            }
        }
    });
    vd.field_bool("targets_dynamic");
    vd.field_bool("target_non_existing");
    vd.field_trigger("target_root_trigger", Scopes::Country, Tooltipped::No);
    vd.field_trigger("target_trigger", sc_builder, Tooltipped::No);
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
        vd.field_variable_or_integer("days_mission_timeout", &mut sc);
        vd.field_trigger("activation", Scopes::Country, Tooltipped::No);
        vd.field_effect("complete_effect", Scopes::Country, Tooltipped::Yes);
        vd.field_trigger("custom_cost_trigger", Scopes::Country, Tooltipped::No);
        vd.field_localization("custom_cost_text", &mut sc);
        vd.field_numeric("ai_hint_pp_cost");
        vd.field_variable_or_integer("days_remove", &mut sc);
        vd.field_trigger("cancel_trigger", sc_builder, Tooltipped::Yes);
        vd.field_effect("cancel_effect", sc_builder, Tooltipped::Yes);
        vd.field_trigger("remove_trigger", sc_builder, Tooltipped::Yes);
        vd.field_effect("remove_effect", sc_builder, Tooltipped::Yes);
        vd.field_effect("timeout_effect", sc_builder, Tooltipped::Yes);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_choice(
            "on_map_mode",
            &["map_only", "decision_view_only", "map_and_decisions_view"],
        );
        vd.field_value("target_array"); // TODO

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.field_variable_or_integer("days_remove", &mut sc);
        vd.field_variable_or_integer("days_re_enable", &mut sc);
        vd.field_variable_or_integer("cost", &mut sc);
        vd.field_bool("fixed_random_seed");
    }
}
