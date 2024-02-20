use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ScriptedGui {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::ScriptedGui, ScriptedGui::add)
}

impl ScriptedGui {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedGui, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedGui {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        // TODO: vanilla vic3 does not use scripted guis so it's not clear which fields are allowed there.
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        if let Some(token) = vd.field_value("scope") {
            if let Some(scope) = Scopes::from_snake_case(token.as_str()) {
                sc = ScopeContext::new(scope, token);
            } else {
                warn(ErrorKey::Scopes).msg("unknown scope type").loc(token).push();
            }
        }

        // These two are undocumented in CK3 and Vic3
        vd.field_validated_sc("confirm_title", &mut sc, validate_desc);
        vd.field_validated_sc("confirm_text", &mut sc, validate_desc);

        vd.field_validated_list("saved_scopes", |token, _data| {
            sc.define_name(token.as_str(), Scopes::all_but_none(), token);
        });

        vd.field_validated_block("is_shown", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        // This is from Imperator but is reported to work in CK3 too
        vd.field_validated_block("ai_is_valid", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("is_valid", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("effect", |b, data| {
            // TODO: whether this is tooltipped depends on whether the gui calls for it
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });
        // This is from Imperator but is reported to work in CK3 too
        vd.field_validated_block_sc("ai_chance", &mut sc, validate_modifiers_with_base);
    }
}
