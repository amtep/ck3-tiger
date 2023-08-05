use crate::block::Block;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct ScriptedCost {}

impl ScriptedCost {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedCost, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedCost {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        if key.is("hybridize_culture") {
            sc.define_name("culture", Scopes::Culture, key);
        } else if key.is("reforge_artifact") || key.is("repair_artifact") {
            sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("artifact", Scopes::Artifact, key);
        } else if key.is("travel_leader") {
            sc.define_name("speed_aptitude", Scopes::Value, key);
            sc.define_name("safety_aptitude", Scopes::Value, key);
        } else if key.is("deactivate_accolade") {
            sc = ScopeContext::new(Scopes::Accolade, key);
        } else if key.is("create_accolade") {
            sc.define_name("owner", Scopes::Character, key);
        }

        validate_cost(block, data, &mut sc);
    }
}
