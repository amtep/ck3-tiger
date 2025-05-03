use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Operation {}
#[derive(Clone, Debug)]
pub struct OperationToken {}
#[derive(Clone, Debug)]
pub struct OperationPhase {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Operation, Operation::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::OperationToken, OperationToken::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::OperationPhase, OperationPhase::add)
}

impl Operation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Operation, key, block, Box::new(Self {}));
    }
}

impl OperationToken {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::OperationToken, key, block, Box::new(Self {}));
    }
}

impl OperationPhase {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::OperationPhase, key, block, Box::new(Self {}));
    }
}

impl DbKind for Operation {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("icon");
        vd.field_item("icon", Item::Sprite);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
        vd.req_field("desc");
        vd.field_item("desc", Item::Localization);

        vd.req_field("days");
        vd.field_integer("days");
        vd.req_field("operatives");
        vd.field_integer("operatives");
        vd.req_field("network_strength");
        vd.field_integer_range("network_strength", 0..=100);

        vd.field_integer("priority");
        vd.field_integer("danger_level");

        vd.req_field("phases");
        vd.multi_field_validated_block("phases", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_blocks(Item::OperationPhase, |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::Country, key);
                validate_modifiers_with_base(block, data, &mut sc);
            });
        });

        let mut sc = ScopeContext::new(Scopes::Country, key);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);

        vd.field_trigger_rooted("allowed", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("available", Tooltipped::Yes, Scopes::Country);
        vd.field_list_items("awarded_tokens", Item::OperationToken);

        vd.field_list("cost_modifiers"); // TODO what are these?
        vd.field_list("risk_modifiers"); // TODO what are these?

        for field in &["operation_target", "selection_target"] {
            vd.field_validated_key_block(field, |key, block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_validated_list("targets", |value, data| {
                    let mut sc = ScopeContext::new(Scopes::Country, key);
                    validate_target(value, data, &mut sc, Scopes::Country);
                });
            });
        }
        vd.field_choice("target_type", &["province", "strategic_region"]);

        vd.field_validated_block("equipment", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("civilian_factories", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_integer("amount");
                vd.field_integer("days");
            });
            vd.validate_item_key_values(Item::Equipment, |_, mut vd| {
                vd.integer();
            });
        });
        vd.field_list_items("required_tokens", Item::OperationToken);
        vd.field_bool("return_on_complete");
        vd.field_effect_rooted("on_start", Tooltipped::Yes, Scopes::Country);
        vd.field_effect_rooted("outcome_potential", Tooltipped::Yes, Scopes::Country);
        vd.field_numeric("outcome_extra_chance");
        vd.field_list("outcome_modifiers"); // TODO what are these?
        vd.field_effect_rooted("outcome_execute", Tooltipped::Yes, Scopes::Country);
        vd.field_effect_rooted("outcome_extra_execute", Tooltipped::Yes, Scopes::Country);
        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::Country);
        vd.field_bool("will_lead_to_war_with");

        vd.field_bool("prevent_captured_operative_to_die");
        vd.field_bool("scale_cost_independent_of_target");
        vd.field_bool("is_captured_cipher");
        vd.field_bool("is_staged_coup");
        vd.field_numeric("risk_chance");
        vd.field_numeric("experience");
        vd.field_numeric("cost_multiplier");
        vd.field_trigger_rooted("requirements", Tooltipped::Yes, Scopes::Country);
        vd.field_item("map_icon", Item::Sprite);
    }
}

impl DbKind for OperationToken {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("name");
        vd.field_item("name", Item::Localization);
        vd.req_field("desc");
        vd.field_item("desc", Item::Localization);
        vd.req_field("icon");
        vd.field_item("icon", Item::Sprite);
        vd.req_field("text_icon");
        vd.field_item("text_icon", Item::Sprite);

        vd.multi_field_value("targeted_modifier"); // TODO what is this?

        vd.req_field("intel_source");
        vd.field_choice("intel_source", &["navy", "army", "civilian", "airforce"]);
        vd.req_field("intel_gain");
        vd.field_numeric("intel_gain");
    }
}

impl DbKind for OperationPhase {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("name");
        vd.field_item("name", Item::Localization);
        vd.req_field("desc");
        vd.field_item("desc", Item::Localization);
        vd.req_field("icon");
        vd.field_item("icon", Item::Sprite);
        vd.req_field("picture");
        vd.field_item("picture", Item::Sprite);

        vd.field_item("outcome", Item::Localization);

        vd.field_item("map_icon", Item::Sprite);
        vd.field_item("outcome_extra", Item::Localization);
        vd.field_item("risk_extra", Item::Localization);
        vd.field_trigger_rooted("requirements", Tooltipped::Yes, Scopes::Country);
        vd.field_bool("return_on_complete");
        vd.field_validated_block("equipment", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("civilian_factories", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_integer("amount");
                vd.field_integer("days");
            });
            vd.validate_item_key_values(Item::Equipment, |_, mut vd| {
                vd.integer();
            });
        });
    }
}
