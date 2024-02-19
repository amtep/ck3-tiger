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
pub struct Building {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Building, Building::add)
}

impl Building {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Building, key, block, Box::new(Self {}));
    }
}

impl DbKind for Building {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Province, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("potential", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block_sc("chance", &mut sc, validate_modifiers_with_base);
        // Somehow both of these are allowed even though they are the same thing...
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);

        vd.field_numeric("max_amount");
        vd.field_numeric("cost");
        vd.field_numeric("time");

        vd.no_warn_remaining();

        // TODO - Not sure what to do with modification_display, it is extremely irregular so I just want to not validate the whole block
        vd.field_block("modification_display"); // TODO

        validate_modifs(block, data, ModifKinds::Province, vd);
    }
}
