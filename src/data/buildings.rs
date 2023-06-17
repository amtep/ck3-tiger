use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_modifiers_with_base;

#[derive(Clone, Debug)]
pub struct Building {}

impl Building {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::BuildingFlag, token);
        }
        if block.field_value_is("type", "special") {
            db.add_flag(Item::SpecialBuilding, key.clone());
        }
        db.add(Item::Building, key, block, Box::new(Self {}));
    }
}

impl DbKind for Building {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !block
            .get_field_bool("is_graphical_background")
            .unwrap_or(false)
        {
            let loca = format!("building_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("building_{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            // TODO: figure out when the building_type_{key} and building_type_{key}_desc locas should exist
        }

        if let Some(icon) = vd.field_value("type_icon") {
            if let Some(icon_path) =
                data.get_defined_string_warn(icon, "NGameIcons|BUILDING_TYPE_ICON_PATH")
            {
                let pathname = format!("{icon_path}/{icon}");
                data.verify_exists_implied(Item::File, &pathname, icon);
            }
        }

        vd.field_script_value_rooted("levy", Scopes::None);
        vd.field_script_value_rooted("max_garrison", Scopes::None);
        vd.field_script_value_rooted("garrison_reinforcement_factor", Scopes::None);
        vd.field_script_value_rooted("construction_time", Scopes::None);
        vd.field_choice("type", &["regular", "special", "duchy_capital"]);

        vd.field_validated_blocks("asset", validate_asset);

        vd.field_validated_block_rooted("is_enabled", Scopes::Province, |block, data, sc| {
            sc.define_name("holder", key.clone(), Scopes::Character);
            validate_normal_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_validated_block_rooted(
            "can_construct_potential",
            Scopes::Province,
            |block, data, sc| {
                sc.define_name("holder", key.clone(), Scopes::Character);
                // TODO: for buildings that are upgrades, can_construct_potential is added to can_construct_showing_failures_only so it will be tooltipped
                let tooltipped = block.get_field_bool("show_disabled").unwrap_or(false);
                let tooltipped = if tooltipped {
                    Tooltipped::Yes
                } else {
                    Tooltipped::No
                };
                validate_normal_trigger(block, data, sc, tooltipped);
            },
        );
        vd.field_validated_block_rooted(
            "can_construct_showing_failures_only",
            Scopes::Province,
            |block, data, sc| {
                sc.define_name("holder", key.clone(), Scopes::Character);
                validate_normal_trigger(block, data, sc, Tooltipped::Yes);
            },
        );
        vd.field_validated_block_rooted("can_construct", Scopes::Province, |block, data, sc| {
            sc.define_name("holder", key.clone(), Scopes::Character);
            validate_normal_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_bool("show_disabled");

        vd.field_script_value_rooted("cost_gold", Scopes::Character);
        vd.field_script_value_rooted("cost_piety", Scopes::Character);
        vd.field_script_value_rooted("cost_prestige", Scopes::Character);

        vd.field_item("next_building", Item::Building);
        vd.field_validated_rooted("effect_desc", Scopes::None, validate_desc);

        vd.field_validated_blocks_rooted(
            "character_modifier",
            Scopes::Character,
            |block, data, sc| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Character, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "character_culture_modifier",
            Scopes::Character,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.req_field("parameter");
                vd.field_item("parameter", Item::CultureParameter);
                validate_modifs(block, data, ModifKinds::Character, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "character_dynasty_modifier",
            Scopes::Character,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.req_field("county_holder_dynasty_perk");
                vd.field_item("county_holder_dynasty_perk", Item::DynastyPerk);
                validate_modifs(block, data, ModifKinds::Character, sc, vd);
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
        vd.field_validated_blocks_rooted(
            "province_culture_modifier",
            Scopes::Province,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.req_field("parameter");
                vd.field_item("parameter", Item::CultureParameter);
                validate_modifs(block, data, ModifKinds::Province, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "province_terrain_modifier",
            Scopes::Province,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.field_item("parameter", Item::CultureParameter);
                vd.field_item("terrain", Item::Terrain);
                vd.field_bool("is_coastal");
                validate_modifs(block, data, ModifKinds::Province, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "province_dynasty_modifier",
            Scopes::Province,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.req_field("county_holder_dynasty_perk");
                vd.field_item("county_holder_dynasty_perk", Item::DynastyPerk);
                validate_modifs(block, data, ModifKinds::Province, sc, vd);
            },
        );

        vd.field_validated_blocks_rooted(
            "county_modifier",
            Scopes::LandedTitle,
            |block, data, sc| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::County, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "county_culture_modifier",
            Scopes::LandedTitle,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.req_field("parameter");
                vd.field_item("parameter", Item::CultureParameter);
                validate_modifs(block, data, ModifKinds::County, sc, vd);
            },
        );

        if let Some(token) = block.get_field_value("type") {
            if token.is("duchy_capital") {
                vd.field_validated_blocks_rooted(
                    "duchy_capital_county_modifier",
                    Scopes::LandedTitle,
                    |block, data, sc| {
                        let vd = Validator::new(block, data);
                        validate_modifs(block, data, ModifKinds::County, sc, vd);
                    },
                );
                vd.field_validated_blocks_rooted(
                    "duchy_capital_county_culture_modifier",
                    Scopes::LandedTitle,
                    |block, data, sc| {
                        let mut vd = Validator::new(block, data);
                        vd.req_field("parameter");
                        vd.field_item("parameter", Item::CultureParameter);
                        validate_modifs(block, data, ModifKinds::County, sc, vd);
                    },
                );
            } else {
                vd.ban_field("duchy_capital_county_modifier", || {
                    "duchy_capital buildings"
                });
                vd.ban_field("duchy_capital_county_culture_modifier", || {
                    "duchy_capital buildings"
                });
            }
        } else {
            vd.ban_field("duchy_capital_county_modifier", || {
                "duchy_capital buildings"
            });
            vd.ban_field("duchy_capital_county_culture_modifier", || {
                "duchy_capital buildings"
            });
        }

        vd.field_validated_blocks_rooted(
            "county_holding_modifier",
            Scopes::LandedTitle,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.req_field("holding");
                vd.field_item("holding", Item::Holding);
                validate_modifs(block, data, ModifKinds::County, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "county_dynasty_modifier",
            Scopes::LandedTitle,
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.req_field("county_holder_dynasty_perk");
                vd.field_item("county_holder_dynasty_perk", Item::DynastyPerk);
                validate_modifs(block, data, ModifKinds::County, sc, vd);
            },
        );
        vd.field_validated_blocks_rooted(
            "county_holder_character_modifier",
            Scopes::Character,
            |block, data, sc| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Character, sc, vd);
            },
        );

        vd.field_values("flag");

        vd.field_validated_block_rooted("on_complete", Scopes::Province, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });

        vd.field_validated_key("ai_value", |key, bv, data| match bv {
            BlockOrValue::Value(token) => {
                token.expect_integer();
            }
            BlockOrValue::Block(block) => {
                let mut sc = ScopeContext::new_root(Scopes::Province, key.clone());
                sc.define_name("holder", key.clone(), Scopes::Character);
                validate_modifiers_with_base(block, data, &mut sc);
            }
        });

        vd.field_bool("is_graphical_background");
    }
}

fn validate_asset(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("type");
    vd.field_choice("type", &["pdxmesh", "entity"]);

    let meshes = block.field_value_is("type", "pdxmesh");
    let itype = if meshes { Item::Pdxmesh } else { Item::Entity };

    vd.req_field_one_of(&["name", "names"]);
    vd.field_item("name", itype);
    vd.field_list_items("names", itype);

    vd.field_item("illustration", Item::File);

    // TODO: get a list of valid soundeffects from somewhere
    vd.field_validated("soundeffect", |bv, data| {
        match bv {
            BlockOrValue::Value(_) => (),
            BlockOrValue::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("soundeffect");
                vd.field_value("soundeffect");
                vd.field_block("soundparameter"); // TODO
            }
        }
    });

    vd.field_list_items("graphical_cultures", Item::BuildingGfx);
    vd.field_list_items("graphical_faiths", Item::GraphicalFaith);
    vd.field_list_items("graphical_regions", Item::Region); // TODO check that it's a graphical region
    vd.field_list_items("governments", Item::GovernmentType);

    vd.field_item("requires_dlc_flag", Item::DlcFeature);
}
