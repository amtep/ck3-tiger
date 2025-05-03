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
pub struct ScientistTrait {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::ScientistTrait, ScientistTrait::add)
}

impl ScientistTrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScientistTrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScientistTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if let Some(name) = vd.field_value("name") {
            data.verify_exists(Item::Localization, name);
        } else {
            data.verify_exists(Item::Localization, key);
        }

        if let Some(icon) = vd.field_value("icon") {
            data.verify_exists(Item::Sprite, icon);
        } else {
            let sprite = format!("GFX_{key}");
            data.verify_exists_implied(Item::Sprite, &sprite, key);
        }

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_list_items("specialization", Item::Specialization);
        vd.field_trigger_rooted("available", Tooltipped::No, Scopes::Country);
    }
}
