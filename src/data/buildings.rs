use crate::block::validator::Validator;
use crate::block::{Block, BV};
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
pub struct Building {
    is_upgrade: bool,
}

impl Building {
    pub fn new() -> Self {
        Self { is_upgrade: false }
    }

    pub fn add(db: &mut Db, key: Token, block: Block) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::BuildingFlag, token);
        }
        if block.field_value_is("type", "special") {
            db.add_flag(Item::SpecialBuilding, key.clone());
        }
        db.add(Item::Building, key, block, Box::new(Self::new()));
    }

    pub fn finalize(db: &mut Db) {
        let mut upgrades = Vec::new();
        for (_, block, _) in db.iter_itype(Item::Building) {
            if let Some(token) = block.get_field_value("next_building") {
                upgrades.push(token.to_string());
            }
        }
        for upgrade in upgrades {
            db.set_property(Item::Building, &upgrade, "is_upgrade");
        }
    }
}

impl DbKind for Building {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let graphical_only = block
            .get_field_bool("is_graphical_background")
            .unwrap_or(false);
        if !graphical_only {
            let loca = format!("building_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("building_{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            if !self.is_upgrade {
                let loca = format!("building_type_{key}");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("building_type_{key}_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
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
            sc.define_name("holder", Scopes::Character, key.clone());
            sc.define_name("county", Scopes::LandedTitle, key.clone());
            let tooltipped = if graphical_only {
                Tooltipped::No
            } else {
                Tooltipped::FailuresOnly
            };
            validate_normal_trigger(block, data, sc, tooltipped);
        });
        vd.field_validated_key_block("can_construct_potential", |key, block, data| {
            let mut sc = ScopeContext::new_root(Scopes::Province, key.clone());
            sc.define_name("holder", Scopes::Character, key.clone());
            sc.define_name("county", Scopes::LandedTitle, key.clone());
            // For buildings that are upgrades, can_construct_potential is added to can_construct_showing_failures_only so it will be tooltipped
            let tooltipped =
                block.get_field_bool("show_disabled").unwrap_or(false) || self.is_upgrade;
            let tooltipped = if tooltipped {
                Tooltipped::FailuresOnly
            } else {
                Tooltipped::No
            };
            validate_normal_trigger(block, data, &mut sc, tooltipped);
        });
        vd.field_validated_block_rooted(
            "can_construct_showing_failures_only",
            Scopes::Province,
            |block, data, sc| {
                sc.define_name("holder", Scopes::Character, key.clone());
                sc.define_name("county", Scopes::LandedTitle, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::FailuresOnly);
            },
        );
        vd.field_validated_block_rooted("can_construct", Scopes::Province, |block, data, sc| {
            sc.define_name("holder", Scopes::Character, key.clone());
            sc.define_name("county", Scopes::LandedTitle, key.clone());
            validate_normal_trigger(block, data, sc, Tooltipped::Yes);
        });
        vd.field_bool("show_disabled");

        vd.field_script_value_rooted("cost_gold", Scopes::Character);
        vd.field_script_value_rooted("cost_piety", Scopes::Character);
        vd.field_script_value_rooted("cost_prestige", Scopes::Character);

        vd.field_item("next_building", Item::Building);
        vd.field_validated_rooted("effect_desc", Scopes::None, validate_desc);

        vd.field_validated_blocks("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_blocks("character_culture_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("parameter");
            vd.field_item("parameter", Item::CultureParameter);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_blocks("character_dynasty_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("county_holder_dynasty_perk");
            vd.field_item("county_holder_dynasty_perk", Item::DynastyPerk);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_validated_blocks("province_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });
        vd.field_validated_blocks("province_culture_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("parameter");
            vd.field_item("parameter", Item::CultureParameter);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });
        vd.field_validated_blocks("province_terrain_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("parameter", Item::CultureParameter);
            vd.field_item("terrain", Item::Terrain);
            vd.field_bool("is_coastal");
            validate_modifs(block, data, ModifKinds::Province, vd);
        });
        vd.field_validated_blocks("province_dynasty_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("county_holder_dynasty_perk");
            vd.field_item("county_holder_dynasty_perk", Item::DynastyPerk);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });

        vd.field_validated_blocks("county_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::County, vd);
        });
        vd.field_validated_blocks("county_culture_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("parameter");
            vd.field_item("parameter", Item::CultureParameter);
            validate_modifs(block, data, ModifKinds::County, vd);
        });

        if let Some(token) = block.get_field_value("type") {
            if token.is("duchy_capital") {
                vd.field_validated_blocks("duchy_capital_county_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::County, vd);
                });
                vd.field_validated_blocks(
                    "duchy_capital_county_culture_modifier",
                    |block, data| {
                        let mut vd = Validator::new(block, data);
                        vd.req_field("parameter");
                        vd.field_item("parameter", Item::CultureParameter);
                        validate_modifs(block, data, ModifKinds::County, vd);
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

        vd.field_validated_blocks("county_holding_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("holding");
            vd.field_item("holding", Item::Holding);
            validate_modifs(block, data, ModifKinds::County, vd);
        });
        vd.field_validated_blocks("county_dynasty_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("county_holder_dynasty_perk");
            vd.field_item("county_holder_dynasty_perk", Item::DynastyPerk);
            validate_modifs(block, data, ModifKinds::County, vd);
        });
        vd.field_validated_blocks("county_holder_character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_values("flag");

        vd.field_validated_block_rooted("on_complete", Scopes::Province, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });

        vd.field_validated_key("ai_value", |key, bv, data| match bv {
            BV::Value(token) => {
                token.expect_integer();
            }
            BV::Block(block) => {
                let mut sc = ScopeContext::new_root(Scopes::Province, key.clone());
                sc.define_name("holder", Scopes::Character, key.clone());
                sc.define_name("county", Scopes::LandedTitle, key.clone());
                validate_modifiers_with_base(block, data, &mut sc);
            }
        });

        vd.field_bool("is_graphical_background");
    }

    fn set_property(&mut self, _key: &Token, _block: &Block, property: &str) {
        if property == "is_upgrade" {
            self.is_upgrade = true;
        }
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

    vd.field_validated("soundeffect", |bv, data| {
        match bv {
            BV::Value(token) => data.verify_exists(Item::Sound, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("soundeffect");
                vd.field_item("soundeffect", Item::Sound);
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
