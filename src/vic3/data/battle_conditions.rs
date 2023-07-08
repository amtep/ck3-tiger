use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct BattleCondition {}

impl BattleCondition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::BattleCondition, key, block, Box::new(Self {}));
    }
}

impl DbKind for BattleCondition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Unit | ModifKinds::Battle, vd);
        });

        let mut sc = ScopeContext::new(Scopes::BattleSide, key);
        sc.define_name("is_advancing_side", Scopes::Bool, key); // undocumented
        sc.define_name("character", Scopes::Character, key); // undocumented

        vd.field_script_value("weight", &mut sc);
        // undocumented
        vd.field_validated_block("possible", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}
