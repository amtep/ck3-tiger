use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ScriptedGui {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::jomini(), Item::ScriptedGui, ScriptedGui::add)
}

impl ScriptedGui {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedGui, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedGui {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        if let Some(token) = vd.field_value("scope") {
            if let Some(scope) = Scopes::from_snake_case(token.as_str()) {
                sc = ScopeContext::new(scope, token);
            } else {
                warn(ErrorKey::Scopes).msg("unknown scope type").loc(token).push();
            }
        }

        // TODO: JominiNotification
        vd.field_value("notification_key");
        vd.field_validated_sc("confirm_title", &mut sc.clone(), validate_desc);
        vd.field_validated_sc("confirm_text", &mut sc.clone(), validate_desc);
        vd.field_trigger("ai_is_valid", &mut sc.clone(), Tooltipped::No);
        vd.field_validated_block_sc("ai_chance", &mut sc.clone(), validate_modifiers_with_base);
        vd.field_validated("ai_frequency", validate_non_dynamic_script_value);

        vd.field_validated_list("saved_scopes", |token, _| {
            sc.define_name(token.as_str(), Scopes::all_but_none(), token);
        });
        vd.field_trigger("is_shown", &mut sc.clone(), Tooltipped::No);
        vd.field_trigger("is_valid", &mut sc.clone(), Tooltipped::No);
        // TODO: whether this is tooltipped depends on whether the gui calls for it
        vd.field_effect("effect", &mut sc.clone(), Tooltipped::No);
    }
}
