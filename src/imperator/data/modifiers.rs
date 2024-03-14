use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
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
pub struct Modifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Modifier, Modifier::add)
}

impl Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        vd.field_bool("show_in_outliner");

        vd.field_validated_block("cancellation_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_cancellation_effect", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::No);
        });

        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}
