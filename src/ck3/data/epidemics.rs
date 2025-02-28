use crate::Everything;
use crate::block::{BV, Block};
use crate::ck3::tables::misc::OUTBREAK_INTENSITIES;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_duration, validate_possibly_named_color};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct EpidemicType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::EpidemicType, EpidemicType::add)
}

impl EpidemicType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EpidemicType, key, block, Box::new(Self {}));
    }
}

impl DbKind for EpidemicType {
    fn validate(&self, key: &Token, block: &Block, data: &crate::Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("trait");
        vd.field_item("trait", Item::Trait);
        vd.field_validated("color", validate_possibly_named_color);

        if !vd.field_validated_rooted("name", Scopes::Epidemic, validate_desc) {
            let loca = format!("epidemic_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_integer_range("priority", 1..);

        vd.field_validated_block("shader_data", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_precise_numeric_range("strength", 0.0..=1.0);
            vd.field_precise_numeric_range("edge_fade", 0.0..=1.0);
            vd.field_precise_numeric_range("tile_multiplier", 0.0..=1.0);
            vd.field_integer_range("texture_index", 0..);
            vd.field_choice("channel", &["red", "green", "blue", "alpha"]);
        });

        vd.field_validated_block_build_sc(
            "can_infect_character",
            build_character_epidemic_sc,
            |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            },
        );

        vd.field_script_value_no_breakdown_build_sc("character_infection_chance", |key| {
            let mut sc = build_character_epidemic_sc(key);
            // undocumented
            sc.define_name("province", Scopes::Province, key);
            sc
        });

        vd.field_validated_block_build_sc(
            "on_character_infected",
            build_character_epidemic_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );

        vd.field_validated_block_build_sc(
            "on_province_infected",
            build_province_epidemic_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );

        vd.field_validated_block_build_sc(
            "on_province_recovered",
            build_province_epidemic_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );

        vd.field_validated_block_rooted("on_start", Scopes::Epidemic, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::No);
        });

        vd.field_validated_block_build_sc(
            "on_monthly",
            build_character_epidemic_sc,
            |block, data, sc| {
                validate_effect(block, data, sc, Tooltipped::No);
            },
        );

        vd.field_validated_block_rooted("on_end", Scopes::Epidemic, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::No);
        });

        vd.field_validated_block("infection_levels", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                let mut vd = Validator::new(block, data);
                validate_non_dynamic_script_value(&BV::Value(key.clone()), data);

                vd.field_validated_block("province_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Province, vd);
                });

                vd.field_validated_block("county_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::County, vd);
                });

                vd.field_validated_block("realm_modifier", |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
            });
        });

        vd.field_validated_block("outbreak_intensities", |block, data| {
            let mut vd = Validator::new(block, data);
            for &level in OUTBREAK_INTENSITIES {
                vd.req_field(level);
                vd.field_validated_block(level, validate_outbreak_level);
            }
        });

        if !data.item_exists(Item::EpidemicDeathReason, key.as_str()) {
            let msg = format!("no deathreason found for epidemic {key}");
            let info = "this will lead to the game showing 0 deaths from this epidemic";
            warn(ErrorKey::MissingItem).msg(msg).info(info).loc(key).push();
        }
    }
}

fn build_character_epidemic_sc(key: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Character, key);
    sc.define_name("epidemic", Scopes::Epidemic, key);
    sc
}

fn build_province_epidemic_sc(key: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Province, key);
    sc.define_name("epidemic", Scopes::Epidemic, key);
    sc
}

fn validate_outbreak_level(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_bool("global_notification");
    vd.field_script_value_no_breakdown_build_sc("outbreak_chance", |key| {
        let mut sc = ScopeContext::new(Scopes::Province, key);
        sc.define_name("epidemic_type", Scopes::EpidemicType, key);
        sc
    });

    vd.field_script_value_build_sc("spread_chance", build_province_epidemic_sc);
    vd.field_script_value_no_breakdown_build_sc("max_provinces", |key| {
        ScopeContext::new(Scopes::None, key)
    });

    vd.field_validated_block_build_sc(
        "infection_duration",
        build_province_epidemic_sc,
        validate_duration,
    );

    vd.field_validated_block_build_sc(
        "infection_progress_duration",
        build_province_epidemic_sc,
        validate_duration,
    );

    vd.field_validated_block_build_sc(
        "infection_recovery_duration",
        build_province_epidemic_sc,
        validate_duration,
    );
}
