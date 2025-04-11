use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashSet;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct UnitLeaderTrait {}
#[derive(Clone, Debug)]
pub struct UnitLeaderSkill {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::UnitLeaderTrait, UnitLeaderTrait::add)
}

const SKILL_CATEGORIES: &[&str] = &[
    "leader_attack_skills",
    "leader_coordination_skills",
    "leader_defense_skills",
    "leader_logistics_skills",
    "leader_maneuvering_skills",
    "leader_planning_skills",
    "leader_skills",
];

const SKILL_TYPES: &[&str] = &["navy", "corps_commander", "field_marshal", "operative"];
const TRAIT_TYPES: &[&str] =
    &["all", "land", "navy", "corps_commander", "field_marshal", "operative"];
const TRAIT_TYPES2: &[&str] = &[
    "basic_trait",
    "status_trait",
    "personality_trait",
    "assignable_trait",
    "basic_terrain_trait",
    "assignable_terrain_trait",
    "exile",
];

impl UnitLeaderTrait {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if SKILL_CATEGORIES.contains(&key.as_str()) {
            db.add(Item::UnitLeaderSkill, key, block, Box::new(UnitLeaderSkill {}));
        } else if key.is("leader_traits") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::UnitLeaderTrait, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for UnitLeaderTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        let mut mk = ModifKinds::UnitLeader;

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field("type");
        if let Some(bv) = block.get_field("type") {
            match bv {
                BV::Value(token) => {
                    if !TRAIT_TYPES.contains(&token.as_str()) {
                        let msg = format!("expected one of {}", TRAIT_TYPES.join(", "));
                        err(ErrorKey::Choice).msg(msg).loc(token).push();
                    }
                    mk |= modifkinds_for_trait_type(token.as_str());
                }
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);
                    for token in vd.values() {
                        if !TRAIT_TYPES.contains(&token.as_str()) {
                            let msg = format!("expected one of {}", TRAIT_TYPES.join(", "));
                            err(ErrorKey::Choice).msg(msg).loc(token).push();
                        }
                        mk |= modifkinds_for_trait_type(token.as_str());
                    }
                }
            }
        }
        vd.field_choice("trait_type", TRAIT_TYPES2);

        vd.field_integer("attack_skill");
        vd.field_integer("defense_skill");
        vd.field_integer("logistics_skill");
        vd.field_integer("planning_skill");
        vd.field_integer("maneuvering_skill");
        vd.field_integer("coordination_skill");
        vd.field_numeric("attack_skill_factor");
        vd.field_numeric("defense_skill_factor");
        vd.field_numeric("logistics_skill_factor");
        vd.field_numeric("planning_skill_factor");
        vd.field_numeric("maneuvering_skill_factor");
        vd.field_numeric("coordination_skill_factor");
        vd.field_bool("show_in_combat");
        vd.field_item("override_effect_tooltip", Item::Localization);
        vd.field_item("custom_effect_tooltip", Item::Localization);
        vd.field_item("custom_prerequisite_tooltip", Item::Localization);
        vd.field_item("custom_gain_xp_trigger_tooltip", Item::Localization);
        vd.field_item("mutually_exclusive", Item::UnitLeaderTrait);

        vd.multi_field_validated_block("parent", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("traits", Item::UnitLeaderTrait);
            vd.field_integer("num_parents_needed");
        });
        vd.multi_field_list_items("any_parent", Item::UnitLeaderTrait);
        vd.multi_field_list_items("all_parents", Item::UnitLeaderTrait);

        vd.field_integer("gui_row");
        vd.field_integer("gui_column");
        vd.field_trigger_full("allowed", Scopes::Character, Tooltipped::Yes);
        vd.field_trigger_full("prerequisites", Scopes::Character, Tooltipped::Yes);
        vd.field_trigger_full("gain_xp", Scopes::Combatant, Tooltipped::No);
        // TODO: scope is a unit leader. ROOT is country you are from and FROM is any target nationality for agents
        vd.field_trigger_full("gain_xp_leader", Scopes::Character, Tooltipped::No);
        vd.field_integer("gain_xp_on_spotting");

        vd.field_trigger_full("unit_trigger", Scopes::Division, Tooltipped::No);
        vd.field_validated_block("unit_type", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_item("type", Item::SubUnit);
        });

        vd.field_item("slot", Item::AdvisorSlot);
        vd.field_item("specialist_advisor_trait", Item::CountryLeaderTrait);
        vd.field_item("expert_advisor_trait", Item::CountryLeaderTrait);
        vd.field_item("genius_advisor_trait", Item::CountryLeaderTrait);

        for field in &[
            "modifier",
            "non_shared_modifier",
            "corps_commander_modifier",
            "field_marshal_modifier",
        ] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("custom_modifier_tooltip", Item::Localization);
                vd.unknown_block_fields(|key, block| {
                    data.verify_exists(Item::Terrain, key);
                    let mut vd = Validator::new(block, data);
                    vd.field_numeric("attack");
                    vd.field_numeric("movement");
                    vd.field_numeric("defence");
                });
                validate_modifs(block, data, mk, vd);
            });
        }
        vd.field_validated_block("sub_unit_modifiers", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                data.verify_exists(Item::SubUnit, key);
                let vd = Validator::new(block, data);
                validate_modifs(block, data, mk, vd);
            });
        });

        vd.field_validated_block("trait_xp_factor", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::UnitLeaderTrait, key);
                value.expect_number();
            });
        });

        vd.field_effect_full("on_add", Scopes::Character, Tooltipped::Yes);
        vd.field_effect_full("on_remove", Scopes::Character, Tooltipped::Yes);
        vd.field_effect_full("daily_effect", Scopes::Character, Tooltipped::No);

        vd.field_integer("cost");

        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block_sc("new_commander_weight", &mut sc, validate_modifiers_with_base);
        vd.field_item("enable_ability", Item::Ability);
    }
}

impl DbKind for UnitLeaderSkill {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let mut seen = TigerHashSet::default();
        for (key, block) in vd.integer_blocks() {
            let mut vd = Validator::new(block, data);
            let mut mk = ModifKinds::UnitLeader;
            vd.req_field("cost");
            vd.req_field("type");
            vd.field_integer("cost");
            vd.field_choice("type", SKILL_TYPES);
            if let Some(skill_type) = block.get_field_value("type") {
                mk |= modifkinds_for_trait_type(skill_type.as_str());
            }
            vd.field_validated_block("modifier", |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, mk, vd);
            });
            if let Some(skill_type) = block.get_field_value("type") {
                if !seen.insert((key.as_str(), skill_type.as_str())) {
                    let msg = format!("duplicate skill entry for {skill_type} {key}");
                    warn(ErrorKey::DuplicateItem).msg(msg).loc(key).push();
                }
            }
        }

        for &skill_type in SKILL_TYPES {
            if skill_type == "operative" {
                continue;
            }
            if (key.is("leader_coordination_skills") || key.is("leader_maneuvering_skills"))
                && skill_type != "navy"
            {
                continue;
            }
            let range = if key.is("leader_skills") { 1..=9 } else { 1..=10 };
            for level in range {
                if !seen.contains(&(&level.to_string(), skill_type)) {
                    let msg = format!("missing skill entry for {skill_type} {level}");
                    err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
                }
            }
        }

        if key.is("leader_skills") {
            for level in 1..=2 {
                let skill_type = "operative";
                if !seen.contains(&(&level.to_string(), skill_type)) {
                    let msg = format!("missing skill entry for {skill_type} {level}");
                    err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
                }
            }
        }
    }
}

fn modifkinds_for_trait_type(s: &str) -> ModifKinds {
    match s {
        "land" | "corps_commander" | "field_marshal" => ModifKinds::Army,
        "navy" => ModifKinds::Naval,
        "operative" => ModifKinds::IntelligenceAgency,
        "all" => ModifKinds::Army | ModifKinds::Naval | ModifKinds::IntelligenceAgency,
        _ => ModifKinds::empty(),
    }
}
