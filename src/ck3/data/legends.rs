use crate::block::{Block, BV};
use crate::ck3::tables::misc::LEGEND_QUALITY;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::game::GameFlags;
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::{validate_non_dynamic_script_value, validate_script_value_no_breakdown};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target, validate_trigger};
use crate::validate::{validate_duration, validate_possibly_named_color};
use crate::validator::Validator;
use crate::Everything;

#[derive(Clone, Debug)]
pub struct LegendType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::LegendType, LegendType::add)
}

impl LegendType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LegendType, key, block, Box::new(Self {}));
    }
}

impl DbKind for LegendType {
    fn validate(&self, key: &Token, block: &Block, data: &crate::Everything) {
        let loca = format!("legend_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("legend_{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("legend_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_validated_block_build_sc(
            "on_province_spread",
            build_province_legend_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_block_build_sc(
            "on_province_recovered",
            build_province_legend_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_block_rooted("on_start", Scopes::Legend, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_block_rooted("on_end", Scopes::Legend, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::No);
        });
        // ScopeContext undocumented
        vd.field_validated_block_build_sc(
            "on_yearly",
            build_character_legend_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_block_build_sc(
            "on_legend_start_promote",
            build_character_legend_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No); // TODO: verify tooltip
            },
        );
        vd.field_validated_block_build_sc(
            "on_legend_stop_promote",
            build_character_legend_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_block_build_sc(
            "is_valid_protagonist",
            build_character_character_sc,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::Yes); // TODO: verify tooltip
            },
        );
        vd.field_validated_build_sc(
            "ai_protagonist_weight",
            build_character_character_sc,
            validate_script_value_no_breakdown,
        );
        vd.field_validated_block("quality", |block, data| {
            let mut vd = Validator::new(block, data);
            for &quality in LEGEND_QUALITY {
                vd.req_field(quality);
                vd.field_validated_block(quality, validate_legend_quality);
            }
        });
    }

    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _from: &Token,
        _from_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        if let Some(block) = block.get_field_block("quality") {
            for (_, block) in block.iter_definitions() {
                if let Some(block) = block.get_field_block("impact") {
                    if let Some(block) = block.get_field_block("on_complete") {
                        validate_effect(block, data, sc, Tooltipped::Yes); // TODO verify tooltip
                    }
                }
            }
        }
    }
}

fn build_province_legend_sc(key: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Province, key);
    sc.define_name("legend", Scopes::Legend, key);
    sc
}

fn build_character_legend_sc(key: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("legend", Scopes::Legend, key);
    sc
}

fn build_character_character_sc(key: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("creator", Scopes::Character, key);
    sc
}

fn validate_legend_quality(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_build_sc(
        "spread_chance",
        build_province_legend_sc,
        validate_script_value_no_breakdown,
    );
    vd.field_validated("max_provinces", validate_non_dynamic_script_value);
    vd.field_validated_block_rooted("owner_cost", Scopes::Character, validate_cost);
    vd.field_validated_block_build_sc("promoter_cost", build_character_legend_sc, validate_cost);
    vd.field_validated_block_rooted("creation_cost", Scopes::Character, validate_cost);
    vd.field_validated_block_rooted("upgrade_cost", Scopes::Character, validate_cost);
    vd.field_validated_block_rooted("removal_duration", Scopes::None, validate_duration);
    vd.field_validated_block("impact", |block, data| {
        let mut vd = Validator::new(block, data);
        validate_impact_modifiers(&mut vd);
        // proper validation in `validate_call`
        vd.field_block("on_complete");
    });
    vd.field_validated_block("ai_chance", validate_ai_chance);
}

fn validate_impact_modifiers(vd: &mut Validator) {
    vd.field_validated_block("province_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Province, vd);
    });
    vd.field_validated_block("county_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::County, vd);
    });
    vd.field_validated_block("owner_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.field_validated_block("promoter_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
}

fn validate_ai_chance(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_rooted("create", Scopes::Character, validate_script_value_no_breakdown);
    vd.field_validated_build_sc(
        "promote",
        build_character_legend_sc,
        validate_script_value_no_breakdown,
    );
    vd.field_validated_build_sc(
        "take_unowned",
        build_character_legend_sc,
        validate_script_value_no_breakdown,
    );
    vd.field_validated_build_sc(
        "upgrade",
        build_character_legend_sc,
        validate_script_value_no_breakdown,
    );
    vd.field_validated_build_sc(
        "complete",
        |key| {
            let mut sc = build_character_legend_sc(key);
            sc.define_name("can_afford_current_level", Scopes::Bool, key);
            sc
        },
        validate_script_value_no_breakdown,
    );
}

#[derive(Clone, Debug)]
pub struct LegendSeed {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::LegendSeed, LegendSeed::add)
}

impl LegendSeed {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LegendSeed, key, block, Box::new(Self {}));
    }
}

impl DbKind for LegendSeed {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let loca = format!("legend_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("legend_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        vd.req_field("quality");
        vd.req_field("type");
        vd.req_field("chronicle");

        vd.field_choice("quality", LEGEND_QUALITY);
        vd.field_item("type", Item::LegendType);
        vd.field_validated_block_rooted("is_shown", Scopes::Character, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_block_rooted("is_valid", Scopes::Character, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::Yes); // TODO verify tooltip
        });

        if let Some(chronicle_token) = vd.field_value("chronicle").cloned() {
            data.verify_exists(Item::LegendChronicle, &chronicle_token);

            if let Some((_, _, chronicle)) =
                data.get_item::<LegendChronicle>(Item::LegendChronicle, chronicle_token.as_str())
            {
                vd.field_validated_key_block("chronicle_properties", |key, block, data| {
                    let mut found_properties = TigerHashSet::default();
                    let mut sc = ScopeContext::new(Scopes::Character, key);
                    let mut vd = Validator::new(block, data);
                    vd.unknown_fields(|key, bv| {
                        if let Some(scopes) = chronicle.properties.get(key).copied() {
                            found_properties.insert(key.clone());

                            match bv {
                                BV::Value(value) => {
                                    validate_target(value, data, &mut sc, scopes);
                                }
                                BV::Block(block) => {
                                    let mut vd = Validator::new(block, data);
                                    vd.field_validated_value("target", |_, mut vd| {
                                        vd.target(&mut sc, scopes);
                                    });
                                    vd.field_validated_block("is_valid", |block, data| {
                                        validate_trigger(block, data, &mut sc, Tooltipped::No);
                                    });
                                }
                            }
                        } else {
                            let msg =
                                format!("property {key} not found in {chronicle_token} chronicle");
                            err(ErrorKey::Validation).msg(msg).loc(key).push();
                        }
                    });

                    for property in chronicle.properties.keys() {
                        if !found_properties.contains(property) {
                            let msg = format!("property {property} not found");
                            err(ErrorKey::Validation)
                                .msg(msg)
                                .loc(key)
                                .loc_msg(property, "from here")
                                .push();
                        }
                    }
                });
                vd.field_validated_block("chronicle_chapters", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.unknown_value_fields(|key, value| {
                        if !chronicle.chapters.contains(key) {
                            let msg =
                                format!("chapter {key} not found in {chronicle_token} chronicle");
                            err(ErrorKey::Validation).msg(msg).loc(key).push();
                        }
                        data.verify_exists(Item::Localization, value);
                    });
                });

                // Validate type's `on_complete` block based on the chronicle's properties
                if let Some(value) = vd.field_value("type") {
                    data.validate_call(
                        Item::LegendType,
                        key,
                        block,
                        &mut build_impact_on_complete_sc(chronicle, value),
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct LegendChronicle {
    pub properties: TigerHashMap<Token, Scopes>,
    chapters: TigerHashSet<Token>,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::LegendChronicle, LegendChronicle::add)
}

impl LegendChronicle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        let mut properties = TigerHashMap::default();
        let mut chapters = TigerHashSet::default();

        if let Some(block) = block.get_field_block("properties") {
            for (key, value) in block.iter_assignments() {
                if let Some(scopes) = Scopes::from_snake_case(value.as_str()) {
                    properties.insert(key.clone(), scopes);
                }
            }
        }

        if let Some(block) = block.get_field_block("chapters") {
            for (key, _) in block.iter_assignments() {
                chapters.insert(key.clone());
            }
        }
        db.add(Item::LegendChronicle, key, block, Box::new(Self { properties, chapters }));
    }
}

impl DbKind for LegendChronicle {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("properties") {
            for (key, _) in block.iter_assignments() {
                db.add_flag(Item::LegendProperty, key.clone());
            }
        }

        if let Some(block) = block.get_field_block("chapters") {
            for (key, _) in block.iter_assignments() {
                db.add_flag(Item::LegendChapter, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if !vd.field_validated_build_sc(
            "name",
            |key| build_impact_on_complete_sc(self, key),
            validate_desc,
        ) {
            let loca = format!("legend_chronicle_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if !vd.field_validated_build_sc(
            "description",
            |key| build_impact_on_complete_sc(self, key),
            validate_desc,
        ) {
            let loca = format!("legend_chronicle_{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        // undocumented
        vd.field_item("portrait_animation", Item::PortraitAnimation);

        vd.field_validated_block("properties", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|_, value| {
                if Scopes::from_snake_case(value.as_str()).is_none() {
                    let msg = "expected a valid scope type";
                    err(ErrorKey::Validation).msg(msg).loc(value).push();
                }
            });
        });
        vd.field_validated_block_build_sc(
            "chapters",
            |key| build_root_properties_sc(Scopes::Legend, self, key),
            |block, data, sc| {
                let mut vd = Validator::new(block, data);
                vd.unknown_value_fields(|_, value| {
                    data.validate_localization_sc(value.as_str(), sc);
                });
            },
        );
        // Assume the same scope context as impact in `LegendType`
        vd.field_validated_block("impact", |block, data| {
            let mut vd = Validator::new(block, data);
            validate_impact_modifiers(&mut vd);
            vd.field_validated_block_build_sc(
                "on_complete",
                |key| build_impact_on_complete_sc(self, key),
                |block, data, sc| {
                    validate_effect(block, data, sc, Tooltipped::Yes); // TODO verify tooltip
                },
            );
        });
    }
}

fn build_impact_on_complete_sc(chronicle: &LegendChronicle, key: &Token) -> ScopeContext {
    let mut sc = build_root_properties_sc(Scopes::Character, chronicle, key);
    sc.define_name("protagonist", Scopes::Character, key);
    sc
}

fn build_root_properties_sc(
    root: Scopes,
    chronicle: &LegendChronicle,
    key: &Token,
) -> ScopeContext {
    let mut sc = ScopeContext::new(root, key);
    for (property, scopes) in &chronicle.properties {
        sc.define_name(property.as_str(), *scopes, key);
    }
    sc
}
