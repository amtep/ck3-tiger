use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Faction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Faction, Faction::add)
}

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

        // TODO: docs say description but vanilla uses desc. Verify.
        if !vd.field_validated_rooted("description", Scopes::Faction, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.req_field("short_effect_desc");
        vd.field_validated_rooted("short_effect_desc", Scopes::Faction, validate_desc);

        vd.field_effect_rooted("demand", Tooltipped::No, Scopes::Faction);
        vd.field_effect_rooted("update_effect", Tooltipped::No, Scopes::Faction);
        // docs say "on_declaration"
        vd.field_effect_rooted("on_war_start", Tooltipped::No, Scopes::Faction);
        vd.field_effect_rooted("character_leaves", Tooltipped::No, Scopes::Faction);
        vd.field_effect_builder("leader_leaves", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Faction, key);
            sc.define_name("faction_member", Scopes::Character, key);
            sc
        });
        vd.field_validated_key_block("ai_join_score", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("faction", Scopes::Faction, key);
            validate_modifiers_with_base(block, data, &mut sc);
        });
        vd.field_validated_key_block("ai_create_score", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("target", Scopes::Character, key);
            // TODO: check if it's a claimant faction before setting claimant and title
            sc.define_name("claimant", Scopes::Character, key);
            sc.define_name("title", Scopes::LandedTitle, key);
            validate_modifiers_with_base(block, data, &mut sc);
        });
        vd.field_validated_key_block("county_join_score", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
            sc.define_name("faction", Scopes::Faction, key);
            validate_modifiers_with_base(block, data, &mut sc);
        });
        vd.field_validated_key_block("county_create_score", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
            sc.define_name("target", Scopes::Character, key);
            validate_modifiers_with_base(block, data, &mut sc);
        });
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
        vd.field_validated_rooted("power_threshold", Scopes::Faction, |bv, data, sc| match bv {
            BV::Value(t) => _ = t.expect_integer(),
            BV::Block(b) => validate_modifiers_with_base(b, data, sc),
        });
        vd.field_trigger_rooted("is_valid", Tooltipped::No, Scopes::Faction);
        vd.field_trigger_builder("is_character_valid", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("faction", Scopes::Faction, key);
            sc
        });
        vd.field_trigger_builder("is_county_valid", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
            sc.define_name("faction", Scopes::Faction, key);
            sc
        });
        vd.field_trigger_builder("can_character_join", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("faction", Scopes::Faction, key);
            sc
        });
        vd.field_trigger_builder("can_character_create", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });
        vd.field_trigger_builder("can_character_create_ui", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });
        vd.field_trigger_builder("can_character_become_leader", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("faction", Scopes::Faction, key);
            sc
        });
        vd.field_trigger_builder("can_county_join", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
            sc.define_name("faction", Scopes::Faction, key);
            sc
        });
        vd.field_trigger_builder("can_county_create", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
            sc.define_name("target", Scopes::Character, key);
            sc
        });

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
        vd.field_effect_rooted("on_creation", Tooltipped::No, Scopes::Faction);
        vd.field_effect_rooted("on_destroy", Tooltipped::No, Scopes::Faction);
    }
}
