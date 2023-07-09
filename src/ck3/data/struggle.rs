use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{old_warn, ErrorKey};
use crate::scopes::Scopes;
use crate::scriptvalue::validate_scriptvalue;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug)]
pub struct Struggle {}

impl Struggle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(block) = block.get_field_block("phase_list") {
            for (key, block) in block.iter_definitions() {
                db.add_flag(Item::StrugglePhase, key.clone());
                for field in &["war_effects", "culture_effects", "faith_effects", "other_effects"] {
                    if let Some(block) = block.get_field_block(field) {
                        for field in &[
                            "common_parameters",
                            "involved_parameters",
                            "interloper_parameters",
                            "uninvolved_parameters",
                        ] {
                            if let Some(block) = block.get_field_block(field) {
                                for (key, value) in block.iter_assignments() {
                                    if value.is("yes") {
                                        db.add_flag(Item::StrugglePhaseParameter, key.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        db.add(Item::Struggle, key, block, Box::new(Self {}));
    }
}

impl DbKind for Struggle {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Struggle, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_list_items("cultures", Item::Culture);
        vd.field_list_items("faiths", Item::Faith);
        vd.field_list_items("regions", Item::Region);

        vd.field_numeric_range("involvement_prerequisite_percentage", 0.0, 1.0);

        vd.req_field("phase_list");
        vd.field_validated_block("phase_list", |block, data| {
            let mut has_one = false;
            let mut has_ending = false;
            let mut vd = Validator::new(block, data);
            for (key, block) in vd.unknown_block_fields() {
                data.verify_exists(Item::Localization, key);
                let loca = format!("{key}_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let pathname = format!("gfx/interface/icons/struggle_types/{key}.dds");
                data.verify_exists_implied(Item::File, &pathname, key);
                has_one = true;
                validate_phase(block, data);
                if let Some(vec) = block.get_field_list("ending_decisions") {
                    has_ending |= !vec.is_empty();
                }
            }
            if !has_one {
                old_warn(block, ErrorKey::Validation, "must have at least one phase");
            }
            if !has_ending {
                let msg = "must have at least one phase with ending_decisions";
                old_warn(block, ErrorKey::Validation, msg);
            }
        });

        vd.req_field("start_phase");
        vd.field_item("start_phase", Item::StrugglePhase);

        vd.field_validated_block("on_start", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_end", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No); // TODO: check tooltipped
        });
        vd.field_validated_block("on_change_phase", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No); // TODO: check tooltipped
        });
        vd.field_validated_key_block("on_join", |key, block, data| {
            // Docs say it's Struggle scope but that's wrong.
            let mut sc = ScopeContext::new(Scopes::Character, key);
            validate_effect(block, data, &mut sc, Tooltipped::No); // TODO: check tooltipped
        });
    }
}

fn validate_phase(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_item("background", Item::File);
    vd.req_field("future_phases");
    vd.field_validated_block("future_phases", |block, data| {
        let mut vd = Validator::new(block, data);
        let mut has_one = false;
        for (key, block) in vd.unknown_block_fields() {
            let mut vd = Validator::new(block, data);
            has_one = true;
            data.verify_exists(Item::StrugglePhase, key); // TODO: check that it belongs to this struggle
            vd.field_bool("default");
            vd.field_validated_block("catalysts", validate_catalyst_list);
        }
        if !has_one {
            old_warn(block, ErrorKey::Validation, "must have at least one future phase");
        }
    });

    for field in &["war_effects", "culture_effects", "faith_effects", "other_effects"] {
        vd.field_validated_block(field, validate_phase_effects);
    }

    vd.field_list_items("ending_decisions", Item::Decision);
}

fn validate_catalyst_list(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, bv) in vd.unknown_fields() {
        if bv.expect_value().is_some() {
            data.verify_exists(Item::Catalyst, key);
            let mut sc = ScopeContext::new(Scopes::None, key);
            validate_scriptvalue(bv, data, &mut sc);
        }
    }
}

fn validate_phase_effects(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("common_parameters", validate_struggle_parameters);
    vd.field_validated_block("involved_parameters", validate_struggle_parameters);
    vd.field_validated_block("interloper_parameters", validate_struggle_parameters);
    vd.field_validated_block("uninvolved_parameters", validate_struggle_parameters);

    for field in &["involved_character_modifier", "interloper_character_modifier"] {
        vd.field_validated_block(field, |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
    }

    for field in &["involved_doctrine_character_modifier", "interloper_doctrine_character_modifier"]
    {
        vd.field_validated_block(field, |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("doctrine", Item::Doctrine);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
    }

    for field in &[
        "all_county_modifier",
        "involved_county_modifier",
        "interloper_county_modifier",
        "uninvolved_county_modifier",
    ] {
        vd.field_validated_block(field, |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::County, vd);
        });
    }
}

fn validate_struggle_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, value) in vd.unknown_value_fields() {
        if !value.is("yes") {
            let msg = format!("expected `{key} = yes`");
            old_warn(value, ErrorKey::Validation, &msg);
        }

        let loca = format!("struggle_parameter_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
    }
}

#[derive(Clone, Debug)]
pub struct Catalyst {}

impl Catalyst {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Catalyst, key, block, Box::new(Self {}));
    }
}

impl DbKind for Catalyst {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut _vd = Validator::new(block, data);
    }
}
