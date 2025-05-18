use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Mission {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Mission, Mission::add)
}

impl Mission {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Mission, key, block, Box::new(Self {}));
    }
}

impl DbKind for Mission {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for (key, block) in block.iter_definitions() {
            if !&[
                "icon",
                "header",
                "repeatable",
                "chance",
                "ai_chance",
                "on_potential",
                "potential",
                "on_start",
                "abort",
                "on_abort",
                "on_completion",
            ]
            .iter()
            .any(|&v| key.is(v))
            {
                db.add(Item::MissionTask, key.clone(), block.clone(), Box::new(MissionTask {}));
            }
        }
    }

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

        vd.field_validated_block_sc("ai_chance", &mut sc, validate_modifiers_with_base);

        vd.field_effect("on_potential", Tooltipped::No, &mut sc);
        vd.field_trigger("potential", Tooltipped::No, &mut sc);
        vd.field_effect("on_start", Tooltipped::No, &mut sc);
        vd.field_trigger("abort", Tooltipped::No, &mut sc);
        vd.field_effect("on_abort", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_completion", Tooltipped::Yes, &mut sc);

        // The individual mission tasks. They are validated by the validate_task function instead of the MissionTask Item.
        vd.unknown_block_fields(|key, block| {
            validate_task(key, block, &mut sc, data);
        });
    }
}

fn validate_task(key: &Token, block: &Block, sc: &mut ScopeContext, data: &Everything) {
    let mut vd = Validator::new(block, data);

    data.verify_exists(Item::Localization, key);
    let loca = format!("{key}_DESC");
    data.verify_exists_implied(Item::Localization, &loca, key);

    vd.field_value("icon");
    vd.field_script_value("duration", sc);
    vd.field_action("monthly_on_action", sc);
    vd.field_bool("final");

    vd.field_list_items("requires", Item::MissionTask);
    vd.field_list_items("prevented_by", Item::MissionTask);

    vd.field_trigger("potential", Tooltipped::No, sc);
    vd.field_trigger_builder("highlight", Tooltipped::Yes, |key| {
        let mut sc = sc.clone();
        sc.define_name("province", Scopes::Province, key);
        sc
    });
    vd.field_trigger("allow", Tooltipped::Yes, sc);
    vd.field_trigger("bypass", Tooltipped::Yes, sc);

    vd.field_validated_block_sc("ai_chance", sc, validate_modifiers_with_base);

    vd.field_effect("on_start", Tooltipped::Yes, sc);
    vd.field_effect("on_completion", Tooltipped::Yes, sc);
    vd.field_effect("on_bypass", Tooltipped::No, sc);
}

#[derive(Clone, Debug)]
pub struct MissionTask {}

impl DbKind for MissionTask {
    fn validate(&self, _key: &Token, _block: &Block, _data: &Everything) {
        // All validation in MissionTask's is done in the validate_task function which is called from the Mission Item
        // This is so the scopes saves in Mission can get passed down into the MissionTask properly while also saving MissionTask Items like that need to be saved.
    }
}
