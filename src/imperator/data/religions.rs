use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Religion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Religion, Religion::add)
}

impl Religion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Religion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Religion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("desc_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("modifier");
        vd.req_field("color");
        vd.req_field("religion_category");

        vd.field_choice("religion_category", &["prophets", "pantheon", "firetemples", "sages"]);
        vd.field_bool("can_deify_ruler");
        vd.field_value("sacrifice_icon");
        vd.field_item("sacrifice_sound", Item::Sound);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("color", validate_color);
    }
}
