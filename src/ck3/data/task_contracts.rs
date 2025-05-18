use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

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
        fn sc_weight(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("employer", Scopes::Character, key);
            sc
        }

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

        vd.field_trigger_builder("valid_to_create", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("employer", Scopes::Character, key);
            sc
        });
        vd.field_trigger_builder("valid_to_accept", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("employer", Scopes::Character, key);
            sc
        });
        vd.field_trigger_rooted("valid_to_continue", Tooltipped::No, Scopes::TaskContract);
        vd.field_trigger_rooted("valid_to_keep", Tooltipped::No, Scopes::TaskContract);

        vd.field_effect_builder("on_create", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("contract", Scopes::TaskContract, key);
            sc
        });

        for field in &["on_accepted", "on_completed", "on_invalidated"] {
            vd.field_effect_rooted(field, Tooltipped::No, Scopes::TaskContract);
        }

        vd.field_bool("should_show_toast_on_complete");

        vd.field_validated_block("task_contract_reward", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                let mut vd = Validator::new(block, data);
                vd.field_effect_rooted("effect", Tooltipped::Yes, Scopes::TaskContract);
                vd.field_bool("should_print_on_complete");
                vd.field_bool("visible");
                vd.field_bool("positive");
            });
        });

        vd.field_script_value_no_breakdown_builder("weight", sc_weight);
    }
}
