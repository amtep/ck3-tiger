use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::{Item, ItemLoader};
use crate::game::GameFlags;
use crate::token::Token;
use crate::validator::Validator;
use crate::modif::{validate_modifs, ModifKinds};
use crate::imperator::tables::misc::{DLC_IMPERATOR};

#[derive(Clone, Debug)]
pub struct SetupMain {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::SetupMain, SetupMain::add)
}

impl SetupMain {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SetupMain, key, block, Box::new(Self {}));
    }
}

impl DbKind for SetupMain {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("treasure_manager", |block, data| {
            validate_treasures(block, data);
        });
        vd.field_validated_block("family", |block, data| {
            validate_families(block, data);
        });
        vd.field_validated_block("diplomacy", |block, data| {
            validate_diplomacy(block, data);
        });
        vd.field_validated_block("provinces", |block, data| {
            validate_provinces(block, data);
        });
        vd.field_validated_block("road_network", |block, data| {
            validate_roads(block, data);
        });
        vd.field_validated_block("country", |block, data| {
            validate_countries(block, data);
        });
        vd.field_validated_block("trade", |block, data| {
            validate_trade(block, data);
        });
        vd.field_validated_block("great_work_manager", |block, data| {
            validate_great_works(block, data);
        });
    }
}

fn validate_treasures(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("database", |block, data| {
        let mut vd = Validator::new(block, data);
        for (_, block) in vd.integer_blocks() {
            let mut vd = Validator::new(block, data);
            // TODO - This key field should be saved in the db as a new Treasure Item. The Treasure Item would then need to be added to the `treasure:X` scope
            // How do I save the key as a new db Item type here??? Not sure how to do this right now.
            vd.field_item("key", Item::Localization);
            vd.field_choice("dlc", DLC_IMPERATOR);
            vd.field("icon");
            vd.multi_field_validated_block("state_modifier", |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Country | ModifKinds::Province | ModifKinds::State, vd);
            });
        }
    });
}

fn validate_families(block: &Block, data: &Everything) {
    // TODO - imperator - Family should be its own Item type with the "key" field being the name of each one. 
    // Then the "fam:" link should be updated with the new item.
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("families", |block, data| {
        let mut vd = Validator::new(block, data);
        for (_, block) in vd.integer_blocks() {
            let mut vd = Validator::new(block, data);
            vd.field_item("key", Item::Localization);
            vd.field_item("owner", Item::Localization); // can be any country tag declared in setup
            vd.field_item("culture", Item::Localization);
            vd.field_integer("prestige");
            vd.field_integer("color");
        }
    });
}
fn validate_diplomacy(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.multi_field_validated_block("defensive_league", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.multi_field_item("member", Item::Localization);
    });
    for field in &["dependency", "guarantee", "alliance"] {
        vd.multi_field_validated_block(field, |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("first", Item::Localization);
            vd.field_item("second", Item::Localization);
            if field == &"dependency" {
                vd.field_item("subject_type", Item::SubjectType);
            }
        });
    }
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::Localization, key);
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, _value| {
            let mut vd = Validator::new(block, data);
            data.verify_exists(Item::Localization, key);
            vd.field_bool("trade_access");
        });
    });
}
fn validate_provinces(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    for (_, block) in vd.integer_blocks() {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("treasure_slots", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("treasures", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_integers_at_least(1);
            });
        });
        vd.field_validated_block("modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_item("modifier", Item::Modifier);
            vd.field_bool("always");
        });
        vd.field_integer("great_work");
        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Building, key);
            value.expect_number();
        });
    }
}
fn validate_roads(block: &Block, data: &Everything) {
    // This is just 2 provinces connected like
    let mut vd = Validator::new(block, data);

    vd.unknown_value_fields(|_key, value| {
        value.expect_number();
    });
}
fn validate_countries(block: &Block, data: &Everything) {
    /*
        Example:
        <country> = {
            family = 0
            family = 1
            family = 2
            government = <government>
            diplomatic_stance=<diplo stance>
            primary_culture = <culture>
            religion = <religion>
            
            technology={
                military_tech={ level=2 progress=0 }
                civic_tech={ level=2 progress=0 }
                oratory_tech={ level=2 progress=0 }
                religious_tech= { level=2 progress=0  }
            }
            
            capital = 1
            pantheon = {
            
            { deity = 1 }
            { deity = 2 } 
            { deity = 6 } 
            { deity = 4 }
            
            }
            is_antagonist = yes
        
            treasures = { 201 61 39 }
        
            own_control_core =  {
                1 2 3 4 5 6 7 8 15 16 18 19 20 24 25 26 27 31 37 40 36 39 50
            }
            
            succession_law=egyption_succession_law
            <law_group> = <law_type>
        }
    */

    let mut vd = Validator::new(block, data);

    vd.field_validated_block("countries", |block, data| {
        let mut vd = Validator::new(block, data);

        vd.validated_blocks(|block, data| {
            let mut vd = Validator::new(block, data);

            vd.field_item("government", Item::GovernmentType);
            vd.field_item("diplomatic_stance", Item::DiplomaticStance);
            vd.field_item("religion", Item::Religion);
            vd.field_item("culture", Item::Culture);

            vd.field_integer("family");
            vd.field_integer("capital");
            vd.field_bool("is_antagonist");

            vd.field_validated_block("treasures", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_integers_at_least(1);
            });
            vd.field_validated_block("own_control_core", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_integers_at_least(1);
            });

            vd.field_validated_block("technology", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_validated_block("military_tech", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_integer("level");
                    vd.field_integer("progress");
                });
                vd.field_validated_block("civic_tech", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_integer("level");
                    vd.field_integer("progress");
                });
                vd.field_validated_block("oratory_tech", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_integer("level");
                    vd.field_integer("progress");
                });
                vd.field_validated_block("religious_tech", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_integer("level");
                    vd.field_integer("progress");
                });
            });

            vd.field_validated_block("pantheon", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validated_blocks(|block, data| {
                    let mut vd = Validator::new(block, data);
                    // TODO - it should be possible to validate if this is actually a valid value
                    // by making sure that the integer is defined in setup/main/deities files. Probably would need to setup a new DeityId Item or something.
                    vd.field_integer("deity");
                });
            });

            vd.field_choice(
                "succession",
                &[
                    "elective_monarchy",
                    "old_egyptian_succession",
                    "agnatic",
                    "cognatic",
                    "agnatic_seniority",
                ],
            );

            vd.unknown_value_fields(|key, _value| {
                data.verify_exists(Item::LawGroup, key);
                data.verify_exists(Item::Law, key);
            });
        });
    });
}

fn validate_trade(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.multi_field_validated_block("route", |block, data| {
        let mut vd = Validator::new(block, data);

        vd.field_integer("from");
        vd.field_integer("to");
        vd.field_item("trade_goods", Item::TradeGood);
    });
}

fn validate_great_works(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("great_works_database", |block, data| {
        let mut vd = Validator::new(block, data);
        for (_, block) in vd.integer_blocks() {
            let mut vd = Validator::new(block, data);
            vd.field_bool("ancient_wonder");
            vd.field("key");
            vd.field_choice("great_work_state", &["great_work_state_completed"]);
            vd.field_item("great_work_category", Item::GreatWorkCategory);
            vd.field_date("finished_date");

            vd.field_validated_block("great_work_name", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("name", Item::Localization);
            });

            vd.field_validated_block("great_work_components", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validated_blocks(|block, data| {
                    let mut vd = Validator::new(block, data);

                    vd.field_item("great_work_module", Item::GreatWorkModule);
                    vd.field_item("great_work_material", Item::GreatWorkMaterial);
                });
            });

            vd.field_validated_block("great_work_effect_selections", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validated_blocks(|block, data| {
                    let mut vd = Validator::new(block, data);

                    vd.field_item("great_work_effect", Item::GreatWorkEffect);
                    vd.field_item("great_work_effect_tier", Item::GreatWorkEffectTier);
                });
            });
        }
    });
}