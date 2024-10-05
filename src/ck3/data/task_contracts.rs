use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::{Builder, Validator};

#[derive(Clone, Debug)]
pub struct TaskContractType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::TaskContractType, TaskContractType::add)
}

impl TaskContractType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TaskContractType, key, block, Box::new(Self {}));
    }
}

impl DbKind for TaskContractType {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(group) = block.get_field_value("group") {
            db.add_flag(Item::TaskContractGroup, group.clone());
        }
        if let Some(block) = block.get_field_block("task_contract_reward") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::TaskContractReward, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::TaskContractType, key);

        let loca = format!("{key}_contract");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("group");
        vd.field_item("icon", Item::File);

        if block.has_key("desc") {
            vd.field_validated_sc("desc", &mut sc, validate_desc);
        } else {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_desc_title");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if block.has_key("task_contract_request") {
            vd.field_validated_sc("task_contract_request", &mut sc, validate_desc);
        } else {
            let loca = format!("{key}_request");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_bool("travel");
        vd.field_bool("is_criminal");
        vd.field_bool("use_diplomatic_range");

        vd.field_validated_key_block("valid_to_create", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("employer", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("valid_to_accept", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("employer", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_key_block("valid_to_continue", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::TaskContract, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("valid_to_keep", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::TaskContract, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key_block("on_create", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("contract", Scopes::TaskContract, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        for field in &["on_accepted", "on_completed", "on_invalidated"] {
            vd.field_validated_key_block(field, |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::TaskContract, key);
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
        }

        vd.field_bool("should_show_toast_on_complete");

        vd.field_validated_block("task_contract_reward", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                let mut vd = Validator::new(block, data);
                vd.field_validated_key_block("effect", |key, block, data| {
                    let mut sc = ScopeContext::new(Scopes::TaskContract, key);
                    validate_effect(block, data, &mut sc, Tooltipped::Yes);
                });
                vd.field_bool("should_print_on_complete");
                vd.field_bool("visible");
                vd.field_bool("positive");
            });
        });

        let sc_weight: &Builder = &|key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("employer", Scopes::Character, key);
            sc
        };
        vd.field_script_value_full("weight", sc_weight, false);
    }
}
