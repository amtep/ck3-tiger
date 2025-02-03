use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::helpers::TigerHashSet;
use crate::item::Item;
use crate::report::{err, warn, ErrorKey, ErrorLoc};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target;
use crate::trigger::validate_trigger;
use crate::validate::{validate_color, validate_optional_duration, validate_possibly_named_color};
use crate::validator::{Validator, ValueValidator};
use crate::vic3::data::buildings::BuildingType;
use crate::vic3::tables::misc::{LOBBY_FORMATION_REASON, STATE_TYPES, STRATA};

pub fn validate_activate_production_method(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("building_type");
    vd.req_field("production_method");
    vd.field_item("building_type", Item::BuildingType);
    // TODO: check that the production method belongs to the building type
    vd.field_item("production_method", Item::ProductionMethod);
}

pub fn validate_add_culture_sol_modifier(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("culture");
    vd.field_target("culture", sc, Scopes::Culture);
    validate_optional_duration(&mut vd, sc);
    vd.field_script_value("multiplier", sc); // seems to be actually an adder
}

pub fn validate_add_religion_sol_modifier(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("religion");
    vd.field_target("religion", sc, Scopes::Religion);
    validate_optional_duration(&mut vd, sc);
    vd.field_script_value("multiplier", sc); // seems to be actually an adder
}

pub fn validate_add_enactment_modifier(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_item("name", Item::Modifier);
    vd.field_script_value("multiplier", sc);
}

pub fn validate_add_modifier(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(value) => {
            data.verify_exists(Item::Modifier, value);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("name");
            vd.field_item("name", Item::Modifier);
            vd.field_script_value("multiplier", sc);
            validate_optional_duration(&mut vd, sc);
            vd.field_bool("is_decaying");
        }
    }
}

pub fn validate_add_journalentry(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_item("type", Item::JournalEntry);
    vd.field_item("objective_subgoal", Item::ObjectiveSubgoal); // undocumented
    vd.field_target("target", sc, Scopes::all());
}

pub fn validate_add_loyalists(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("value");
    vd.field_script_value("value", sc);
    vd.field_item_or_target("interest_group", sc, Item::InterestGroup, Scopes::InterestGroup);
    vd.field_item_or_target("pop_type", sc, Item::PopType, Scopes::PopType);
    vd.field_choice("strata", STRATA);
    vd.field_item_or_target("culture", sc, Item::Culture, Scopes::Culture);
    vd.field_item_or_target("religion", sc, Item::Religion, Scopes::Religion);
}

pub fn validate_add_technology_progress(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("progress");
    vd.field_numeric("progress");
    vd.req_field("technology");
    vd.field_item("technology", Item::Technology);
}

pub fn validate_add_war_goal(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("holder");
    vd.field_item_or_target("holder", sc, Item::Country, Scopes::Country);
    vd.req_field("type");
    vd.field_item("type", Item::Wargoal);
    vd.field_target("state", sc, Scopes::State);
    // TODO: verify this; there's only one example in vanilla
    vd.advice_field("country", "docs say `country` but it's `target_country`");
    vd.field_target("target_country", sc, Scopes::Country);
    vd.field_target("target_state", sc, Scopes::State);
    vd.field_target("region", sc, Scopes::StateRegion);
    vd.field_bool("primary_demand");
}

pub fn validate_remove_war_goal(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("who");
    vd.field_item_or_target("who", sc, Item::Country, Scopes::Country);
    vd.req_field("war_goal");
    vd.field_item("war_goal", Item::Wargoal);
}

pub fn validate_addremove_backers(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    for value in vd.values() {
        if !data.item_exists(Item::Country, value.as_str()) {
            validate_target(value, data, sc, Scopes::Country);
        }
    }
}

pub fn validate_call_election(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("months");
    vd.field_script_value("months", sc);
}

pub fn validate_change_institution_investment_level(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("institution");
    vd.field_item("institution", Item::Institution);
    vd.req_field("investment");
    vd.field_integer("investment");
}

pub fn validate_set_institution_investment_level(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("institution");
    vd.field_item("institution", Item::Institution);
    vd.req_field("level");
    vd.field_integer("level");
}

pub fn validate_diplomatic_pact(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.req_field("type");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.field_item_or_target("country", sc, Item::Country, Scopes::Country);
    vd.field_item_or_target("first_state", sc, Item::StateRegion, Scopes::State);
    vd.field_item_or_target("second_state", sc, Item::StateRegion, Scopes::State);
    vd.field_item("type", Item::DiplomaticAction);
}

pub fn validate_country_value(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.req_field("value");
    vd.field_item_or_target("country", sc, Item::Country, Scopes::Country);
    vd.field_script_value("value", sc);
}

pub fn validate_create_building(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("building");
    vd.field_item("building", Item::BuildingType);
    let building = block.get_field_value("building");
    vd.field_validated_list("activate_production_methods", |token, data| {
        data.verify_exists(Item::ProductionMethod, token);
        if let Some(building) = building {
            if let Some((_, block, building_item)) =
                data.get_item::<BuildingType>(Item::BuildingType, building.as_str())
            {
                building_item.validate_production_method(token, building, block, data);
            }
        }
    });
    vd.field_bool("subsidized");
    vd.field_numeric_range("reserves", 0.0..=1.0);
    vd.field_validated_value("level", |_, mut vd| {
        vd.maybe_is("arable_land");
        vd.integer();
    });
    vd.field_validated_block("add_ownership", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.multi_field_validated_block("country", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("country");
            vd.field_target("country", sc, Scopes::Country);
            vd.req_field("levels");
            vd.field_integer("levels");
        });
        // Docs say "country" for both, but vanilla uses "building".
        vd.multi_field_validated_block("building", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("country");
            vd.field_target("country", sc, Scopes::Country);
            vd.req_field("levels");
            vd.field_integer("levels");
            vd.req_field("type");
            vd.field_item("type", Item::BuildingType);
            vd.req_field("region");
            vd.field_item("region", Item::StateRegion);
        });
        // undocumented
        vd.multi_field_validated_block("company", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("country");
            vd.field_target("country", sc, Scopes::Country);
            vd.req_field("type");
            vd.field_item("type", Item::CompanyType);
            vd.req_field("levels");
            vd.field_integer("levels");
        });
    });
}

pub fn validate_create_character(
    key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_localization("name", sc);
    vd.field_localization("first_name", sc);
    vd.field_localization("last_name", sc);
    if block.has_key("name") {
        vd.ban_field("first_name", || "characters without `name`");
        vd.ban_field("last_name", || "characters without `name`");
    } else if block.has_key("first_name") {
        if !block.has_key("last_name") {
            let msg = "character has `first_name` but no `last_name`";
            warn(ErrorKey::Validation).msg(msg).loc(key).push();
        }
    } else if block.has_key("last_name") {
        let msg = "character has `last_name` but no `first_name`";
        warn(ErrorKey::Validation).msg(msg).loc(key).push();
    }
    vd.field_validated_value("culture", |_, mut vd| {
        vd.maybe_is("primary_culture");
        vd.item_or_target(sc, Item::Culture, Scopes::Culture);
    });
    vd.field_item_or_target("religion", sc, Item::Religion, Scopes::Religion);
    vd.field_validated_value("female", |_, mut vd| {
        vd.maybe_bool();
        vd.target(sc, Scopes::Character);
    });
    vd.field_validated_value("noble", |_, mut vd| {
        vd.maybe_bool();
        vd.target(sc, Scopes::Character);
    });
    vd.field_bool("ruler");
    vd.field_bool("heir");
    vd.field_bool("historical");
    vd.field_validated("age", |bv, data| {
        match bv {
            BV::Value(value) => {
                // age = integer or character scope
                let mut vd = ValueValidator::new(value, data);
                vd.maybe_integer();
                vd.target(sc, Scopes::Character);
            }
            BV::Block(block) => {
                // age = { min max }
                let mut vd = Validator::new(block, data);
                vd.req_tokens_integers_exactly(2);
            }
        }
    });
    vd.field_item_or_target("ideology", sc, Item::Ideology, Scopes::Ideology);
    vd.field_item_or_target("interest_group", sc, Item::InterestGroup, Scopes::InterestGroup);
    vd.field_item("template", Item::CharacterTemplate);
    vd.field_validated_key_block("on_created", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        validate_effect(block, data, &mut sc, Tooltipped::No);
    });
    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::Character, name);
    }
    vd.field_validated_key_block("trait_generation", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        validate_effect(block, data, &mut sc, Tooltipped::No);
    });
    // The item option is undocumented
    vd.field_item_or_target("hq", sc, Item::StrategicRegion, Scopes::Hq | Scopes::StrategicRegion);

    // undocumented fields

    // TODO: not known how age and birth_date interact
    vd.field_date("birth_date");
    vd.field_list_items("traits", Item::CharacterTrait);
    vd.field_item("dna", Item::Dna);
    vd.field_bool("is_general");
    vd.field_bool("is_admiral");
    vd.field_bool("is_agitator");
    vd.field_bool("ig_leader");
    vd.field_item("commander_rank", Item::CommanderRank);
}

pub fn validate_create_country(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("tag", Item::Country);
    vd.field_target_ok_this("origin", sc, Scopes::Country);
    vd.multi_field_target("state", sc, Scopes::State);
    vd.multi_field_target("province", sc, Scopes::Province);
    vd.field_validated_key_block("on_created", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::Country, key);
        validate_effect(block, data, &mut sc, Tooltipped::No);
    });
}

pub fn validate_create_dynamic_country(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_target_ok_this("origin", sc, Scopes::Country);
    if !block.has_key("origin") {
        vd.req_field("country_type");
        vd.req_field("tier");
        vd.req_field("culture");
        vd.req_field("religion");
        vd.req_field("capital");
        vd.req_field("color");
        vd.req_field("primary_unit_color");
        vd.req_field("secondary_unit_color");
        vd.req_field("tertiary_unit_color");
    }
    vd.field_item("country_type", Item::CountryType);
    vd.field_item("tier", Item::CountryTier);
    vd.multi_field_target("culture", sc, Scopes::Culture);
    vd.field_target("religion", sc, Scopes::Religion);
    vd.field_target("capital", sc, Scopes::State);
    vd.field_item("social_hierarchy", Item::SocialHierarchy);
    vd.field_validated_key_block("cede_state_trigger", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::State, key);
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });
    vd.field_validated("color", validate_possibly_named_color);
    vd.field_validated("primary_unit_color", validate_possibly_named_color);
    vd.field_validated("secondary_unit_color", validate_possibly_named_color);
    vd.field_validated("tertiary_unit_color", validate_possibly_named_color);
    vd.field_validated_key_block("on_created", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::Country, key);
        validate_effect(block, data, &mut sc, Tooltipped::No);
    });
}

pub fn validate_create_diplomatic_play(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_localization("name", sc);
    vd.field_integer_range("escalation", 0..=100);
    vd.field_bool("war");
    vd.field_item_or_target_ok_this("initiator", sc, Item::Country, Scopes::Country);
    vd.field_item("type", Item::DiplomaticPlay);
    vd.advice_field(
        "handle_annexation_as_civil_war",
        "docs say `handle_annexation_as_civil_war` but it's `annex_as_civil_war`",
    );
    vd.field_bool("annex_as_civil_war");
    for field in &["add_initiator_backers", "add_target_backers"] {
        vd.field_validated_list(field, |token, data| {
            let mut vd = ValueValidator::new(token, data);
            vd.maybe_item(Item::Country);
            vd.target(sc, Scopes::Country);
        });
    }
    vd.multi_field_validated_block_sc("add_war_goal", sc, validate_war_goal);

    // undocumented

    vd.field_target("target_state", sc, Scopes::State);
    vd.field_target("target_country", sc, Scopes::Country);
    vd.field_target("target_region", sc, Scopes::StrategicRegion);
}

fn validate_war_goal(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);
    vd.set_case_sensitive(false);
    vd.field_item_or_target_ok_this("holder", sc, Item::Country, Scopes::Country);
    vd.field_item("type", Item::Wargoal);
    vd.advice_field("state", "docs say `state` but it's `target_state`");
    vd.field_target("target_state", sc, Scopes::State);
    vd.advice_field("country", "docs say `country` but it's `target_country`");
    vd.field_target("target_country", sc, Scopes::Country);
    vd.advice_field("region", "docs say `region` but it's `target_region`");
    vd.field_target("target_region", sc, Scopes::StrategicRegion);
    vd.field_bool("primary_demand");
}

pub fn validate_create_mass_migration(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("origin");
    vd.field_target("origin", sc, Scopes::Country);
    vd.req_field("culture");
    vd.field_target("culture", sc, Scopes::Culture);
}

pub fn validate_create_military_formation(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_localization("name", sc);
    vd.field_choice("type", &["army", "fleet"]);
    let is_fleet = block.field_value_is("type", "fleet");
    vd.field_target("hq_region", sc, Scopes::StrategicRegion);
    vd.multi_field_validated_block("combat_unit", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_target("type", sc, Scopes::CombatUnitType);
        vd.field_choice("service_type", &["regular", "conscript"]);
        if let Some(token) = vd.field_value("service_type") {
            if is_fleet && token.is("conscript") {
                let msg = "conscript is not applicable to fleets";
                err(ErrorKey::Choice).msg(msg).loc(token).push();
            }
        }
        vd.field_target("state_region", sc, Scopes::StateRegion);
        vd.field_integer("count");
    });
    if is_fleet {
        vd.ban_field("mobilization_options", || "armies");
    }
    vd.field_validated_list("mobilization_options", |token, data| {
        let mut vd = ValueValidator::new(token, data);
        vd.target(sc, Scopes::MobilizationOption);
    });

    // undocumented

    if let Some(name) = vd.field_value("save_scope_as") {
        sc.define_name_token(name.as_str(), Scopes::MilitaryFormation, name);
    }
}

pub fn validate_create_pop(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    // This effect is undocumented

    #[allow(clippy::integer_division)]
    fn sum_fractions_warner<E: ErrorLoc>(sum_fractions: i64, loc: E) {
        if sum_fractions != 100_000 {
            let msg = format!(
                "fractions should add to exactly 1, currently {}.{:05}",
                sum_fractions / 100_000,
                sum_fractions % 100_000,
            );
            warn(ErrorKey::Validation).msg(msg.trim_end_matches('0')).loc(loc).push();
        };
    }

    vd.field_item("pop_type", Item::PopType);
    vd.field_integer("size");

    let mut available_cultures = TigerHashSet::default();

    if let Some(token) = vd.field_value("culture") {
        available_cultures.insert(token.clone());

        vd.ban_field("cultures", || "pops with several cultures");
        vd.field_item("culture", Item::Culture);
    } else {
        vd.field_validated_block("cultures", |block, data| {
            let mut vd = Validator::new(block, data);
            let mut sum_fractions = 0_i64;

            vd.validate_item_key_values(Item::Culture, |key, mut vd| {
                available_cultures.insert(key.clone());
                vd.numeric_range(0.0..=1.0);

                sum_fractions += vd.value().get_fixed_number().unwrap_or(0);
            });

            sum_fractions_warner(sum_fractions, block);
        });
    }

    if block.has_key("religion") {
        vd.ban_field("split_religion", || "pops without a `religion` field");
        vd.field_item("religion", Item::Religion);
    } else if block.has_key("split_religion") {
        let mut used_cultures = TigerHashSet::default();

        vd.multi_field_validated_block("split_religion", |block, data| {
            let mut vd = Validator::new(block, data);
            let mut only_one_culture = false;

            vd.validate_item_key_blocks(Item::Culture, |key, block, data| {
                if only_one_culture {
                    let msg = "split_religion should contain only one culture block";
                    err(ErrorKey::DuplicateItem).msg(msg).loc(key).push();
                }
                only_one_culture = true;

                if !available_cultures.contains(key.as_str()) {
                    let msg = "culture being split does not appear in pop";
                    err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
                }

                match used_cultures.get(key) {
                    Some(duplicate) => {
                        let msg =
                            format!("trying to split religion of culture {key} multiple times");
                        let msg_other = "first split here";
                        err(ErrorKey::DuplicateField)
                            .msg(msg)
                            .loc(key)
                            .loc_msg(duplicate, msg_other)
                            .push();
                    }
                    None => {
                        used_cultures.insert(key.clone());
                    }
                }

                let mut vd = Validator::new(block, data);
                let mut sum_fractions = 0_i64;

                vd.validate_item_key_values(Item::Religion, |_, mut vd| {
                    vd.numeric_range(0.0..=1.0);

                    sum_fractions += vd.value().get_fixed_number().unwrap_or(0);
                });

                sum_fractions_warner(sum_fractions, block);
            });

            if !only_one_culture {
                let msg = "split_religion must contain one culture block";
                err(ErrorKey::DuplicateItem).msg(msg).loc(block).push();
            }
        });
    }
}

pub fn validate_create_state(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    // This effect is undocumented

    vd.field_target("country", sc, Scopes::Country);
    vd.field_list_items("owned_provinces", Item::Province);
    vd.field_choice("state_type", STATE_TYPES);
}

pub fn validate_create_trade_route(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_item("goods", Item::Goods);
    vd.field_integer("level");

    vd.advice_field("import", "docs say `import = yes` but it's `direction = import`");
    vd.field_choice("direction", &["export", "import"]);
    vd.field_target("origin", sc, Scopes::StateRegion);
    // docs say state_region but it's market
    vd.field_target("target", sc, Scopes::Market);
}

pub fn validate_form_government(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_script_value("value", sc);
    vd.multi_field_item("interest_group_type", Item::InterestGroup);
}

pub fn validate_set_secret_goal(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.req_field("secret_goal");
    vd.field_item_or_target("country", sc, Item::Country, Scopes::Country);
    vd.field_item("secret_goal", Item::SecretGoal);
}

pub fn validate_post_notification(
    _key: &Token,
    mut vd: ValueValidator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.item(Item::Message);
    vd.implied_localization_sc("notification_", "_name", sc);
    vd.implied_localization_sc("notification_", "_desc", sc);
    vd.implied_localization_sc("notification_", "_tooltip", sc);
}

pub fn validate_progress(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("value");
    vd.req_field("name");
    vd.field_script_value("value", sc);
    vd.field_item("name", Item::ScriptedProgressBar);
}

pub fn validate_join_war(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("target");
    vd.req_field("side");
    vd.field_target("target", sc, Scopes::Country);
    vd.field_target("side", sc, Scopes::Country);
}

pub fn validate_create_truce(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("country");
    vd.req_field("months");
    vd.advice_field("tcountry", "documentation says tcountry but it's just country");
    vd.field_target("country", sc, Scopes::Country);
    // TODO: docs say integer, but check if script value is allowed
    vd.field_integer("months");
}

pub fn validate_create_power_bloc(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("map_color");
    vd.req_field("identity");
    // TODO: see if a full desc is allowed here. Docs just say loc key.
    vd.field_validated_sc("name", sc, validate_desc);
    // TODO: check if named colors are allowed
    vd.field_validated_block("map_color", validate_color);
    vd.field_item("identity", Item::PowerBlocIdentity);
    vd.multi_field_item("principle", Item::Principle);
    vd.multi_field_target("member", sc, Scopes::Country);

    // undocumented

    vd.field_date("founding_date");
}

pub fn validate_create_lobby(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("target");
    vd.field_item("type", Item::PoliticalLobby);
    vd.field_target("target", sc, Scopes::Country);
    vd.multi_field_target("add_interest_group", sc, Scopes::InterestGroup);
    // undocumented
    vd.field_choice("lobby_formation_reason", LOBBY_FORMATION_REASON);
}

pub fn validate_create_movement(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_item("type", Item::PoliticalMovement);
    vd.advice_field("movement_type", "docs say movement_type but it's just type");
    vd.field_target("religion", sc, Scopes::Religion);
    vd.field_target("culture", sc, Scopes::Culture);
}

pub fn validate_create_catalyst(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("target");
    vd.field_item("type", Item::DiplomaticCatalyst);
    vd.field_target("target", sc, Scopes::Country);
}

pub fn validate_change_appeasement(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("amount");
    vd.req_field("factor");
    vd.field_item("factor", Item::PoliticalLobbyAppeasement);
    vd.field_script_value("amount", sc);
}

/// Validate `set_pop_wealth` and `add_pop_wealth`
pub fn validate_pop_wealth(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("wealth_distribution");
    vd.field_script_value("wealth_distribution", sc);
    vd.field_bool("update_loyalties");
}

pub fn validate_kill_character(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(value) => {
            // kill_character = yes
            let mut vd = ValueValidator::new(value, data);
            vd.bool();
        }
        BV::Block(block) => {
            // kill_character = { hidden = yes value = yes }
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.field_bool("value");
            vd.field_bool("hidden");
        }
    }
}

// Validated `kill_population`, `kill_population_in_state`, `kill_population_percent`, and
// `kill_population_percent_in_state`.
pub fn validate_kill_population(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    let percent = key.is("kill_population_percent") || key.is("kill_population_percent_in_state");
    if percent {
        vd.field_numeric_range("percent", 0.0..=1.0);
    } else {
        vd.field_integer("value");
    }
    vd.field_target("culture", sc, Scopes::Culture);
    vd.field_target("religion", sc, Scopes::Religion);
    vd.field_target("interest_group", sc, Scopes::InterestGroup);
    vd.field_item("pop_type", Item::PopType);
    vd.field_choice("strata", STRATA);
}

pub fn validate_pop_literacy(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.field_script_value("literacy_rate", sc);
}

pub fn validate_move_partial_pop(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("state");
    vd.field_target("state", sc, Scopes::State);
    // TODO: verify if these can be script values. Doc example just gives numbers.
    vd.field_script_value("population", sc);
    vd.field_script_value("population_ratio", sc);
}

pub fn validate_set_hub_name(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.req_field("name");
    vd.field_choice("type", &["city", "farm", "mine", "port", "wood"]);
    vd.field_item("name", Item::Localization);
}

pub fn validate_sort(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("order");
    if let Some(name) = vd.field_value("name") {
        // The "order" is evaluated in the scope of the variable list item, which is not known.
        sc.open_scope(Scopes::all(), name.clone());
        vd.field_script_value("order", sc);
        sc.close();
    }
}
