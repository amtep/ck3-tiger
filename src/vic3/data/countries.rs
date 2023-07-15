use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;

#[derive(Clone, Debug)]
pub struct Country {}

impl Country {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Country, key, block, Box::new(Self {}));
    }
}

impl DbKind for Country {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_ADJ");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated("color", validate_possibly_named_color);
        vd.field_choice(
            "country_type",
            &["recognized", "unrecognized", "colonial", "decentralized"],
        );
        vd.field_item("tier", Item::CountryTier);
        vd.field_list_items("cultures", Item::Culture);
        vd.field_item("religion", Item::Religion);
        vd.field_item("capital", Item::StateRegion);
        vd.field_bool("is_named_from_capital");

        vd.field_validated_key_block(
            "valid_as_home_country_for_separatists",
            |key, block, data| {
                // TODO: what is the scope type here?
                let mut sc = ScopeContext::new(Scopes::None, key);
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            },
        );
    }
}
