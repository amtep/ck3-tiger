use std::fmt::Debug;
use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, exact_dup_error, BANNED_NAMES};
use crate::macros::{MacroCache, MACRO_MAP};
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Debug, Default)]
pub struct Effects {
    scope_overrides: FnvHashMap<String, Scopes>,
    effects: FnvHashMap<String, Effect>,
}

impl Effects {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.effects.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                if other.block.equivalent(&block) {
                    exact_dup_error(&key, &other.key, "scripted effect");
                } else {
                    dup_error(&key, &other.key, "scripted effect");
                }
            }
        }
        if BANNED_NAMES.contains(&key.as_str()) {
            let msg = "scripted effect has the same name as an important builtin";
            err(ErrorKey::NameConflict).strong().msg(msg).loc(key).push();
        } else {
            let scope_override = self.scope_overrides.get(key.as_str()).copied();
            if block.source.is_some() {
                MACRO_MAP.insert_loc(key.loc);
            }
            self.effects.insert(key.to_string(), Effect::new(key, block, scope_override));
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.effects.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.effects.values().map(|item| &item.key)
    }

    pub fn get(&self, key: &str) -> Option<&Effect> {
        self.effects.get(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.effects.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Effects {
    fn config(&mut self, config: &Block) {
        if let Some(block) = config.get_field_block("scope_override") {
            for (key, token) in block.iter_assignments() {
                let mut scopes = Scopes::empty();
                if token.lowercase_is("all") {
                    scopes = Scopes::all();
                } else {
                    for part in token.split('|') {
                        if let Some(scope) = Scopes::from_snake_case(part.as_str()) {
                            scopes |= scope;
                        } else {
                            let msg = format!("unknown scope type `{part}`");
                            warn(ErrorKey::Config).msg(msg).loc(part).push();
                        }
                    }
                }
                self.scope_overrides.insert(key.as_str().to_string(), scopes);
            }
        }
    }

    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_effects")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Debug)]
pub struct Effect {
    pub key: Token,
    block: Block,
    cache: MacroCache<ScopeContext>,
    scope_override: Option<Scopes>,
}

impl Effect {
    pub fn new(key: Token, block: Block, scope_override: Option<Scopes>) -> Self {
        Self { key, block, cache: MacroCache::default(), scope_override }
    }

    pub fn validate(&self, data: &Everything) {
        if self.block.source.is_none() {
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            sc.set_strict_scopes(false);
            if self.scope_override.is_some() {
                sc.set_no_warn(true);
            }
            self.validate_call(&self.key, data, &mut sc, Tooltipped::No);
        }
    }

    pub fn validate_call(
        &self,
        key: &Token,
        data: &Everything,
        sc: &mut ScopeContext,
        tooltipped: Tooltipped,
    ) {
        if !self.cached_compat(key, &[], tooltipped, sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            our_sc.set_strict_scopes(false);
            if self.scope_override.is_some() {
                our_sc.set_no_warn(true);
            }
            self.cache.insert(key, &[], tooltipped, false, our_sc.clone());
            validate_effect(&self.block, data, &mut our_sc, tooltipped);
            if let Some(scopes) = self.scope_override {
                our_sc = ScopeContext::new_unrooted(scopes, key);
                our_sc.set_strict_scopes(false);
            }
            sc.expect_compatibility(&our_sc, key);
            self.cache.insert(key, &[], tooltipped, false, our_sc);
        }
    }

    pub fn macro_parms(&self) -> Vec<&'static str> {
        self.block.macro_parms()
    }

    pub fn cached_compat(
        &self,
        key: &Token,
        args: &[(&'static str, Token)],
        tooltipped: Tooltipped,
        sc: &mut ScopeContext,
    ) -> bool {
        self.cache.perform(key, args, tooltipped, false, |our_sc| {
            sc.expect_compatibility(our_sc, key);
        })
    }

    pub fn validate_macro_expansion(
        &self,
        key: &Token,
        args: &[(&'static str, Token)],
        data: &Everything,
        sc: &mut ScopeContext,
        tooltipped: Tooltipped,
    ) {
        // Every invocation is treated as different even if the args are the same,
        // because we want to point to the correct one when reporting errors.
        if !self.cached_compat(key, args, tooltipped, sc) {
            if let Some(block) = self.block.expand_macro(args, key.loc) {
                let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
                our_sc.set_strict_scopes(false);
                if self.scope_override.is_some() {
                    our_sc.set_no_warn(true);
                }
                // Insert the dummy sc before continuing. That way, if we recurse, we'll hit
                // that dummy context instead of macro-expanding again.
                self.cache.insert(key, args, tooltipped, false, our_sc.clone());
                validate_effect(&block, data, &mut our_sc, tooltipped);
                if let Some(scopes) = self.scope_override {
                    our_sc = ScopeContext::new_unrooted(scopes, key);
                    our_sc.set_strict_scopes(false);
                }

                sc.expect_compatibility(&our_sc, key);
                self.cache.insert(key, args, tooltipped, false, our_sc);
            }
        }
    }
}
