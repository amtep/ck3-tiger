use crate::block::Block;
use crate::ck3::validate::{validate_cost, validate_maa_stats};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_possibly_named_color;
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct CultureEra {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureEra, CultureEra::add)
}

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
        vd.multi_field_item("custom", Item::Localization);

        validate_modifiers(&mut vd);

        vd.multi_field_validated_block("maa_upgrade", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("men_at_arms", Item::MenAtArms);
            validate_maa_stats(&mut vd);
        });

        vd.multi_field_item("unlock_building", Item::Building);
        vd.multi_field_item("unlock_decision", Item::Decision);
        vd.multi_field_item("unlock_casus_belli", Item::CasusBelli);
        vd.multi_field_item("unlock_maa", Item::MenAtArms);
        vd.multi_field_item("unlock_law", Item::Law);
    }

    // TODO: validate that none have the same year
    // If they have the same year, the game gets confused about which era is later
}

#[derive(Clone, Debug)]
pub struct Culture {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Culture, Culture::add)
}

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

        vd.field_validated("color", validate_possibly_named_color);

        vd.field_item("ethos", Item::CultureEthos);
        vd.field_item("heritage", Item::CultureHeritage);
        vd.field_item("language", Item::Language);
        vd.field_item("martial_custom", Item::MartialCustom);

        vd.field_list_items("traditions", Item::CultureTradition);
        vd.multi_field_item("name_list", Item::NameList);

        vd.multi_field_list_items("coa_gfx", Item::Localization);
        vd.field_list_items("building_gfx", Item::Localization);
        vd.multi_field_list_items("clothing_gfx", Item::Localization);
        vd.field_list_items("unit_gfx", Item::Localization);

        vd.field_validated_block("ethnicities", |block, data| {
            let mut vd = Validator::new(block, data);
            for (_, value) in vd.integer_values() {
                data.verify_exists(Item::Ethnicity, value);
            }
        });

        vd.multi_field_validated_block("dlc_tradition", |block, data| {
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

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CulturePillar, CulturePillar::add)
}

impl CulturePillar {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("parameters") {
            for (key, value) in block.iter_assignments() {
                if value.is("yes") {
                    db.add_flag(Item::CultureParameter, key.clone());
                }
            }
        }
        if let Some(pillar) = block.get_field_value("type") {
            if pillar.is("language") {
                db.add_flag(Item::Language, key.clone());
            } else if pillar.is("ethos") {
                db.add_flag(Item::CultureEthos, key.clone());
            } else if pillar.is("heritage") {
                db.add_flag(Item::CultureHeritage, key.clone());
            } else if pillar.is("martial_custom") {
                db.add_flag(Item::MartialCustom, key.clone());
            }
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

        vd.multi_field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        validate_modifiers(&mut vd);

        let mut sc = ScopeContext::new(Scopes::Culture, key);
        sc.define_name("character", Scopes::Character, key);
        sc.define_list("traits", Scopes::CulturePillar | Scopes::CultureTradition, key); // undocumented
        vd.field_script_value("ai_will_do", &mut sc);
        vd.field_validated_block("is_shown", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("can_pick", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        if block.field_value_is("type", "language") {
            vd.field_validated("color", validate_possibly_named_color);
        } else {
            vd.ban_field("color", || "languages");
        }
        if block.field_value_is("type", "heritage") {
            vd.field_value("audio_parameter");
        } else {
            vd.ban_field("audio_parameter", || "heritages");
        }
        vd.field_validated_block("parameters", validate_parameters);
    }
}

#[derive(Clone, Debug)]
pub struct CultureTradition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureTradition, CultureTradition::add)
}

impl CultureTradition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("parameters") {
            for (key, value) in block.iter_assignments() {
                if value.is("yes") {
                    db.add_flag(Item::CultureParameter, key.clone());
                }
            }
        }
        if let Some(value) = block.get_field_value("category") {
            db.add_flag(Item::CultureTraditionCategory, value.clone());
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
        vd.field_validated_block("parameters", validate_parameters);
        vd.field_value("category");
        vd.field_validated_block("layers", |block, data| {
            let mut layer_path = Vec::new();
            if let Some(block) =
                data.get_defined_array_warn(key, "NGraphics|CULTURE_TRADITION_LAYER_PATHS")
            {
                for path in block.iter_values_warn() {
                    layer_path.push(path.as_str());
                }
            }

            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                if let Some(layer_idx) =
                    key.expect_integer().and_then(|i| match usize::try_from(i) {
                        Ok(u) if u < layer_path.len() => Some(u),
                        _ => {
                            let msg = format!(
                                "layer index out of range between 0 and {}",
                                layer_path.len()
                            );
                            err(ErrorKey::Range).msg(msg).loc(key).push();
                            None
                        }
                    })
                {
                    let loca = format!("{}/{}", layer_path[layer_idx], value);
                    data.verify_exists_implied(Item::Entry, &loca, value);
                }
            });
        });

        vd.field_validated_key_block("can_pick", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("replacing", Scopes::CultureTradition, key);
            sc.define_name("character", Scopes::Character, key);
            sc.define_list("traits", Scopes::CulturePillar | Scopes::CultureTradition, key); // undocumented
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_key_block("can_pick_for_hybridization", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("character", Scopes::Character, key);
            sc.define_list("traits", Scopes::CulturePillar | Scopes::CultureTradition, key); // undocumented
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        validate_modifiers(&mut vd);
        vd.multi_field_validated_block("doctrine_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("doctrine", Item::Doctrine);
            vd.field_item("name", Item::Localization);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_key_block("cost", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("replacing", Scopes::CultureTradition, key);
            sc.define_name("character", Scopes::Character, key);
            sc.define_list("traits", Scopes::CulturePillar | Scopes::CultureTradition, key); // undocumented
            validate_cost(block, data, &mut sc);
        });
        let mut sc = ScopeContext::new(Scopes::Culture, key);
        sc.define_name("character", Scopes::Character, key);
        sc.define_list("traits", Scopes::CulturePillar | Scopes::CultureTradition, key); // undocumented
        vd.field_validated_block("is_shown", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        sc.define_name("replacing", Scopes::CultureTradition, key);
        vd.field_script_value_no_breakdown("ai_will_do", &mut sc);
    }
}

fn validate_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, value| {
        if matches!(key.as_str(), "number_of_spouses" | "number_of_consorts") {
            ValueValidator::new(value, data).integer_range(0..);
        } else {
            ValueValidator::new(value, data).bool();
        }
        // culture parameter loca are lowercased, verified in 1.11
        let loca = format!("culture_parameter_{}", key.as_str().to_lowercase());
        data.verify_exists_implied(Item::Localization, &loca, key);
    });
}

fn validate_modifiers(vd: &mut Validator) {
    vd.multi_field_validated_block("character_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.multi_field_validated_block("culture_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Culture, vd);
    });
    vd.multi_field_validated_block("county_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::County, vd);
    });
    vd.multi_field_validated_block("province_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Province, vd);
    });
}

#[derive(Clone, Debug)]
pub struct CultureAesthetic {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureAesthetic, CultureAesthetic::add)
}

impl CultureAesthetic {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureAesthetic, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureAesthetic {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("name_list", Item::NameList);
        vd.field_list_items("building_gfx", Item::BuildingGfx);
        vd.field_list_items("clothing_gfx", Item::ClothingGfx);
        vd.field_list_items("unit_gfx", Item::UnitGfx);
        vd.field_list_items("coa_gfx", Item::CoaGfx);

        vd.field_validated_key_block("is_shown", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("character", Scopes::Character, key);
            sc.define_list("trait", Scopes::CultureTradition | Scopes::CulturePillar, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct CultureCreationName {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureCreationName, CultureCreationName::add)
}

impl CultureCreationName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureCreationName, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureCreationName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("culture", Scopes::Culture, key);
        if block.field_value_is("hybrid", "yes") {
            sc.define_name("other_culture", Scopes::Culture, key);
        }

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if !vd.field_validated_sc("collective_noun", &mut sc, validate_desc) {
            let loca = format!("{key}_collective_noun");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if !vd.field_validated_sc("prefix", &mut sc, validate_desc) {
            let loca = format!("{key}_prefix"); // docs say {key}_trigger
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_bool("hybrid");
    }
}

#[derive(Clone, Debug)]
pub struct NameEquivalency {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::NameEquivalency, NameEquivalency::add)
}

impl NameEquivalency {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::NameEquivalency, key, block, Box::new(Self {}));
    }
}

impl DbKind for NameEquivalency {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        for name in vd.values() {
            data.verify_exists(Item::Localization, name);
        }
    }
}
