use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::hoi4::tables::modifs::lookup_modif;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::{Builder, Validator};

#[derive(Clone, Debug)]
pub struct IndustrialOrg {}
#[derive(Clone, Debug)]
pub struct IndustrialOrgPolicy {}
#[derive(Clone, Debug)]
pub struct IndustrialOrgBonusWeight {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::IndustrialOrg, IndustrialOrg::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::IndustrialOrgPolicy, IndustrialOrgPolicy::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::IndustrialOrgBonusWeight, IndustrialOrgBonusWeight::add)
}

impl IndustrialOrg {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for block in block.get_field_blocks("trait") {
            if let Some(token) = block.get_field_value("token") {
                db.add_flag(Item::IndustrialOrgTrait, token.clone());
            }
        }
        for block in block.get_field_blocks("add_trait") {
            if let Some(token) = block.get_field_value("token") {
                db.add_flag(Item::IndustrialOrgTrait, token.clone());
            }
        }
        db.add(Item::IndustrialOrg, key, block, Box::new(Self {}));
    }
}

impl IndustrialOrgPolicy {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::IndustrialOrgPolicy, key, block, Box::new(Self {}));
    }
}

impl IndustrialOrgBonusWeight {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::IndustrialOrgBonusWeight, key, block, Box::new(Self {}));
    }
}

impl DbKind for IndustrialOrg {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !vd.field_item("name", Item::Localization) {
            data.verify_exists(Item::Localization, key);
        }
        vd.field_item("icon", Item::Sprite);
        vd.field_item("background", Item::Sprite);

        let has_include = vd.field_item("include", Item::IndustrialOrg);
        if !has_include {
            vd.req_field("allowed");
        }
        vd.field_trigger("allowed", Scopes::Country, Tooltipped::No);

        vd.field_trigger("visible", Scopes::IndustrialOrg, Tooltipped::No);
        vd.field_trigger("available", Scopes::IndustrialOrg, Tooltipped::Yes);

        vd.field_validated_list("equipment_type", |value, data| {
            if !data.item_exists(Item::EquipmentBonusType, value.as_str())
                && !data.item_exists(Item::EquipmentGroup, value.as_str())
            {
                let msg = format!("{value} not found as equipment bonus type or equipment group");
                err(ErrorKey::MissingItem).msg(msg).loc(value).push();
            }
        });

        vd.field_validated_list("research_categories", |value, data| {
            if !data.item_exists(Item::Technology, value.as_str())
                && !data.item_exists(Item::TechnologyCategory, value.as_str())
            {
                let msg = format!("{value} not found as technology or technology category");
                err(ErrorKey::MissingItem).msg(msg).loc(value).push();
            }
        });

        for field in &[
            "on_design_team_assigned_to_tech",
            "on_design_team_assigned_to_variant",
            "on_industrial_manufacturer_assigned",
            "on_tech_research_cancelled",
            "on_tech_research_completed",
            "on_industrial_manufacturer_unassigned",
        ] {
            let sc_builder: &Builder = &|key| {
                let mut sc = ScopeContext::new(Scopes::IndustrialOrg, key);
                sc.push_as_from(Scopes::Country, key);
                sc
            };
            vd.field_effect(field, sc_builder, Tooltipped::No);
        }

        vd.field_numeric("research_bonus");
        vd.field_integer("task_capacity");

        let mut sc = ScopeContext::new(Scopes::IndustrialOrg, key);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);

        vd.multi_field_validated_block("tree_header_text", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("text", Item::Localization);
            vd.field_numeric("x");
        });

        vd.field_validated_block("initial_trait", |block, data| {
            validate_mio_trait(key, block, data);
        });

        if has_include {
            vd.field_list_choice(
                "delete_included_values",
                &[
                    "name",
                    "icon",
                    "background",
                    "allowed",
                    "visible",
                    "available",
                    "equipment_type",
                    "research_categories",
                    "research_bonus",
                    "task_capacity",
                    "ai_will_do",
                    "on_design_team_assigned_to_tech",
                    "on_design_team_assigned_to_variant",
                    "on_industrial_manufacturer_assigned",
                    "on_tech_research_cancelled",
                    "on_tech_research_completed",
                    "on_industrial_manufacturer_unassigned",
                ],
            );
            vd.multi_field_validated_block("add_trait", |block, data| {
                validate_mio_trait(key, block, data);
            });
            vd.multi_field_validated_block("override_trait", |block, data| {
                validate_mio_trait(key, block, data);
            });
            // TODO: check that the trait is from the included mio
            vd.field_list_items("remove_trait", Item::IndustrialOrgTrait);
            vd.ban_field("trait", || "mios that do not use `include =`");
        } else {
            vd.multi_field_validated_block("trait", |block, data| {
                validate_mio_trait(key, block, data);
            });
            vd.ban_field("delete_included_values", || "mios that use `include =`");
            vd.ban_field("add_trait", || "mios that use `include =`");
            vd.ban_field("override_trait", || "mios that use `include =`");
            vd.ban_field("remove_trait", || "mios that use `include =`");
        }
    }
}

impl DbKind for IndustrialOrgPolicy {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !vd.field_item("name", Item::Localization) {
            data.verify_exists(Item::Localization, key);
        }
        vd.field_item("icon", Item::Sprite);

        vd.field_integer("cost");
        vd.field_integer("cooldown");

        vd.field_trigger("allowed", Scopes::IndustrialOrg, Tooltipped::No);
        vd.field_trigger("visible", Scopes::IndustrialOrg, Tooltipped::No);
        vd.field_trigger("available", Scopes::IndustrialOrg, Tooltipped::Yes);

        vd.field_validated_block("equipment_bonus", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("same_as_mio", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
                    vd.numeric();
                });
            });

            vd.unknown_block_fields(|key, block| {
                if !data.item_exists(Item::EquipmentGroup, key.as_str())
                    && !data.item_exists(Item::EquipmentCategory, key.as_str())
                {
                    let mut vd = Validator::new(block, data);
                    vd.validate_item_key_values(Item::EquipmentBonusType, |_, mut vd| {
                        vd.numeric();
                    });
                }
            });
        });

        vd.field_validated_block("production_bonus", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("same_as_mio", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validate_item_key_values(Item::ProductionStat, |_, mut vd| {
                    vd.numeric();
                });
            });
            vd.unknown_block_fields(|key, block| {
                if !data.item_exists(Item::EquipmentGroup, key.as_str())
                    && !data.item_exists(Item::EquipmentCategory, key.as_str())
                {
                    let mut vd = Validator::new(block, data);
                    vd.validate_item_key_values(Item::ProductionStat, |_, mut vd| {
                        vd.numeric();
                    });
                }
            });
        });

        vd.field_validated_block("organization_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::WarProduction, vd);
        });
        vd.field_effect("on_add", Scopes::IndustrialOrg, Tooltipped::Yes);
        vd.field_effect("on_remove", Scopes::IndustrialOrg, Tooltipped::Yes);

        let mut sc = ScopeContext::new(Scopes::IndustrialOrg, key);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
    }
}

impl DbKind for IndustrialOrgBonusWeight {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !key.is("default") {
            data.verify_exists(Item::EquipmentBonusType, key);
        }

        vd.unknown_value_fields(|key, value| {
            if !data.item_exists(Item::EquipmentStat, key.as_str())
                && !data.item_exists(Item::ProductionStat, key.as_str())
                && lookup_modif(key, data, None).is_none()
            {
                let msg = format!(
                    "{key} is not an equipment stat, production stat, or organization modifier"
                );
                warn(ErrorKey::UnknownField).msg(msg).loc(key).push();
            }

            value.expect_number();
        });
    }
}

fn validate_mio_trait(mio: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("token");
    vd.field_identifier("token", "token");

    if !vd.field_item("name", Item::Localization) {
        if let Some(token) = block.get_field_value("token") {
            let loca = format!("{mio}_{token}");
            data.verify_exists_implied(Item::Localization, &loca, token);
        }
    }

    vd.field_item("icon", Item::Sprite);
    vd.field_bool("special_trait_background");

    vd.multi_field_validated_block("parent", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("traits");
        vd.field_list_items("traits", Item::IndustrialOrgTrait);
        vd.field_integer("num_parents_needed");
    });
    vd.field_list_items("any_parent", Item::IndustrialOrgTrait);
    vd.field_list_items("all_parents", Item::IndustrialOrgTrait);
    vd.field_list_items("mutually_exclusive", Item::IndustrialOrgTrait);

    vd.field_trigger("visible", Scopes::IndustrialOrg, Tooltipped::No);
    vd.field_trigger("available", Scopes::IndustrialOrg, Tooltipped::Yes);
    vd.field_effect("on_complete", Scopes::IndustrialOrg, Tooltipped::Yes);

    vd.field_validated_list("limit_to_equipment_type", |value, data| {
        if !data.item_exists(Item::Equipment, value.as_str())
            && !data.item_exists(Item::EquipmentGroup, value.as_str())
        {
            let msg = format!("{value} not found as equipment or equipment group");
            err(ErrorKey::MissingItem).msg(msg).loc(value).push();
        }
    });

    vd.field_validated_block("equipment_bonus", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
            vd.numeric();
        });
    });

    vd.field_validated_block("production_bonus", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.validate_item_key_values(Item::ProductionStat, |_, mut vd| {
            vd.numeric();
        });
    });

    vd.field_validated_block("organization_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::WarProduction, vd);
    });

    vd.field_validated_block("position", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_integer("x");
        vd.field_integer("y");
    });
    vd.field_item("relative_position_id", Item::IndustrialOrgTrait);

    let mut sc = ScopeContext::new(Scopes::IndustrialOrg, mio);
    vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
}
