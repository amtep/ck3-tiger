//! Track scope context for the builtin on-actions in the various games.
//!
//! On-actions are script items that are called by the game engine, either at scheduled intervals
//! or when certain things happen.

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::block::BV;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::game::Game;
#[cfg(feature = "ck3")]
use crate::item::Item;
use crate::parse::pdxfile::parse_pdx_internal;
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Debug, Clone)]
struct OnActionScopeContext {
    root: Scopes,
    names: Vec<(String, Scopes)>,
    lists: Vec<(String, Scopes)>,
}

static ON_ACTION_SCOPES_MAP: Lazy<FnvHashMap<String, OnActionScopeContext>> = Lazy::new(|| {
    build_on_action_hashmap(match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::on_action::ON_ACTION_SCOPES,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::on_action::ON_ACTION_SCOPES,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::on_action::ON_ACTION_SCOPES,
    })
});

#[allow(unused_variables)] // only ck3 uses `data`
pub fn on_action_scopecontext(key: &Token, data: &Everything) -> Option<ScopeContext> {
    if let Some(oa_sc) = ON_ACTION_SCOPES_MAP.get(key.as_str()) {
        let mut sc = ScopeContext::new(oa_sc.root, key);
        for (name, s) in &oa_sc.names {
            sc.define_name(name, *s, key);
        }
        for (list, s) in &oa_sc.lists {
            sc.define_list(list, *s, key);
        }
        sc.set_strict_scopes(false);
        return Some(sc);
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        if let Some(relation) = key.as_str().strip_suffix("_quarterly_pulse") {
            if data.item_exists(Item::Relation, relation) {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                sc.define_name("quarter", Scopes::Value, key); // undocumented
                sc.set_strict_scopes(true);
                return Some(sc);
            }
        } else {
            for pfx in &["on_set_relation_", "on_remove_relation_", "on_death_relation_"] {
                if let Some(relation) = key.as_str().strip_prefix(pfx) {
                    if data.item_exists(Item::Relation, relation) {
                        let mut sc = ScopeContext::new(Scopes::Character, key);
                        sc.define_name("target", Scopes::Character, key); // undocumented
                        sc.set_strict_scopes(true);
                        return Some(sc);
                    }
                }
            }
        }
    }
    None
}

fn build_on_action_hashmap(description: &'static str) -> FnvHashMap<String, OnActionScopeContext> {
    let mut hash: FnvHashMap<String, OnActionScopeContext> = FnvHashMap::default();

    let mut block = parse_pdx_internal(description, "on action builtin scopes");
    for item in block.drain() {
        let field = item.get_field().expect("internal error");
        match field.bv() {
            BV::Value(token) => {
                // key1 = key2 means copy from key2
                let value = hash.get(token.as_str()).expect("internal error");
                hash.insert(field.key().to_string(), value.clone());
            }
            BV::Block(block) => {
                let root = block.get_field_value("root").expect("internal error");
                let root = Scopes::from_snake_case(root.as_str()).expect("internal error");
                let mut value = OnActionScopeContext { root, names: Vec::new(), lists: Vec::new() };
                for (key, token) in block.iter_assignments() {
                    if key.is("root") {
                        continue;
                    }
                    let s = Scopes::from_snake_case(token.as_str()).expect("internal error");
                    value.names.push((key.to_string(), s));
                }
                for (key, block) in block.iter_definitions() {
                    if key.is("list") {
                        for (key, token) in block.iter_assignments() {
                            let s =
                                Scopes::from_snake_case(token.as_str()).expect("internal error");
                            value.lists.push((key.to_string(), s));
                        }
                    }
                }
                hash.insert(field.key().to_string(), value);
            }
        }
    }

    hash
}
