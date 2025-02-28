use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GovernorPolicy {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GovernorPolicy, GovernorPolicy::add)
}

impl GovernorPolicy {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GovernorPolicy, key, block, Box::new(Self {}));
    }
}

impl DbKind for GovernorPolicy {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::State, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("province", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::State, vd);
        });
        vd.field_validated_block("capital", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::State, vd);
        });
        vd.field_validated_block("non_capital", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::State, vd);
        });

        vd.field_action("on_action", &sc);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}
