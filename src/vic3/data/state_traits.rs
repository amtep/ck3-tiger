use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct StateTrait {}

impl StateTrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::StateTrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for StateTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.req_field("icon");
        vd.field_item("icon", Item::File);

        vd.field_list_items("required_techs_for_colonization", Item::Technology);
        vd.field_list_items("disabling_technologies", Item::Technology);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
    }
}
