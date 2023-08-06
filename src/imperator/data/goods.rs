use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct TradeGood {}

impl TradeGood {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TradeGoods, key, block, Box::new(Self {}));
    }
}

impl DbKind for TradeGood {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}DESC");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_numeric("category");
        vd.field_numeric("gold");

        // Unit types are optional
        vd.field_item("allow_unit_type", Item::UnitType);

        vd.field_validated_block("province", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });

        vd.field_validated_block("country", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("color", validate_color);
    }
}
