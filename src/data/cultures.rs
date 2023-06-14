use crate::block::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct CultureEra {}

impl CultureEra {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureEra, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureEra {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("year");
        vd.field_integer("year");

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("invalid_for_government", Item::GovernmentType);
        vd.field_items("custom", Item::Localization);

        vd.field_validated_blocks_rooted(
            "character_modifier",
            Scopes::Character,
            |block, data, sc| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Character, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted("culture_modifier", Scopes::Culture, |block, data, sc| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Culture, sc, vd);
        });
        vd.field_validated_blocks_rooted(
            "county_modifier",
            Scopes::LandedTitle,
            |block, data, sc| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::County, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "province_modifier",
            Scopes::Province,
            |block, data, sc| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Province, sc, vd);
            },
        );

        vd.field_validated_blocks("maa_upgrade", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("men_at_arms", Item::MenAtArms);
            vd.field_numeric("pursuit");
            vd.field_numeric("screen");
            vd.field_numeric("damage");
            vd.field_numeric("toughness");
            vd.field_numeric("siege_value");
            vd.field_integer("siege_tier");
        });

        vd.field_items("unlock_building", Item::Building);
        vd.field_items("unlock_decision", Item::Decision);
        vd.field_items("unlock_casus_belli", Item::CasusBelli);
        vd.field_items("unlock_maa", Item::MenAtArms);
        vd.field_items("unlock_law", Item::Law);
    }

    // TODO: validate that none have the same year
}
