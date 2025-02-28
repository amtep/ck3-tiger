use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Institution {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Institution, Institution::add)
}

impl Institution {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Institution, key, block, Box::new(Self {}));
    }
}

impl DbKind for Institution {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("icon");
        vd.field_item("icon", Item::File);
        vd.req_field("background_texture");
        vd.field_item("background_texture", Item::File);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
    }
}
