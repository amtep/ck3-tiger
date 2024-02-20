use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Ambition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Ambition, Ambition::add)
}

impl Ambition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Ambition, key, block, Box::new(Self {}));
    }
}

impl DbKind for Ambition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        sc.define_name("ongoing_scheme_target", Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("can_be_picked");
        vd.req_field("finished_when");
        vd.req_field("chance");

        vd.field_validated_block("can_be_picked", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("finished_when", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("abort", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block_sc("chance", &mut sc, validate_modifiers_with_base);

        vd.field_numeric("duration");
        vd.field_bool("content");
        vd.field_bool("skip_initial_abort");

        vd.field_item("on_monthly", Item::OnAction);
        vd.field_item("on_start", Item::OnAction);
        vd.field_item("on_finish", Item::OnAction);
        vd.field_item("on_abort", Item::OnAction);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
    }
}
