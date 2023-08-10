use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug)]
pub struct GovernmentType {}

impl GovernmentType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GovernmentType, key, block, Box::new(Self {}));
    }
}

impl DbKind for GovernmentType {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_integer("oratory_ideas");
        vd.field_integer("military_ideas");
        vd.field_integer("civic_ideas");
        vd.field_integer("religious_ideas");
        vd.field_integer("minimum_electable_age");
        vd.field_integer("election_delay");
        vd.field_integer("ruler_term");

        vd.field_bool("has_co_ruler");
        vd.field_bool("can_deify_ruler");
        vd.field_bool("revolt");
        vd.field_bool("ruler_consort_benefits");
        vd.field_bool("use_regnal_numbers");
        vd.field_bool("allows_migration");

        vd.field_choice("type", &["republic", "monarchy", "tribal"]);

        vd.field_validated_block("base", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        vd.field_validated_block("bonus", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
        
        vd.field_validated_block("color", validate_color);        
    }
}