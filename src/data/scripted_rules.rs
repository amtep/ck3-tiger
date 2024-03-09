use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::item::{Item, ItemLoader};
use crate::parse::pdxfile::parse_pdx_internal;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct ScriptedRule {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3.union(GameFlags::Vic3), Item::ScriptedRule, ScriptedRule::add)
}

impl ScriptedRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedRule, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedRule {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        if let Some(sr_sc) = SCRIPTED_RULE_SCOPES_MAP.get(key.as_str()) {
            let mut sc = ScopeContext::new(sr_sc.root, key);
            for (name, s) in &sr_sc.names {
                sc.define_name(name, *s, key);
            }
            for (list, s) in &sr_sc.names {
                sc.define_list(list, *s, key);
            }
            validate_trigger(block, data, &mut sc, sr_sc.tooltipped);
        } else {
            let msg = "unknown scripted rule";
            err(ErrorKey::Validation).msg(msg).loc(key).push();
            let mut sc = ScopeContext::new(Scopes::non_primitive(), key);
            sc.set_strict_scopes(false);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        }
    }
}

#[derive(Debug, Clone)]
struct ScriptedRuleScopeContext {
    tooltipped: Tooltipped,
    root: Scopes,
    names: Vec<(&'static str, Scopes)>,
    lists: Vec<(&'static str, Scopes)>,
}

/// Processed version of game-specific `SCRIPTED_RULES`.
static SCRIPTED_RULE_SCOPES_MAP: Lazy<FnvHashMap<&'static str, ScriptedRuleScopeContext>> =
    Lazy::new(|| {
        let rules = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::tables::rules::SCRIPTED_RULES,
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::tables::rules::SCRIPTED_RULES,
        };
        build_scripted_rule_hashmap(rules)
    });

// Mostly copied from build_on_action_hashmap.
// TODO: more generic facility for this?
fn build_scripted_rule_hashmap(
    description: &'static str,
) -> FnvHashMap<&'static str, ScriptedRuleScopeContext> {
    let mut hash: FnvHashMap<&'static str, ScriptedRuleScopeContext> = FnvHashMap::default();

    let mut block = parse_pdx_internal(description, "scripted rule builtin scopes");
    for (key, block) in block.drain_definitions_warn() {
        let root = block.get_field_value("root").expect("internal error");
        let root = Scopes::from_snake_case(root.as_str()).expect("internal error");
        let tooltipped = if block.field_value_is("tooltipped", "yes") {
            Tooltipped::Yes
        } else {
            Tooltipped::No
        };
        let mut value =
            ScriptedRuleScopeContext { tooltipped, root, names: Vec::new(), lists: Vec::new() };
        for (key, token) in block.iter_assignments() {
            if key.is("root") || key.is("tooltipped") {
                continue;
            }
            let s = Scopes::from_snake_case(token.as_str()).expect("internal error");
            value.names.push((key.as_str(), s));
        }
        for (key, block) in block.iter_definitions() {
            if key.is("list") {
                for (key, token) in block.iter_assignments() {
                    let s = Scopes::from_snake_case(token.as_str()).expect("internal error");
                    value.lists.push((key.as_str(), s));
                }
            }
        }
        hash.insert(key.as_str(), value);
    }

    hash
}
