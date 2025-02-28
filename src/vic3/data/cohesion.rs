use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CohesionLevel {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CohesionLevel, CohesionLevel::add)
}

impl CohesionLevel {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CohesionLevel, key, block, Box::new(Self {}));
    }
}

impl DbKind for CohesionLevel {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        // Entire item is undocumented.

        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_numeric("threshold");

        vd.field_item("background_texture", Item::File);

        vd.field_numeric("ai_unification_support_score");

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::PowerBloc, vd);
        });
    }
}
