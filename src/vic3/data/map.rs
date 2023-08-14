use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{untidy, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MapLayer {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MapLayer, MapLayer::add)
}

impl MapLayer {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        // This function warns at `untidy` level because if map layers are actually missing,
        // they will be warned about by their users.
        if key.is("layer") {
            if let Some(name) = block.get_field_value("name") {
                db.add_exact_dup_ok(Item::MapLayer, name.clone(), block, Box::new(Self {}));
            } else {
                untidy(ErrorKey::FieldMissing).msg("unnamed map layer").loc(key).push();
            }
        } else {
            untidy(ErrorKey::UnknownField).weak().msg("unknown map layer item").loc(key).push();
        }
    }
}

impl DbKind for MapLayer {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name"); // this is the key

        vd.field_integer("fade_in");
        vd.field_integer("fade_out");

        // None of the following have examples in vanilla
        vd.field_value("category");
        vd.field_value("masks");
        vd.field_value("visibility_tags");
    }
}
