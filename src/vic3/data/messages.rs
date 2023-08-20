//! Notifications that are triggered with the `post_notification` effect.

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
    ItemLoader::Normal(GameFlags::Vic3, Item::Message, Message::add)
}

impl Message {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Message, key, block, Box::new(Self {}));
    }
}

impl DbKind for Message {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("notification_{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("notification_{key}_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("notification_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("type");
        vd.req_field("texture");
        vd.req_field("notification_type");

        vd.field_value("type"); // TODO: find out what is allowed here
        vd.field_item("group", Item::Localization);
        vd.field_item("texture", Item::File);
        vd.field_choice("notification_type", &["none", "feed", "toast", "popup"]);

        vd.field_integer("days");

        if block.field_value_is("notification_type", "popup") {
            vd.field_value("popup_name"); // TODO: find out what is allowed here
        } else {
            vd.ban_field("popup_name", || "`notification_type = popup`");
        }

        vd.field_item("on_created_soundeffect", Item::Sound);
    }
}
