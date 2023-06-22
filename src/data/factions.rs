use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
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

        if !vd.field_validated_rooted("name", Scopes::Faction, validate_desc) {
            data.verify_exists(Item::Localization, key);
        }

        if !vd.field_validated_rooted("description", Scopes::Faction, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.req_field("short_effect_desc");
        vd.field_validated_rooted("short_effect_desc", Scopes::Faction, validate_desc);

        vd.field_validated_block_rooted("demand", Scopes::Faction, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_block_rooted("update_effect", Scopes::Faction, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
        // docs say "on_declaration"
        vd.field_validated_block_rooted("on_war_start", Scopes::Faction, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_block_rooted("character_leaves", Scopes::Faction, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_key_block_rooted(
            "leader_leaves",
            Scopes::Faction,
            |key, block, data, sc| {
                sc.define_name("faction_member", Scopes::Character, key.clone());
                validate_normal_effect(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_key_block_rooted(
            "ai_join_score",
            Scopes::Character,
            |key, block, data, sc| {
                sc.define_name("faction", Scopes::Faction, key.clone());
                validate_modifiers_with_base(block, data, sc);
            },
        );
        vd.field_validated_key_block_rooted(
            "ai_create_score",
            Scopes::Character,
            |key, block, data, sc| {
                sc.define_name("target", Scopes::Character, key.clone());
                // TODO: check if it's a claimant faction before setting claimant and title
                sc.define_name("claimant", Scopes::Character, key.clone());
                sc.define_name("title", Scopes::LandedTitle, key.clone());
                validate_modifiers_with_base(block, data, sc);
            },
        );
        vd.field_validated_key_block_rooted(
            "county_join_score",
            Scopes::LandedTitle,
            |key, block, data, sc| {
                sc.define_name("faction", Scopes::Faction, key.clone());
                validate_modifiers_with_base(block, data, sc);
            },
        );
        vd.field_validated_key_block_rooted(
            "county_create_score",
            Scopes::LandedTitle,
            |key, block, data, sc| {
                sc.define_name("target", Scopes::Character, key.clone());
                validate_modifiers_with_base(block, data, sc);
            },
        );
        vd.field_script_value_rooted("county_power", Scopes::LandedTitle);
        vd.field_validated_block_rooted(
            "ai_demand_chance",
            Scopes::Faction,
            validate_modifiers_with_base,
        );
        vd.field_validated_block_rooted(
            "discontent_progress",
            Scopes::Faction,
            validate_modifiers_with_base,
        );
        vd.field_validated_rooted(
            "power_threshold",
            Scopes::Faction,
            |bv, data, sc| match bv {
                BV::Value(t) => _ = t.expect_integer(),
                BV::Block(b) => validate_modifiers_with_base(b, data, sc),
            },
        );
        vd.field_validated_block_rooted("is_valid", Scopes::Faction, |block, data, sc| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_key_block_rooted(
            "is_character_valid",
            Scopes::Character,
            |key, block, data, sc| {
                sc.define_name("faction", Scopes::Faction, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_key_block_rooted(
            "is_county_valid",
            Scopes::LandedTitle,
            |key, block, data, sc| {
                sc.define_name("faction", Scopes::Faction, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_key_block_rooted(
            "can_character_join",
            Scopes::Character,
            |key, block, data, sc| {
                sc.define_name("faction", Scopes::Faction, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::Yes);
            },
        );
        vd.field_validated_key_block_rooted(
            "can_character_create",
            Scopes::Character,
            |key, block, data, sc| {
                sc.define_name("target", Scopes::Character, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::Yes);
            },
        );
        vd.field_validated_key_block_rooted(
            "can_character_create_ui",
            Scopes::Character,
            |key, block, data, sc| {
                sc.define_name("target", Scopes::Character, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::Yes);
            },
        );
        vd.field_validated_block_rooted(
            "can_character_become_leader",
            Scopes::Character,
            |block, data, sc| {
                validate_normal_trigger(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_key_block_rooted(
            "can_county_join",
            Scopes::LandedTitle,
            |key, block, data, sc| {
                sc.define_name("faction", Scopes::Faction, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::No);
            },
        );
        vd.field_validated_key_block_rooted(
            "can_county_create",
            Scopes::LandedTitle,
            |key, block, data, sc| {
                sc.define_name("target", Scopes::Character, key.clone());
                validate_normal_trigger(block, data, sc, Tooltipped::No);
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
        vd.field_validated_block_rooted("on_creation", Scopes::Faction, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
        vd.field_validated_block_rooted("on_destroy", Scopes::Faction, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
    }
}
