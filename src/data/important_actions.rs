use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct ImportantAction {}

impl ImportantAction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ImportantAction, key, block, Box::new(Self {}));
    }
}

impl DbKind for ImportantAction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_label");
        data.item_used(Item::Localization, &loca); // TODO: when is _label needed?
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_click");
        if block.field_value_is("combine_into_one", "yes") {
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_combined_label");
            data.item_used(Item::Localization, &loca); // TODO: when is _label needed?
            let loca = format!("{key}_combined_group_label");
            data.item_used(Item::Localization, &loca); // TODO: when is _label needed?
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
            let pathname = format!("gfx/interface/icons/alerts/{token}.dds");
            data.verify_exists_implied(Item::File, &pathname, token);
        } else if block.field_value_is("type", "alert") {
            let pathname = format!("gfx/interface/icons/alerts/{key}.dds");
            data.verify_exists_implied(Item::File, &pathname, key);
        }

        vd.field_bool("is_dangerous");
        vd.field_integer("priority");
        vd.field_bool("combine_into_one");

        vd.field_validated_block("check_create_action", |block, data| {
            // TODO: "only interface effects are allowed"
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("effect", |block, data| {
            let mut sc = sc.clone();
            // TODO: The scope context will contain all scopes passed in the try_create_important_action call
            sc.set_strict_scopes(false);
            // TODO: "only interface effects are allowed"
            validate_normal_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_item("soundeffect", Item::Sound);

        // undocumented
        vd.field_validated_block("unimportant", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}
