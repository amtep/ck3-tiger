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
pub struct ProvinceRank {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::ProvinceRank, ProvinceRank::add)
}

impl ProvinceRank {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ProvinceRank, key, block, Box::new(Self {}));
    }
}

impl DbKind for ProvinceRank {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_numeric("holy_site_treasure_slots");
        vd.field_bool("default");
        vd.field_bool("is_established_city");

        vd.field_validated_block("rank_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });

        vd.field_validated_block("color", validate_color);
    }
}
