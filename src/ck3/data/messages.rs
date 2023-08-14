use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Message {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Message, Message::add)
}

impl Message {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Message, key, block, Box::new(Self {}));
    }
}

impl DbKind for Message {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("display", &["feed", "toast"]);
        vd.field_item("title", Item::Localization); // docs say text
        vd.field_item("desc", Item::Localization);
        vd.field_item("tooltip", Item::Localization);
        vd.field_item("soundeffect", Item::Sound);
        if let Some(icon) = vd.field_value("icon") {
            // docs say message_icons
            let pathname = format!("gfx/interface/icons/message_feed/{icon}.dds");
            data.verify_exists_implied(Item::File, &pathname, icon);
        }
        vd.field_choice("style", &["good", "bad", "neutral"]);

        vd.field_list("flags"); // docs say integers but they are text
    }
}
