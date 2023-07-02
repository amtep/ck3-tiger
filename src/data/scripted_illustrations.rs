use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct ScriptedIllustration {}

impl ScriptedIllustration {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedIllustration, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedIllustration {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // TODO: validate the call from gui
        let mut sc = ScopeContext::new(Scopes::all(), key);

        vd.field_validated_bvs("texture", |bv, data| match bv {
            BV::Value(token) => validate_texture(key, token, data),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_validated_value("reference", validate_texture);
                vd.field_validated_block("trigger", |block, data| {
                    validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
                });
            }
        });
        vd.field_validated_blocks("environment", |block, data| {
            let mut vd = Validator::new(block, data);
            if let Some(token) = vd.field_value("reference") {
                data.verify_exists(Item::Environment, token);
            }
            vd.field_validated_block("trigger", |block, data| {
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            });
        });
    }
}

fn validate_texture(_key: &Token, token: &Token, data: &Everything) {
    let pathname = format!("gfx/interface/illustrations/{token}");
    data.verify_exists_implied(Item::File, &pathname, token);
}
