use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ScriptedGui {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::ScriptedGui, ScriptedGui::add)
}

impl ScriptedGui {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("scripted_gui") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::ScriptedGui, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `scripted_gui` here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl DbKind for ScriptedGui {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::all(), key);

        vd.req_field("context_type");
        if let Some(token) = vd.field_value("context_type") {
            let mut found = false;
            for (context, scopes) in CONTEXT_TYPES {
                if token.is(context) {
                    found = true;
                    sc = ScopeContext::new(*scopes, token);
                }
            }
            if !found {
                let msg = format!("unknown scripted gui context type `{token}`");
                err(ErrorKey::Choice).weak().msg(msg).loc(token).push();
            }
        }

        // TODO
        vd.field_value("window_name");

        vd.field_choice("parent_window_token", PARENT_WINDOWS);
        // TODO
        vd.field_value("parent_window_window");
        vd.field_item("parent_scripted_gui", Item::ScriptedGui);
        vd.field_item("map_mode", Item::MapMode);

        vd.field_trigger("visible", &mut sc, Tooltipped::No);
        vd.field_validated_block("effects", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                // TODO: validate keys
                validate_effect(block, data, &mut sc, Tooltipped::Yes);
            });
        });
        vd.field_validated_block("triggers", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                // TODO: validate keys
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
        });
        // TODO: validate
        vd.field_block("properties");
        // TODO: validate
        vd.field_block("dynamic_lists");

        vd.field_variable("dirty", &mut sc);

        vd.field_trigger_rooted("ai_enabled", Scopes::Country, Tooltipped::No);
        vd.field_integer("ai_test_interval");
        vd.field_integer("ai_test_variance");
        vd.field_trigger_rooted("ai_check", Scopes::Country, Tooltipped::No);
        vd.field_choice("ai_test_scopes", AI_TEST);
        vd.field_trigger("ai_check_scope", &mut sc, Tooltipped::No);
        vd.field_validated_block("ai_weights", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                // TODO: validate keys
                let mut vd = Validator::new(block, data);
                vd.field_bool("ignore_lower_weights");
                vd.field_integer("weight");
                vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
            });
        });
        vd.field_integer("ai_max_weight_taken_per_test");
    }
}

const CONTEXT_TYPES: &[(&str, Scopes)] = &[
    ("player_context", Scopes::Country),
    ("selected_country_context", Scopes::Country),
    ("selected_state_context", Scopes::State),
    ("selected_state_context", Scopes::State),
    ("diplomacy_target_context", Scopes::Country),
    ("decision_category", Scopes::Country),
    ("diplomatic_action", Scopes::Country),
    ("national_focus_context", Scopes::Country),
    ("country_mapicon", Scopes::Country),
    ("state_mapicon", Scopes::State),
];

const PARENT_WINDOWS: &[&str] = &[
    "top_bar",
    "decision_tab",
    "technology_tab",
    "trade_tab",
    "construction_tab",
    "production_tab",
    "deployment_tab",
    "logistics_tab",
    "diplomacy_tab",
    "national_focus",
    "politics_tab",
    "selected_country_view",
    "selected_state_view",
    "selected_country_view_info",
    "selected_country_view_diplomacy",
    "army_ledger",
    "navy_ledger",
    "civilian_ledger",
    "air_ledger",
    "tech_infantry_folder",
    "tech_support_folder",
    "tech_armor_folder",
    "tech_artillery_folder",
    "tech_land_doctrine_folder",
    "tech_naval_folder",
    "tech_naval_doctrine_folder",
    "tech_air_techs_folder",
    "tech_air_doctrine_folder",
    "tech_electronics_folder",
    "tech_industry_folder",
];

const AI_TEST: &[&str] = &[
    "test_self_country",
    "test_enemy_countries",
    "test_ally_countries",
    "test_neighbouring_countries",
    "test_neighbouring_ally_countries",
    "test_neighbouring_enemy_countries",
    "test_self_owned_states",
    "test_enemy_owned_states",
    "test_ally_owned_states",
    "test_self_controlled_states",
    "test_enemy_controlled_states",
    "test_ally_controlled_states",
    "test_neighbouring_states",
    "test_neighbouring_enemy_states",
    "test_neighbouring_ally_states",
    "test_our_neighbouring_states",
    "test_our_neighbouring_states_against_allies",
    "test_our_neighbouring_states_against_enemies",
    "test_contesded_states",
    "test_if_only_major",
    "test_if_only_coastal",
];
