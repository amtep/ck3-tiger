use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Relation {}

impl Relation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(list) = block.get_field_list("flags") {
            for token in list {
                db.add_flag(Item::RelationFlag, token);
            }
        }
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
        vd.field_list("flags");
        vd.field_value("secret");
        vd.field_bool("special_guest");
        vd.field_bool("hidden");

        vd.field_validated_block_rooted("modifier", Scopes::Character, |block, data, sc| {
            let mut vd = Validator::new(block, data);
            vd.field_value("name");
            // TODO: "This cannot use any references to modifiers generated from other database objects,
            // such as seduce_scheme_power_mult (from schemes) or monthly_diplomacy_lifestyle_xp_gain_mult (from lifestyles)."
            validate_modifs(block, data, ModifKinds::Character, sc, vd);
        });
    }
}
