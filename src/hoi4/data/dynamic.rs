use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DynamicModifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::DynamicModifier, DynamicModifier::add)
}

impl DynamicModifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DynamicModifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for DynamicModifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_trigger_full("enable", Scopes::all_but_none(), Tooltipped::No);
        vd.field_trigger_full("remove_trigger", Scopes::all_but_none(), Tooltipped::No);
        vd.field_item("icon", Item::Sprite);
        vd.field_bool("attacker_modifier");
        validate_modifs(block, data, ModifKinds::all(), vd);
    }
}
