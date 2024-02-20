use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
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

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block_sc("chance", &mut sc, validate_modifiers_with_base);

        vd.field_validated_block("on_start", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("finished_when", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_finish", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("abort_when", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_abort", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_item("monthly_on_action", Item::OnAction);
    }
}
