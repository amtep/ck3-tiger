use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::{validate_color, validate_maa_stats};

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
            validate_maa_stats(&mut vd);
        });

        vd.field_items("unlock_building", Item::Building);
        vd.field_items("unlock_decision", Item::Decision);
        vd.field_items("unlock_casus_belli", Item::CasusBelli);
        vd.field_items("unlock_maa", Item::MenAtArms);
        vd.field_items("unlock_law", Item::Law);
    }

    // TODO: validate that none have the same year
}

#[derive(Clone, Debug)]
pub struct Culture {}

impl Culture {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(list) = block.get_field_list("coa_gfx") {
            for token in list {
                db.add_flag(Item::CoaGfx, token);
            }
        }
        if let Some(list) = block.get_field_list("building_gfx") {
            for token in list {
                db.add_flag(Item::BuildingGfx, token);
            }
        }
        if let Some(list) = block.get_field_list("clothing_gfx") {
            for token in list {
                db.add_flag(Item::ClothingGfx, token);
            }
        }
        if let Some(list) = block.get_field_list("unit_gfx") {
            for token in list {
                db.add_flag(Item::UnitGfx, token);
            }
        }
        db.add(Item::Culture, key, block, Box::new(Self {}));
    }
}

impl DbKind for Culture {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_prefix");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_collective_noun");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_date("created");
        vd.field_list_items("parents", Item::Culture);

        vd.field_validated("color", |bv, data| match bv {
            BlockOrValue::Value(token) => data.verify_exists(Item::NamedColor, token),
            BlockOrValue::Block(block) => validate_color(block, data),
        });

        // TODO: check that these pillars are of the right kinds
        vd.field_item("ethos", Item::CulturePillar);
        vd.field_item("heritage", Item::CulturePillar);
        vd.field_item("language", Item::Language);
        vd.field_item("martial_custom", Item::CulturePillar);

        vd.field_list_items("traditions", Item::CultureTradition);
        vd.field_items("name_list", Item::NameList);

        vd.fields_list_items("coa_gfx", Item::Localization);
        vd.field_list_items("building_gfx", Item::Localization);
        vd.fields_list_items("clothing_gfx", Item::Localization);
        vd.field_list_items("unit_gfx", Item::Localization);

        vd.field_validated_block("ethnicities", |block, data| {
            let mut vd = Validator::new(block, data);
            for (_, value) in vd.integer_values() {
                data.verify_exists(Item::Ethnicity, value);
            }
        });

        vd.field_validated_blocks("dlc_tradition", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("trait");
            vd.req_field("requires_dlc_flag");
            vd.field_item("trait", Item::CultureTradition);
            vd.field_item("requires_dlc_flag", Item::DlcFeature);
            vd.field_item("fallback", Item::CultureTradition);
        });

        vd.field_item("history_loc_override", Item::Localization);
    }
}
