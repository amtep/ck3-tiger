use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ImportantAction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ImportantAction, ImportantAction::add)
}

impl ImportantAction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ImportantAction, key, block, Box::new(Self {}));
    }
}

impl DbKind for ImportantAction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_label");
        data.mark_used(Item::Localization, &loca); // TODO: when is _label needed?
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_click");
        data.verify_exists_implied(Item::Localization, &loca, key);
        if block.field_value_is("combine_into_one", "yes") {
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_combined_label");
            data.mark_used(Item::Localization, &loca); // TODO: when is _label needed?
            let loca = format!("{key}_combined_group_label");
            data.mark_used(Item::Localization, &loca); // TODO: when is _label needed?
            let loca = format!("{key}_combined_group_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_combined_group_description");
            data.verify_exists_implied(Item::Localization, &loca, key);
            if block.has_key("unimportant") {
                let loca = format!("{key}_combined_unimportant");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        }

        vd.field_choice("type", &["action", "alert", "tutorial"]);

        if let Some(token) = vd.field_value("icon") {
            data.verify_icon("NGameIcons|IMPORTANT_ACTIONS_TYPE_ICON_PATH", token, ".dds");
        } else if block.field_value_is("type", "alert") {
            data.verify_icon("NGameIcons|IMPORTANT_ACTIONS_TYPE_ICON_PATH", key, ".dds");
        }

        vd.field_bool("is_dangerous");
        vd.field_integer("priority");
        vd.field_bool("combine_into_one");

        // TODO: "only interface effects are allowed"
        vd.field_effect_rooted("check_create_action", Tooltipped::No, Scopes::Character);
        // TODO: "only interface effects are allowed"
        vd.field_effect_builder("effect", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            // TODO: The scope context will contain all scopes passed in the try_create_important_action call
            sc.set_strict_scopes(false);
            sc
        });

        vd.field_item("soundeffect", Item::Sound);

        // undocumented
        vd.field_trigger_builder("unimportant", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("actor", Scopes::Character, key);
            sc.define_name("recipient", Scopes::Character, key);
            sc
        });
    }
}
