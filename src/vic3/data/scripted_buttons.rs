use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
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
pub struct ScriptedButton {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ScriptedButton, ScriptedButton::add)
}

impl ScriptedButton {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedButton, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedButton {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // TODO: assuming that the scopes from the journalentry are available
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("journal_entry", Scopes::JournalEntry, key);
        sc.define_name("target", Scopes::all(), key);

        vd.req_field("name");
        vd.field_validated_sc("name", &mut sc, validate_desc);
        vd.req_field("desc");
        vd.field_validated_sc("desc", &mut sc, validate_desc);

        vd.field_validated_block("visible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("effect", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block_sc("ai_chance", &mut sc, validate_modifiers_with_base);
    }
}
