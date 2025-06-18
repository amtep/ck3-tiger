use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_duration;
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

        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("journal_entry", Scopes::JournalEntry, key);
        sc.define_name("target", Scopes::all(), key);
        // TODO: check with strict scopes from the journal entry that uses this button
        sc.set_strict_scopes(false);

        vd.req_field("name");
        vd.field_validated_sc("name", &mut sc, validate_desc);
        vd.req_field("desc");
        vd.field_validated_sc("desc", &mut sc, validate_desc);

        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_trigger("visible", Tooltipped::No, &mut sc);
        vd.field_trigger("possible", Tooltipped::Yes, &mut sc);
        vd.field_effect("effect", Tooltipped::Yes, &mut sc);

        vd.field_script_value_no_breakdown("ai_chance", &mut sc);

        // undocumented

        vd.field_trigger("selected", Tooltipped::No, &mut sc);
    }
}
