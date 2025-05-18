use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
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
            vd.field_trigger("is_good_for", Tooltipped::Yes, &mut sc);
            vd.field_trigger("is_bad_for", Tooltipped::Yes, &mut sc);
            vd.field_trigger("is_default", Tooltipped::No, &mut sc);
            vd.field_item("skill", Item::Skill);
        } else {
            vd.ban_field("is_good_for", || "type = education");
            vd.ban_field("is_bad_for", || "type = education");
            vd.ban_field("is_default", || "type = education");
            vd.ban_field("skill", || "type = education");
            vd.req_field("lifestyle");
            vd.field_item("lifestyle", Item::Lifestyle);
        }

        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("is_valid", Tooltipped::Yes, &mut sc);
        vd.field_trigger("is_valid_showing_failures_only", Tooltipped::FailuresOnly, &mut sc);
        vd.field_effect("on_change_from", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_birthday", Tooltipped::No, &mut sc);
        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        let icon = vd.field_value("icon").unwrap_or(key);
        data.verify_icon("NGameIcons|FOCUS_ICON_PATH", icon, ".dds");
        vd.field_script_value("auto_selection_weight", &mut sc);

        // undocumented

        vd.field_effect("on_change_from", Tooltipped::No, &mut sc);
    }
}
