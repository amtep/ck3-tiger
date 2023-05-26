use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Relation {}

impl Relation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Relation, key, block, Box::new(Self {}));
    }
}

impl DbKind for Relation {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_item("corresponding", Item::Relation);
        vd.field_bool("title_grant_target");
        vd.field_list_items("opposites", Item::Relation);
        vd.field_list_items("relation_aliases", Item::Relation);
        vd.field_integer("opinion");
        vd.field_numeric("fertility");
        for (key, _) in vd.integer_values() {
            let val = key.as_str().parse::<i32>().unwrap();
            if !(0..=15).contains(&val) {
                error(key, ErrorKey::Validation, "flag value out of range");
            }
        }
        vd.field_value("secret");
        vd.field_bool("special_guest");
        vd.field_bool("hidden");

        if let Some((key, block)) = vd.definition("modifier") {
            let mut vd = Validator::new(block, data);
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            vd.field_value("name");
            // TODO: "This cannot use any references to modifiers generated from other database objects,
            // such as seduce_scheme_power_mult (from schemes) or monthly_diplomacy_lifestyle_xp_gain_mult (from lifestyles)."
            validate_modifs(block, data, ModifKinds::Character, &mut sc, vd);
        }
    }
}
