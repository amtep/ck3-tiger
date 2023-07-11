use fnv::FnvHashMap;
use once_cell::sync::Lazy; // replace with std version once it's stable

use crate::block::BV;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::parse::pdxfile::parse_pdx_internal;
use crate::scopes::{scope_from_snake_case, Scopes};
use crate::token::Token;

#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO
struct OnActionScopeContext {
    root: Scopes,
    names: Vec<(String, Scopes)>,
    lists: Vec<(String, Scopes)>,
}

#[allow(dead_code)] // TODO
pub fn on_action_scopecontext(key: &Token, _data: &Everything) -> Option<ScopeContext> {
    if let Some(oa_sc) = ON_ACTION_SCOPES.get(key.as_str()) {
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

    None
}

#[allow(dead_code)] // TODO
static ON_ACTION_SCOPES: Lazy<FnvHashMap<String, OnActionScopeContext>> = Lazy::new(|| {
    build_on_action_hashmap(
        "
",
    )
});

#[allow(dead_code)] // TODO
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
                let root = scope_from_snake_case(root.as_str()).expect("internal error");
                let mut value = OnActionScopeContext { root, names: Vec::new(), lists: Vec::new() };
                for (key, token) in block.iter_assignments() {
                    if key.is("root") {
                        continue;
                    }
                    let s = scope_from_snake_case(token.as_str()).expect("internal error");
                    value.names.push((key.to_string(), s));
                }
                for (key, block) in block.iter_definitions() {
                    if key.is("list") {
                        for (key, token) in block.iter_assignments() {
                            let s = scope_from_snake_case(token.as_str()).expect("internal error");
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
