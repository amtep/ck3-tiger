use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Acclimatation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Acclimatation, Acclimatation::add)
}

impl Acclimatation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Acclimatation, key, block, Box::new(Self {}));
    }
}

impl DbKind for Acclimatation {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_validated_block("gain_when", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Modifier, key);
                value.expect_number();
            });
        });
        vd.field_numeric("weather_modifiers_reduction_factor");
        for field in &["change_camo_when", "forbid_camo_when"] {
            vd.field_validated_list(field, |value, _| {
                if !value.is("snow") {
                    data.verify_exists(Item::Terrain, value);
                }
            });
        }
    }
}
