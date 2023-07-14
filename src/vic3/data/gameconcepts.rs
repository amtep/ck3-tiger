use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct GameConcept {}

impl GameConcept {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GameConcept, key, block, Box::new(Self {}));
    }
}

impl DbKind for GameConcept {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("texture", Item::File);
        vd.field_bool("is_loading_tip");
        vd.field_item("tutorial_lesson", Item::TutorialLesson);
    }
}
