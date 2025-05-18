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
pub struct PartyAgenda {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::PartyAgenda, PartyAgenda::add)
}

impl PartyAgenda {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PartyAgenda, key, block, Box::new(Self {}));
    }
}

impl DbKind for PartyAgenda {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Party, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_DESC");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_trigger("potential", Tooltipped::No, &mut sc);

        vd.field_validated_block_sc("chance", &mut sc, validate_modifiers_with_base);

        vd.field_effect("on_start", Tooltipped::No, &mut sc);
        vd.field_trigger("finished_when", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_finish", Tooltipped::Yes, &mut sc);
        vd.field_trigger("abort_when", Tooltipped::No, &mut sc);
        vd.field_effect("on_abort", Tooltipped::No, &mut sc);

        vd.field_action("monthly_on_action", &sc);
    }
}
