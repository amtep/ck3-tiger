use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Focus {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Focus, Focus::add)
}

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

        vd.advice_field("education", "replaced with `type`");
        vd.field_choice("type", &["education", "lifestyle"]);

        let education = block.get_field_value("type").is_some_and(|token| token.is("education"));
        if education {
            vd.ban_field("lifestyle", || "type = lifestyle");
            vd.field_validated_block("is_good_for", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::Yes);
            });
            vd.field_validated_block("is_bad_for", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::Yes);
            });
            vd.field_validated_block("is_default", |block, data| {
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_item("skill", Item::Skill);
        } else {
            vd.ban_field("is_good_for", || "type = education");
            vd.ban_field("is_bad_for", || "type = education");
            vd.ban_field("is_default", || "type = education");
            vd.ban_field("skill", || "type = education");
            vd.req_field("lifestyle");
            vd.field_item("lifestyle", Item::Lifestyle);
        }

        vd.field_validated_block("is_shown", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("is_valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("is_valid_showing_failures_only", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::FailuresOnly);
        });
        vd.field_validated_block("on_change_from", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_birthday", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        let icon = vd.field_value("icon").unwrap_or(key);
        data.verify_icon("NGameIcons|FOCUS_ICON_PATH", icon, ".dds");
        vd.field_script_value("auto_selection_weight", &mut sc);

        // undocumented

        vd.field_validated_block("on_change_to", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}
