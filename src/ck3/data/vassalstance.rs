use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct VassalStance {}

impl VassalStance {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::VassalStance, key, block, Box::new(Self {}));
    }
}

impl DbKind for VassalStance {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("liege", Scopes::Character, key);

        let modif = format!("{key}_opinion");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_same_faith_opinion");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_same_culture_opinion");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_different_faith_opinion");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_different_culture_opinion");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_levy_contribution_add");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_levy_contribution_mult");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_tax_contribution_add");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        let modif = format!("{key}_tax_contribution_mult");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        vd.field_validated_blocks("is_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_script_value("score", &mut sc);
        vd.field_script_value("heir_score", &mut sc);
    }
}
