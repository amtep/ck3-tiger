use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GreatWorkEffect {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GreatWorkEffect, GreatWorkEffect::add)
}

impl GreatWorkEffect {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GreatWorkEffect, key.clone(), block.clone(), Box::new(Self {}));
    }
}

impl DbKind for GreatWorkEffect {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.field_value("icon");
        vd.field_list("flags");

        vd.field_validated_block("allow", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("great_work_tier_effect_modifiers", |block, data| {
            let mut vd = Validator::new(block, data);
            for tier in 1..5 {
                let tier_str = format!("{}_tier_{}", key.as_str(), tier);
                vd.field_validated_block(&tier_str, |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_item("great_work_effect_tier", Item::GreatWorkEffectTier);
                    vd.field_item("tier_modifier_tooltip_override", Item::Localization);
                    vd.field_validated_block("country_modifier", |block, data| {
                        let vd = Validator::new(block, data);
                        validate_modifs(block, data, ModifKinds::Country, vd);
                    });
                    vd.field_validated_block("state_modifier", |block, data| {
                        let vd = Validator::new(block, data);
                        validate_modifs(block, data, ModifKinds::Province | ModifKinds::State, vd);
                    });
                });
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct GreatWorkEffectTier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GreatWorkEffectTier, GreatWorkEffectTier::add)
}

impl GreatWorkEffectTier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GreatWorkEffectTier, key, block, Box::new(Self {}));
    }
}

impl DbKind for GreatWorkEffectTier {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("icon");
        vd.field_numeric("level");
        vd.field_numeric("tier_threshold");
        vd.field_numeric("tier_cost_factor");

        vd.field_validated_block("great_work_effect_upgrade_costs", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validated_blocks(|block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_numeric("level");
                vd.field_validated_block("price", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_numeric("political_influence");
                    vd.field_numeric("stability");
                    vd.field_numeric("gold");
                });
            });
        });

        vd.field_validated_block("great_work_effect_tier_addition_cost", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("level");
            vd.field_validated_block("price", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_numeric("political_influence");
                vd.field_numeric("stability");
                vd.field_numeric("gold");
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct GreatWorkCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GreatWorkCategory, GreatWorkCategory::add)
}

impl GreatWorkCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GreatWorkCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for GreatWorkCategory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_numeric("great_work_prestige");

        vd.field_validated_block("price", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("gold");
        });
        vd.field_validated_block("material_price", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("gold");
        });

        vd.field_validated_block("great_work_material", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::GreatWorkMaterial, key);
                value.expect_number();
            });
        });

        vd.field_validated_block("great_work_surface_materials", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::GreatWorkMaterial, key);
                value.expect_number();
            });
        });

        let choices = &["base", "middle", "top", "pillar", "roof"];
        vd.field_validated_block("great_work_category_slots", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validated_blocks(|block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_choice("key", choices);
                vd.field_numeric("index");
                vd.field_item("localization_key", Item::Localization);
                vd.field_validated_block("attachments", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.validated_blocks(|block, data| {
                        let mut vd = Validator::new(block, data);
                        vd.field_choice("key", choices);
                        vd.field_choice(
                            "function",
                            &[
                                "great_work_function_attach_one_and_scale",
                                "great_work_function_attach_multiple",
                            ],
                        );
                    });
                });
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct GreatWorkMaterial {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GreatWorkMaterial, GreatWorkMaterial::add)
}

impl GreatWorkMaterial {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GreatWorkMaterial, key, block, Box::new(Self {}));
    }
}

impl DbKind for GreatWorkMaterial {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("icon");
        vd.field_numeric("great_work_prestige");
        vd.field_item("great_work_trade_good", Item::TradeGood);
        vd.field_list_numeric_exactly("great_work_uv_offset", 2);
    }
}

#[derive(Clone, Debug)]
pub struct GreatWorkModule {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GreatWorkModule, GreatWorkModule::add)
}

impl GreatWorkModule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GreatWorkModule, key, block, Box::new(Self {}));
    }
}

impl DbKind for GreatWorkModule {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("icon");
        vd.field_choice(
            "great_work_category",
            &["gw_category_ancient", "pyramid", "tower", "building"],
        );
        vd.field_numeric("great_work_category_slot");

        vd.field_validated_list("entity_name", |token, data| {
            data.verify_exists(Item::Entity, token);
        });

        vd.field_choice(
            "great_work_building_type",
            &[
                "great_work_building_type_rectangular",
                "great_work_building_type_square",
                "great_work_building_type_round",
            ],
        );

        vd.field_validated_block("great_work_attachment_data", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_numeric("locator");
            vd.field_numeric("min_distance");
            vd.field_value("locator_min");
            vd.field_value("locator_max");
        });

        vd.field_validated_block("great_work_attachments", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validated_blocks(|block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_choice(
                    "function",
                    &[
                        "great_work_function_attach_one_and_scale",
                        "great_work_function_attach_multiple",
                    ],
                );
                vd.field_validated_block("data", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.validated_blocks(|block, data| {
                        let mut vd = Validator::new(block, data);
                        vd.field_value("locator");
                        vd.field_value("locator_min");
                        vd.field_value("locator_max");
                        vd.field_value("attach_multiple_type");
                        vd.field_choice(
                            "attach_multiple_type",
                            &["attach_multiple_round", "attach_multiple_line"],
                        );
                        vd.field_bool("include_min");
                        vd.field_bool("include_max");
                    });
                });
            });
        });
    }
}

#[derive(Clone, Debug)]
pub struct GreatWorkTemplate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::GreatWorkTemplate, GreatWorkTemplate::add)
}

impl GreatWorkTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GreatWorkTemplate, key, block, Box::new(Self {}));
    }
}

impl DbKind for GreatWorkTemplate {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice(
            "great_work_category",
            &["gw_category_ancient", "pyramid", "tower", "building"],
        );
        vd.field_value("icon");
        vd.field_bool("can_build");
        vd.field_item("localization_key", Item::Localization);

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
}
