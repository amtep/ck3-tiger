use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct AceModifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::AceModifier, AceModifier::add)
}

impl AceModifier {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("modifiers") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::AceModifier, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for AceModifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field("type"); // TODO what are these?
        vd.field_numeric("chance");
        vd.field_validated_block("effect", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Air, vd);
        });
    }
}
