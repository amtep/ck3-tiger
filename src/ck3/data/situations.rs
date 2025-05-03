use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{validate_duration, validate_possibly_named_color};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Situation {}

#[derive(Clone, Debug)]
pub struct SituationCatalyst {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Situation, Situation::add)
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SituationCatalyst, SituationCatalyst::add)
}

impl Situation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Situation, key, block, Box::new(Self {}));
    }
}

impl SituationCatalyst {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SituationCatalyst, key, block, Box::new(Self {}));
    }
}

impl DbKind for Situation {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("sub_regions") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::SituationSubRegion, key.clone());
            }
        }
        if let Some(block) = block.get_field_block("participant_groups") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::SituationParticipantGroup, key.clone());
            }
        }
        if let Some(block) = block.get_field_block("phases") {
            for (key, block) in block.iter_definitions() {
                if let Some(block) = block.get_field_block("parameters") {
                    for (key, _) in block.iter_assignments() {
                        db.add_flag(Item::SituationPhaseParameter, key.clone());
                    }
                }
                if let Some(block) = block.get_field_block("modifier_sets") {
                    for (_, block) in block.iter_definitions() {
                        for (_, block) in block.iter_definitions() {
                            if let Some(block) = block.get_field_block("parameters") {
                                for (key, _) in block.iter_assignments() {
                                    db.add_flag(Item::SituationPhaseParameter, key.clone());
                                }
                            }
                        }
                    }
                }
                db.add_flag(Item::SituationPhase, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("situation_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("situation_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("situation_type_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("situation_type_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("window", &["situation", "the_great_steppe"]);
        if let Some(token) = vd.field_value("gui_window_name") {
            let pathname = format!("gui/{token}.gui");
            data.verify_exists_implied(Item::File, &pathname, token);
        }
        if let Some(token) = vd.field_value("gui_participation_window_name") {
            let pathname = format!("gui/{token}.gui");
            data.verify_exists_implied(Item::File, &pathname, token);
        }
        vd.field_choice("map_mode", &["participant_groups", "sub_regions"]);

        vd.req_field("sub_regions");
        vd.field_validated_block("sub_regions", |block, data| {
            let mut vd = Validator::new(block, data);
            let mut count = 0;
            vd.unknown_block_fields(|sub_region_key, block| {
                count += 1;
                validate_sub_region(sub_region_key, block, data, key);
            });
            if count == 0 {
                let msg = "situation needs at least one sub-region";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        });

        vd.req_field("participant_groups");
        vd.field_validated_block("participant_groups", |block, data| {
            let mut vd = Validator::new(block, data);
            let mut count = 0;
            vd.unknown_block_fields(|pg_key, block| {
                count += 1;
                validate_participant_group(pg_key, block, data, key);
            });
            if count == 0 {
                let msg = "situation needs at least one participant group";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        });

        vd.req_field("phases");
        vd.field_validated_block("phases", |block, data| {
            let mut vd = Validator::new(block, data);
            let mut count = 0;
            vd.unknown_block_fields(|phase_key, block| {
                count += 1;
                validate_phase(phase_key, block, data, key);
            });
            if count == 0 {
                let msg = "situation needs at least one phase";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        });

        vd.field_effect_rooted("on_start", Scopes::Situation, Tooltipped::No);
        vd.field_effect_rooted("on_end", Scopes::Situation, Tooltipped::No);
        vd.field_effect_rooted("on_monthly", Scopes::Situation, Tooltipped::No);
        vd.field_effect_rooted("on_yearly", Scopes::Situation, Tooltipped::No);
        vd.field_effect_rooted("on_join", Scopes::Situation, Tooltipped::Yes);
        vd.field_effect_rooted("on_leave", Scopes::Situation, Tooltipped::Yes);

        vd.field_bool("is_unique");
        vd.field_bool("migration");
        // TODO: check that the start phase is part of this situation's phases
        vd.field_item("start_phase", Item::SituationPhase);
    }
}

impl DbKind for SituationCatalyst {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        // vd is created in order to warn about unknown fields when it is dropped.
        let _vd = Validator::new(block, data);
    }
}

fn validate_participant_group(key: &Token, block: &Block, data: &Everything, situation: &Token) {
    fn sc_builder(key: &Token) -> ScopeContext {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("situation", Scopes::Situation, key);
        sc.define_name("situation_sub_region", Scopes::SituationSubRegion, key);
        sc
    }
    fn sc_with_group(key: &Token) -> ScopeContext {
        let mut sc = sc_builder(key);
        sc.define_name("situation_participant_group", Scopes::SituationParticipantGroup, key);
        sc
    }

    let mut vd = Validator::new(block, data);

    let loca = format!("{situation}_participant_group_{key}");
    data.verify_exists_implied(Item::Localization, &loca, key);
    let loca = format!("{situation}_participant_group_{key}_desc");
    data.verify_exists_implied(Item::Localization, &loca, key);

    vd.field_item("icon", Item::File);
    vd.field_bool("auto_add_rulers");
    vd.field_validated("map_color", validate_possibly_named_color);
    vd.field_bool("require_capital_in_sub_region");
    vd.field_bool("require_domain_in_sub_region");
    vd.field_bool("require_realm_in_sub_region");

    vd.field_trigger_builder("is_character_valid", sc_builder, Tooltipped::Yes);
    vd.field_effect_builder("on_join", sc_with_group, Tooltipped::Yes);
    vd.field_effect_builder("on_leave", sc_with_group, Tooltipped::Yes);
}

fn validate_phase(key: &Token, block: &Block, data: &Everything, situation: &Token) {
    fn sc_builder(key: &Token) -> ScopeContext {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("situation", Scopes::Situation, key);
        sc.define_name("situation_sub_region", Scopes::SituationSubRegion, key);
        sc
    }
    fn sc_builder2(key: &Token) -> ScopeContext {
        let mut sc = ScopeContext::new(Scopes::Situation, key);
        sc.define_name("situation_sub_region", Scopes::SituationSubRegion, key);
        sc
    }

    let mut vd = Validator::new(block, data);

    let loca = format!("{situation}_{key}_situation_phase");
    data.verify_exists_implied(Item::Localization, &loca, key);
    // TODO: {key} and {key}_desc also seem to exist.

    vd.field_validated_block("parameters", validate_parameters);

    vd.field_effect_builder("on_start", sc_builder, Tooltipped::No);
    vd.field_effect_builder("on_end", sc_builder, Tooltipped::No);
    vd.field_item("illustration", Item::File);
    vd.field_item("icon", Item::File);
    vd.field_item("map_province_effect", Item::ProvinceEffect);
    vd.field_numeric_range("map_province_effect_intensity", 0.0..=1.0);
    vd.field_validated_block_sc("max_duration", &mut sc_builder2(key), validate_duration);
    vd.field_choice(
        "max_duration_next_phase",
        &[
            "highest_points",
            "weighted_random_points",
            "random_non_takeover",
            "weighted_non_takeover",
        ],
    );

    vd.field_validated_block("future_phases", |block, data| {
        let mut vd = Validator::new(block, data);

        vd.validate_item_key_blocks(Item::SituationPhase, |_, block, data| {
            let mut vd = Validator::new(block, data);

            vd.field_choice("takeover_type", &["none", "points", "duration"]);
            vd.field_script_value_no_breakdown_builder("takeover_points", sc_builder2);
            vd.field_script_value_no_breakdown_builder("weight", sc_builder2);
            vd.field_validated_block_sc(
                "takeover_duration",
                &mut sc_builder2(key),
                validate_duration,
            );
            vd.field_validated_block("catalysts", |block, data| {
                let mut vd = Validator::new(block, data);

                vd.unknown_fields(|key, bv| {
                    data.verify_exists(Item::SituationCatalyst, key);
                    validate_non_dynamic_script_value(bv, data);
                });
            });
        });
    });

    vd.advice_field("modifier_named_sets", "docs say modifier_named_sets but it's modifier_sets");
    vd.field_validated_block("modifier_sets", |block, data| {
        let mut vd = Validator::new(block, data);

        vd.unknown_block_fields(|key, block| {
            let mut vd = Validator::new(block, data);
            data.verify_exists(Item::Localization, key);

            vd.field_item("icon", Item::File);
            vd.field_validated_block("all", validate_modifier_set);
            // TODO: the participant groups should be from this situation.
            vd.validate_item_key_blocks(Item::SituationParticipantGroup, |_, block, data| {
                validate_modifier_set(block, data);
            });
        });
    });
}

fn validate_sub_region(key: &Token, block: &Block, data: &Everything, situation: &Token) {
    let mut vd = Validator::new(block, data);

    let loca = format!("{situation}_sub_region_{key}");
    data.verify_exists_implied(Item::Localization, &loca, key);

    vd.field_item("illustration", Item::File);
    vd.field_item("icon", Item::File);
    vd.field_validated("map_color", validate_possibly_named_color);
    vd.field_list_items("geographical_regions", Item::Region);
}

fn validate_modifier_set(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_block("county_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::County, vd);
    });
    vd.field_validated_block("character_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.multi_field_validated_block("doctrine_character_modifier", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("name", Item::Localization);
        vd.field_item("doctrine", Item::Doctrine);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.field_validated_block("parameters", validate_parameters);
}

fn validate_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, value| {
        let loca = format!("situation_parameter_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        // TODO: {key}_name also seems to exist.

        if !value.is("yes") {
            let msg = "only `yes` makes sense here";
            warn(ErrorKey::Validation).msg(msg).loc(value).push();
        }
    });
}
