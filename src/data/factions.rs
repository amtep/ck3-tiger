use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::validate_normal_effect;
use crate::everything::{Db, DbKind, Everything};
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_modifiers_with_base;

#[derive(Clone, Debug)]
pub struct Faction {}

impl Faction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Faction, key, block, Box::new(Self {}));
    }
}

impl DbKind for Faction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Faction, key);

        if let Some(bv) = vd.field("name") {
            validate_desc(bv, data, &mut sc);
        } else {
            data.verify_exists(Item::Localization, key);
        }

        if let Some(bv) = vd.field("description") {
            validate_desc(bv, data, &mut sc);
        } else {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.req_field("short_effect_desc");
        vd.field_validated_sc("short_effect_desc", &mut sc, validate_desc);

        if let Some(block) = vd.field_block("demand") {
            validate_normal_effect(block, data, &mut sc, false);
        }
        if let Some(block) = vd.field_block("update_effect") {
            validate_normal_effect(block, data, &mut sc, false);
        }
        // docs say "on_declaration"
        if let Some(block) = vd.field_block("on_war_start") {
            validate_normal_effect(block, data, &mut sc, false);
        }
        if let Some(block) = vd.field_block("character_leaves") {
            validate_normal_effect(block, data, &mut sc, false);
        }
        if let Some(block) = vd.field_block("leader_leaves") {
            validate_normal_effect(block, data, &mut sc, false);
        }
        vd.field_validated_block_rooted(
            "ai_join_score",
            Scopes::Character,
            validate_modifiers_with_base,
        );
        vd.field_validated_block_rooted(
            "ai_create_score",
            Scopes::Character,
            validate_modifiers_with_base,
        );
        vd.field_validated_block_rooted(
            "county_join_score",
            Scopes::LandedTitle,
            validate_modifiers_with_base,
        );
        vd.field_validated_block_rooted(
            "county_create_score",
            Scopes::LandedTitle,
            validate_modifiers_with_base,
        );
        vd.field_script_value_rooted("county_power", Scopes::LandedTitle);
        if let Some(block) = vd.field_block("ai_demand_chance") {
            validate_modifiers_with_base(block, data, &mut sc);
        }
        if let Some(block) = vd.field_block("discontent_progress") {
            validate_modifiers_with_base(block, data, &mut sc);
        }
        if let Some(bv) = vd.field("power_threshold") {
            match bv {
                BlockOrValue::Value(t) => _ = t.expect_integer(),
                BlockOrValue::Block(b) => validate_modifiers_with_base(b, data, &mut sc),
            }
        }
        if let Some(block) = vd.field_block("is_valid") {
            validate_normal_trigger(block, data, &mut sc, false);
        }
        vd.field_validated_block_rooted(
            "is_character_valid",
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, false);
            },
        );
        vd.field_validated_block_rooted(
            "is_county_valid",
            Scopes::LandedTitle,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, false);
            },
        );
        vd.field_validated_block_rooted(
            "can_character_join",
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, true);
            },
        );
        vd.field_validated_block_rooted(
            "can_character_create",
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, true);
            },
        );
        vd.field_validated_block_rooted(
            "can_character_create_ui",
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, true);
            },
        );
        vd.field_validated_block_rooted(
            "can_character_become_leader",
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, false);
            },
        );
        vd.field_validated_block_rooted(
            "can_county_join",
            Scopes::LandedTitle,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, false);
            },
        );
        vd.field_validated_block_rooted(
            "can_county_create",
            Scopes::LandedTitle,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, false);
            },
        );

        vd.field_bool("character_allow_create");
        vd.field_bool("character_allow_join");
        vd.field_bool("county_allow_create");
        vd.field_bool("county_allow_join");
        vd.field_bool("leaders_allowed_to_leave");
        vd.field_bool("player_can_join");
        vd.field_bool("multiple_targeting");

        vd.field_item("casus_belli", Item::CasusBelli);
        vd.field_item("special_character_title", Item::Localization);

        vd.field_bool("ignore_soft_block");
        vd.field_bool("inherit_membership");
        vd.field_bool("requires_county");
        vd.field_bool("requires_character");
        vd.field_bool("requires_leader");
        vd.field_bool("county_can_switch_to_other_faction");
        vd.field_integer("sort_order");
        vd.field_bool("show_special_title");

        // undocumented fields follow
        if let Some(block) = vd.field_block("on_creation") {
            validate_normal_effect(block, data, &mut sc, false);
        }
        if let Some(block) = vd.field_block("on_destroy") {
            validate_normal_effect(block, data, &mut sc, false);
        }
    }
}
