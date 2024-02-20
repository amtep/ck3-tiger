use crate::block::Block;
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
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Mission {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Mission, Mission::add)
}

impl Mission {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for (key, block) in block.iter_definitions() {
            db.add(Item::MissionTask, key.clone(), block.clone(), Box::new(MissionTask {}));
        }
        db.add(Item::Mission, key, block, Box::new(Self {}));
    }
}

impl DbKind for Mission {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca1 = format!("{key}_DESCRIPTION");
        let loca2 = format!("{key}_CRITERIA_DESCRIPTION");
        let loca3 = format!("{key}_BUTTON_DETAILS");
        let loca4 = format!("{key}_BUTTON_TOOLTIP");
        data.verify_exists_implied(Item::Localization, &loca1, key);
        data.verify_exists_implied(Item::Localization, &loca2, key);
        data.verify_exists_implied(Item::Localization, &loca3, key);
        data.verify_exists_implied(Item::Localization, &loca4, key);

        vd.field_value("icon");
        vd.field_value("header");

        vd.field_bool("repeatable");

        vd.field_validated_block_sc("chance", &mut sc, validate_modifiers_with_base);

        // TODO - Any scopes that are saved in the on_potential or on_start get added to the ScopeContext of every other block in the mission tree, except for the potential block for the entire tree.
        // Need to figure out how to propogate these save scopes to the new blocks
        vd.field_validated_block("on_potential", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_start", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("abort", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_abort", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_completion", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });

        // The individual mission tasks. They are validated in the MissionTask class.
        vd.unknown_block_fields(|_, _| ());
    }
}

#[derive(Clone, Debug)]
pub struct MissionTask {}

impl DbKind for MissionTask {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_DESC");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("icon");
        vd.field_script_value("duration", &mut sc);
        vd.field_item("monthly_on_action", Item::OnAction);
        vd.field_bool("final");

        vd.field_list_items("requires", Item::MissionTask);
        vd.field_list_items("prevented_by", Item::MissionTask);

        vd.field_validated_block("highlight", |b, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("province", Scopes::Province, key);
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("bypass", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block_sc("ai_chance", &mut sc, validate_modifiers_with_base);

        vd.field_validated_block("on_start", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_completion", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_bypass", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });
    }
}
