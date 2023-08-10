use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::context::ScopeContext;
use crate::item::Item;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug)]
pub struct AiPlanGoals {}

impl AiPlanGoals {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AiPlanGoals, key, block, Box::new(Self {}));
    }
}

impl DbKind for AiPlanGoals {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_bool("default");
        // TODO - How to do the units like that?
        // levy_anatolian = {  #General Anatolian
        //     default = no

        //     light_infantry = 0.7
        //     light_cavalry = 0.15

        //     heavy_cavalry = 0.1
        //     chariots = 0.05
        // }
    }
}