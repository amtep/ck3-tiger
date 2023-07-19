use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct Focus {}

impl Focus {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Focus, key, block, Box::new(Self {}));
    }
}

impl DbKind for Focus {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_modifier");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_effect_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_bool("education");
        // TODO: figure out the constraints on focus_id. Do they have to be consecutive?
        vd.req_field("focus_id");
        vd.field_integer("focus_id");

        let education = block.get_field_bool("education").unwrap_or(false);
        if education {
            vd.field_item("skill", Item::Skill);
            vd.field_validated_block("is_default", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_validated_block("is_good_for", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::Yes);
            });
            vd.field_validated_block("is_bad_for", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::Yes);
            });
            vd.field_validated_block("on_change_to", |block, data| {
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_validated_block("on_change_from", |block, data| {
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
        } else {
            vd.req_field("lifestyle");
            vd.field_item("lifestyle", Item::Lifestyle);
        }

        // Undocumented
        vd.field_validated_block("is_valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        // Undocumented, but can confirm that both 'is_valid' and 'is_valid_showing_failures_only' do work for education focus.
        vd.field_validated_block("is_valid_showing_failures_only", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::FailuresOnly);
        });

        vd.field_script_value("auto_selection_weight", &mut sc);
        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        let icon = vd.field_value("icon").unwrap_or(key);
        if let Some(icon_path) = data.get_defined_string_warn(icon, "NGameIcons|FOCUS_ICON_PATH") {
            let pathname = format!("{icon_path}/{icon}.dds");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }
    }
}
