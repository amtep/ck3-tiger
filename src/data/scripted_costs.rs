use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_cost;

#[derive(Clone, Debug)]
pub struct ScriptedCost {}

impl ScriptedCost {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedCost, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedCost {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
        if key.is("hybridize_culture") {
            sc.define_name("culture", Scopes::Culture, key.clone());
        } else if key.is("reforge_artifact") || key.is("repair_artifact") {
            sc = ScopeContext::new_root(Scopes::None, key.clone());
            sc.define_name("artifact", Scopes::Artifact, key.clone());
        } else if key.is("travel_leader") {
            sc.define_name("speed_aptitude", Scopes::Value, key.clone());
            sc.define_name("safety_aptitude", Scopes::Value, key.clone());
        } else if key.is("deactivate_accolade") {
            sc = ScopeContext::new_root(Scopes::Accolade, key.clone());
        } else if key.is("create_accolade") {
            sc.define_name("owner", Scopes::Character, key.clone());
        }

        validate_cost(block, data, &mut sc);
    }
}
