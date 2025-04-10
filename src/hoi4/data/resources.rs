use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Resource {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Resource, Resource::add)
}

impl Resource {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("resources") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::Resource, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `resources`";
            err(ErrorKey::FieldMissing).msg(msg).info(info).loc(key).push();
        }
    }
}

impl DbKind for Resource {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_integer("icon_frame");
        vd.field_numeric("cic");
        vd.field_numeric("convoys");
    }
}
