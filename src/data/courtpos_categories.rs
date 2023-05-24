use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::{DbKind, Everything};
use crate::item::Item;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct CourtPositionCategory {}

impl CourtPositionCategory {
    pub fn boxed_new(_key: &Token, _block: &Block) -> Box<dyn DbKind> {
        Box::new(Self {})
    }
}

impl DbKind for CourtPositionCategory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
    }
}
