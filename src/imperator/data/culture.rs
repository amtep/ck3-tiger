use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::context::ScopeContext;
use crate::item::Item;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_color;

#[derive(Clone, Debug)]
pub struct CultureGroup {}

impl CultureGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("color", validate_color);
        vd.field_item("primary", Item::Unit);
        vd.field_item("second", Item::Unit);
        vd.field_item("flank", Item::Unit);
        vd.field_item("primary_navy", Item::Unit);
        vd.field_item("secondary_navy", Item::Unit);
        vd.field_item("flank_navy", Item::Unit);
        vd.field_item("levy_template", Item::LevyTemplate);
        vd.field_item("graphical_culture", Item::GraphicalCultureType);

        vd.field_list("male_names");
        vd.field_list("female_names");
        vd.field_list("family");
        vd.field_list("barbarian_names");

        // TODO - How do I do the Culture item here? cultures are defined inside of the CultureGroup item...

        vd.accepted_block_fields = true;
        // TODO - Not sure what to do with ethnicities...
        // vd.field_validated_block("ethnicities", |block, data| {
        // });
    }
}