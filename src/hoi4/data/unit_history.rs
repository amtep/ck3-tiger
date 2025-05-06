use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashMap;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::pdxfile::PdxEncoding;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct UnitHistory {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Hoi4, Item::UnitHistory, PdxEncoding::Utf8NoBom, ".txt", LoadAsFile::Yes, Recursive::No, UnitHistory::add)
}

impl UnitHistory {
    pub fn add(db: &mut Db, file: Token, block: Block) {
        db.add(Item::UnitHistory, file, block, Box::new(Self {}));
    }
}

impl DbKind for UnitHistory {
    fn add_subitems(&self, _file: &Token, block: &Block, db: &mut Db) {
        for block in block.get_field_blocks("division_template") {
            if let Some(name) = block.get_field_value("name") {
                db.add_flag(Item::DivisionTemplate, name.clone());
            }
        }
        for block in block.get_field_blocks("units") {
            for block in block.get_field_blocks("fleet") {
                for block in block.get_field_blocks("task_force") {
                    for block in block.get_field_blocks("ship") {
                        if let Some(name) = block.get_field_value("name") {
                            db.add_flag(Item::ShipName, name.clone());
                        }
                    }
                }
            }
        }
    }

    fn validate(&self, _file: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.multi_field_validated_block("division_template", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.field_value("name");
            vd.field_item("division_names_group", Item::DivisionNamesGroup);
            vd.field_value("role"); // TODO: where are these defined?
            vd.field_validated_block("regiments", |block, data| {
                let mut vd = Validator::new(block, data);
                let mut used = TigerHashMap::default();
                vd.unknown_block_fields(|key, block| {
                    // TODO: verify these are not support units
                    data.verify_exists(Item::SubUnit, key);
                    validate_position(key, block, data, &mut used, "regiment");
                });
            });
            vd.field_validated_block("support", |block, data| {
                let mut vd = Validator::new(block, data);
                let mut used = TigerHashMap::default();
                vd.unknown_block_fields(|key, block| {
                    // TODO: verify these are support units
                    data.verify_exists(Item::SubUnit, key);
                    validate_position(key, block, data, &mut used, "support");
                });
            });
            vd.field_integer("priority");
            vd.field_integer("template_counter");
            vd.field_bool("is_locked");
        });

        vd.field_validated_block("units", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_validated_block("division", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_value("name");
                vd.field_validated_block("division_name", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_bool("is_name_ordered");
                    vd.field_integer("name_order");
                });
                vd.field_validated_block("officer", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_value("name");
                });
                vd.field_item("location", Item::Province);
                // TODO: check that the template belongs to this country
                vd.field_item("division_template", Item::DivisionTemplate);
                vd.field_numeric("start_experience_factor");
                vd.field_numeric("start_equipment_factor");
                vd.field_validated_block("force_equipment_variants", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.validate_item_key_blocks(Item::Equipment, |_, block, data| {
                        let mut vd = Validator::new(block, data);
                        vd.field_item("owner", Item::CountryTag);
                        vd.field_item("creator", Item::CountryTag);
                        vd.field_value("version_name");
                    });
                });
            });
            vd.multi_field_validated_block("fleet", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_value("name");
                // TODO: check that province is part of country
                vd.field_item("naval_base", Item::Province);
                vd.multi_field_validated_block("task_force", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_value("name");
                    vd.field_item("location", Item::Province);
                    vd.multi_field_validated_block("ship", |block, data| {
                        let mut vd = Validator::new(block, data);
                        vd.field_value("name");
                        vd.field_item("definition", Item::SubUnit);
                        vd.field_numeric("start_experience_factor");
                        vd.field_bool("pride_of_the_fleet");
                        vd.field_validated_block("equipment", |block, data| {
                            let mut vd = Validator::new(block, data);
                            vd.validate_item_key_blocks(Item::Equipment, |_, block, data| {
                                let mut vd = Validator::new(block, data);
                                vd.field_integer("amount");
                                vd.field_item("owner", Item::CountryTag);
                                vd.field_item("creator", Item::CountryTag);
                                vd.field_value("version_name");
                            });
                        });
                    });
                });
            });
        });

        vd.field_validated_block("air_wings", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                if key.is_integer() {
                    // TODO: check that the state is part of the country
                    data.verify_exists(Item::State, key);
                } else {
                    // TODO: "the file containing the carrier must be loaded before the file
                    // containing the air wing"
                    // TODO: check that the named ship is a carrier
                    data.verify_exists(Item::ShipName, key);
                }
                let mut vd = Validator::new(block, data);
                let mut expect_name = false;
                vd.unknown_fields(|key, bv| {
                    if key.is("name") {
                        if !expect_name {
                            let msg = "unexpected `name` field";
                            let info = "the `name` field should go after an air wing definition";
                            warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
                        }
                        bv.expect_value();
                        expect_name = false;
                    } else if key.is("ace") {
                        if let Some(block) = bv.expect_block() {
                            let mut vd = Validator::new(block, data);
                            vd.field_item("modifier", Item::AceModifier);
                            vd.field_value("name");
                            vd.field_value("surname");
                            vd.field_value("callsign");
                            vd.field_bool("is_female");
                        }
                    } else {
                        data.verify_exists(Item::EquipmentBonusType, key);
                        if let Some(block) = bv.expect_block() {
                            let mut vd = Validator::new(block, data);
                            vd.field_item("owner", Item::CountryTag);
                            vd.field_integer("amount");
                            vd.field_item("creator", Item::CountryTag);
                            vd.field_value("version_name");
                            expect_name = true;
                        }
                    }
                });
            });
        });

        vd.field_effect_rooted("instant_effect", Tooltipped::No, Scopes::Country);
    }
}

fn validate_position(
    key: &Token,
    block: &Block,
    data: &Everything,
    used: &mut TigerHashMap<(&str, &str), Token>,
    what: &str,
) {
    let mut vd = Validator::new(block, data);
    vd.field_integer("x");
    vd.field_integer("y");

    // Check for duplicate positions
    if let Some(x) = block.get_field_value("x") {
        if let Some(y) = block.get_field_value("y") {
            let index = (x.as_str(), y.as_str());
            if let Some(other) = used.insert(index, key.clone()) {
                let msg = format!("duplicate {what} position ({x}, {y})");
                let msg_other = "the other one is here";
                warn(ErrorKey::Validation).msg(msg).loc(key).loc_msg(other, msg_other).push();
            }
        }
    }
}
