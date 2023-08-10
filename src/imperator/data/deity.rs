use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::context::ScopeContext;
use crate::scopes::Scopes;
use crate::trigger::validate_trigger;
use crate::effect::validate_effect;

#[derive(Clone, Debug)]
pub struct Deity {}

impl Deity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Deity, key, block, Box::new(Self {}));
    }
}

impl DbKind for Deity {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("omen_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_item("icon", Item::File);
        vd.field_validated_block("passive_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block("omen", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_item("religion", Item::Religion);
        vd.field_item("deity_category", Item::DeityCategory);
        vd.field_validated_block("deification_trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_activate", |b, data| {
            validate_effect(b, data, &mut sc, Tooltipped::Yes);
        });
    }
}