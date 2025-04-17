use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxEncoding;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_duration;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Struggle {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Struggle, Struggle::add)
}

impl Struggle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Struggle, key, block, Box::new(Self {}));
    }
}

impl DbKind for Struggle {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
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
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Struggle, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_list_items("cultures", Item::Culture);
        vd.field_list_items("faiths", Item::Faith);
        vd.field_list_items("regions", Item::Region);

        vd.field_validated_block_sc("transition_state_duration", &mut sc, validate_duration);
        vd.field_numeric_range("involvement_prerequisite_percentage", 0.0..=1.0);

        vd.req_field("phase_list");
        vd.field_validated_block("phase_list", |block, data| {
            let mut has_one = false;
            let mut has_ending = false;
            let phases = block.iter_definitions_warn().map(|(key, _)| key).collect::<Vec<_>>();
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                data.verify_exists(Item::Localization, key);
                let loca = format!("{key}_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
                data.verify_icon("NGameIcons|STRUGGLE_PHASE_TYPE_ICON_PATH", key, ".dds");
                has_one = true;
                validate_phase(block, data, &phases);
                if let Some(vec) = block.get_field_list("ending_decisions") {
                    has_ending |= !vec.is_empty();
                }
            });
            if !has_one {
                warn(ErrorKey::Validation).msg("must have at least one phase").loc(block).push();
            }
            // TODO: Verify if it is OK to have an ending phase but no ending decisions
            if !has_ending {
                let msg = "must have at least one phase with ending_decisions";
                warn(ErrorKey::Validation).msg(msg).loc(block).push();
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
        vd.field_validated_block_rooted("on_join", Scopes::Character, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::No); // TODO: check tooltipped
        });
        vd.field_validated_block("on_monthly", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
    }
}

fn validate_phase(block: &Block, data: &Everything, phases: &[&Token]) {
    let mut vd = Validator::new(block, data);

    // Ending phase
    if vd.field_block("on_start") {
        // Undocumented
        vd.field_bool("save_progress");
        vd.field_validated_block_rooted("on_start", Scopes::Struggle, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::Yes);
        });
        vd.unknown_fields(|key, _| {
            let msg = format!("ending phase should not have {key}, which will be ignored");
            warn(ErrorKey::UnknownField).msg(msg).loc(key).push();
        });
    } else {
        vd.field_validated_block_rooted("duration", Scopes::None, |block, data, sc| {
            if let Some(bv) = block.get_field("points") {
                if let Some(token) = bv.expect_value() {
                    token.expect_integer();
                }
            } else {
                validate_duration(block, data, sc);
            }
        });

        vd.field_item("background", Item::File);
        vd.req_field("future_phases");
        vd.field_validated_block("future_phases", |block, data| {
            let mut vd = Validator::new(block, data);
            let mut has_one = false;
            vd.unknown_block_fields(|key, block| {
                let mut vd = Validator::new(block, data);
                has_one = true;
                data.verify_exists(Item::StrugglePhase, key);
                if !phases.contains(&key) {
                    let msg = format!("{key} is not a struggle phase of this struggle");
                    warn(ErrorKey::UnknownField).msg(msg).loc(key).push();
                }
                vd.field_bool("default");
                vd.field_validated_block("catalysts", validate_catalyst_list);
            });
            if !has_one {
                warn(ErrorKey::Validation)
                    .msg("must have at least one future phase")
                    .loc(block)
                    .push();
            }
        });

        for field in &["war_effects", "culture_effects", "faith_effects", "other_effects"] {
            vd.field_validated_block(field, validate_phase_effects);
        }

        vd.field_list_items("ending_decisions", Item::Decision);
    }
}

fn validate_catalyst_list(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_fields(|key, bv| {
        if bv.expect_value().is_some() {
            data.verify_exists(Item::Catalyst, key);
            let mut sc = ScopeContext::new(Scopes::None, key);
            validate_script_value(bv, data, &mut sc);
        }
    });
}

fn validate_phase_effects(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_item("name", Item::Localization);
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
    vd.unknown_value_fields(|key, value| {
        if !value.is("yes") {
            let msg = format!("expected `{key} = yes`");
            warn(ErrorKey::Validation).msg(msg).loc(value).push();
        }

        let loca = format!("struggle_parameter_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
    });
}

#[derive(Clone, Debug)]
pub struct Catalyst {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Catalyst, Catalyst::add)
}

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

#[derive(Clone, Debug)]
pub struct StruggleHistory {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::StruggleHistory, PdxEncoding::Utf8Bom, ".txt", LoadAsFile::Yes, Recursive::No, StruggleHistory::add)
}

impl StruggleHistory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::StruggleHistory, key, block, Box::new(Self {}));
    }
}

impl DbKind for StruggleHistory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);

        vd.unknown_block_fields(|key, block| {
            key.expect_date();
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("effect", |block, data| {
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
        });
    }
}
