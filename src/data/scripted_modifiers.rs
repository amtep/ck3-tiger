use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, BANNED_NAMES};
use crate::macrocache::MacroCache;
use crate::pdxfile::PdxFile;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{validate_modifiers, validate_scripted_modifier_calls};
use crate::validator::Validator;

#[derive(Debug, Default)]
pub struct ScriptedModifiers {
    scripted_modifiers: FnvHashMap<String, ScriptedModifier>,
}

impl ScriptedModifiers {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.scripted_modifiers.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "scripted modifier");
            }
        }
        if BANNED_NAMES.contains(&key.as_str()) {
            let msg = "scripted modifier has the same name as an important builtin";
            err(ErrorKey::NameConflict).strong().msg(msg).loc(key).push();
        } else {
            self.scripted_modifiers.insert(key.to_string(), ScriptedModifier::new(key, block));
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.scripted_modifiers.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.scripted_modifiers.values().map(|item| &item.key)
    }

    pub fn get(&self, key: &str) -> Option<&ScriptedModifier> {
        self.scripted_modifiers.get(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.scripted_modifiers.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for ScriptedModifiers {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_modifiers")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, fullpath)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Debug)]
pub struct ScriptedModifier {
    pub key: Token,
    block: Block,
    cache: MacroCache<ScopeContext>,
}

impl ScriptedModifier {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block, cache: MacroCache::default() }
    }

    pub fn validate(&self, data: &Everything) {
        // Validate the modifiers that aren't macros
        if self.block.source.is_none() {
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            sc.set_strict_scopes(false);
            self.validate_call(&self.key, data, &mut sc);
        }
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if !self.cached_compat(key, &[], sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            our_sc.set_strict_scopes(false);
            self.cache.insert(key, &[], Tooltipped::No, false, our_sc.clone());
            let mut vd = Validator::new(&self.block, data);
            validate_modifiers(&mut vd, &mut our_sc);
            validate_scripted_modifier_calls(vd, data, &mut our_sc);
            sc.expect_compatibility(&our_sc, key);
            self.cache.insert(key, &[], Tooltipped::No, false, our_sc);
        }
    }

    pub fn macro_parms(&self) -> Vec<&str> {
        self.block.macro_parms()
    }

    pub fn cached_compat(
        &self,
        key: &Token,
        args: &[(&str, Token)],
        sc: &mut ScopeContext,
    ) -> bool {
        self.cache.perform(key, args, Tooltipped::No, false, |our_sc| {
            sc.expect_compatibility(our_sc, key);
        })
    }

    pub fn validate_macro_expansion(
        &self,
        key: &Token,
        args: &[(&str, Token)],
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        // Every invocation is treated as different even if the args are the same,
        // because we want to point to the correct one when reporting errors.
        if !self.cached_compat(key, args, sc) {
            if let Some(block) = self.block.expand_macro(args, key) {
                let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
                our_sc.set_strict_scopes(false);
                // Insert the dummy sc before continuing. That way, if we recurse, we'll hit
                // that dummy context instead of macro-expanding again.
                self.cache.insert(key, args, Tooltipped::No, false, our_sc.clone());
                let mut vd = Validator::new(&block, data);
                validate_modifiers(&mut vd, &mut our_sc);
                validate_scripted_modifier_calls(vd, data, &mut our_sc);
                sc.expect_compatibility(&our_sc, key);
                self.cache.insert(key, args, Tooltipped::No, false, our_sc);
            }
        }
    }
}
