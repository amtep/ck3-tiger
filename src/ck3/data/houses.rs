use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::Severity;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct House {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::House, House::add)
}

impl House {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::House, key, block, Box::new(Self {}));
    }

    pub fn get_dynasty<'a>(key: &str, data: &'a Everything) -> Option<&'a Token> {
        data.database
            .get_key_block(Item::House, key)
            .and_then(|(_, block)| block.get_field_value("dynasty"))
    }
}

impl DbKind for House {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("name");
        vd.req_field("dynasty");

        vd.field_item("name", Item::Localization);
        vd.field_item("prefix", Item::Localization);
        vd.field_item("motto", Item::Localization);
        vd.field_item("dynasty", Item::Dynasty);
        vd.field_value("forced_coa_religiongroup"); // TODO
    }
}

#[derive(Clone, Debug)]
pub struct HousePowerBonus {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::HousePowerBonus, HousePowerBonus::add)
}

impl HousePowerBonus {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::HousePowerBonus, key, block, Box::new(Self {}));
    }
}

impl DbKind for HousePowerBonus {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_house_power");
        data.verify_exists_implied(Item::Localization, &loca, key);

        if let Some(icon_path) =
            data.get_defined_string_warn(key, "NGameIcons|HOUSE_POWER_BONUS_ICON_PATH")
        {
            let pathname = format!("{icon_path}/{key}.dds");
            data.verify_exists_implied_max_sev(Item::File, &pathname, key, Severity::Warning);
        }

        vd.field_validated_block("liege_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_validated_block("member_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_item("illustration", Item::File);
    }
}
