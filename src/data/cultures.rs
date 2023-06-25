use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_color, validate_cost, validate_maa_stats};

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

        validate_modifiers(&mut vd);

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

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_prefix");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_collective_noun");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_date("created");
        vd.field_list_items("parents", Item::Culture);

        vd.field_validated("color", |bv, data| match bv {
            BV::Value(token) => data.verify_exists(Item::NamedColor, token),
            BV::Block(block) => validate_color(block, data),
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

#[derive(Clone, Debug)]
pub struct CulturePillar {}

impl CulturePillar {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("parameters") {
            for (key, value) in block.iter_assignments() {
                if value.is("yes") {
                    db.add_flag(Item::CultureParameter, key.clone());
                }
            }
        }
        if block.field_value_is("type", "language") {
            db.add_flag(Item::Language, key.clone());
        }
        db.add(Item::CulturePillar, key, block, Box::new(Self {}));
    }
}

impl DbKind for CulturePillar {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_choice("type", &["ethos", "heritage", "language", "martial_custom"]);
        vd.field_item("name", Item::Localization);
        if !block.has_key("name") {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        if block.field_value_is("type", "ethos") {
            vd.field_item("desc", Item::Localization);
            if !block.has_key("desc") {
                let loca = format!("{key}_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        } else if block.field_value_is("type", "heritage") {
            let loca = format!("{key}_collective_noun");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_validated_blocks("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        validate_modifiers(&mut vd);

        let mut sc = ScopeContext::new_root(Scopes::Culture, key.clone());
        sc.define_name("character", Scopes::Character, key.clone());
        vd.field_script_value("ai_will_do", &mut sc);
        vd.field_validated_block("is_shown", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("can_pick", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated("color", |bv, data| match bv {
            BV::Value(token) => data.verify_exists(Item::NamedColor, token),
            BV::Block(block) => validate_color(block, data),
        });
        vd.field_block("parameters");
    }
}

#[derive(Clone, Debug)]
pub struct CultureTradition {}

impl CultureTradition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("parameters") {
            for (key, value) in block.iter_assignments() {
                if value.is("yes") {
                    db.add_flag(Item::CultureParameter, key.clone());
                }
            }
        }
        db.add(Item::CultureTradition, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureTradition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_item("name", Item::Localization);
        if !block.has_key("name") {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        vd.field_item("desc", Item::Localization);
        if !block.has_key("desc") {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        vd.field_block("parameters");
        vd.field_value("category");
        vd.field_block("layers"); // TODO

        vd.field_validated_block_rooted("can_pick", Scopes::Culture, |block, data, sc| {
            validate_normal_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_block_rooted(
            "can_pick_for_hybridization",
            Scopes::Culture,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, Tooltipped::Yes);
            },
        );
        validate_modifiers(&mut vd);
        vd.field_validated_blocks("doctrine_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("doctrine", Item::Doctrine);
            vd.field_item("name", Item::Localization);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_key_block("cost", |key, block, data| {
            let mut sc = ScopeContext::new_root(Scopes::Culture, key.clone());
            sc.define_name("replacing", Scopes::Bool, key.clone());
            sc.define_name("character", Scopes::Character, key.clone());
            validate_cost(block, data, &mut sc)
        });
        let mut sc = ScopeContext::new_root(Scopes::Culture, key.clone());
        sc.define_name("character", Scopes::Character, key.clone());
        vd.field_script_value("ai_will_do", &mut sc);
        vd.field_validated_block("is_shown", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

fn validate_modifiers(vd: &mut Validator) {
    vd.field_validated_blocks("character_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.field_validated_blocks("culture_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Culture, vd);
    });
    vd.field_validated_blocks("county_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::County, vd);
    });
    vd.field_validated_blocks("province_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Province, vd);
    });
}
