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
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PartyType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::PartyType, PartyType::add)
}

impl PartyType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PartyType, key, block, Box::new(Self {}));
    }
}

impl DbKind for PartyType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("can_character_belong", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("province", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("ruler_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_value("description");
    }
}
