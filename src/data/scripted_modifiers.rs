use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::macrocache::MacroCache;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_modifiers;

#[derive(Clone, Debug, Default)]
pub struct ScriptedModifiers {
    scripted_modifiers: FnvHashMap<String, ScriptedModifier>,
}

impl ScriptedModifiers {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.scripted_modifiers.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "scripted modifier");
            }
        }
        self.scripted_modifiers.insert(
            key.to_string(),
            ScriptedModifier::new(key.clone(), block.clone()),
        );
    }

    pub fn exists(&self, key: &str) -> bool {
        self.scripted_modifiers.contains_key(key)
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

impl FileHandler for ScriptedModifiers {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_modifiers")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptedModifier {
    pub key: Token,
    block: Block,
    cache: MacroCache<ScopeContext>,
}

impl ScriptedModifier {
    pub fn new(key: Token, block: Block) -> Self {
        Self {
            key,
            block,
            cache: MacroCache::default(),
        }
    }

    pub fn validate(&self, data: &Everything) {
        // Validate the modifiers that aren't macros
        if self.block.source.is_none() {
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
            self.validate_call(&self.key, data, &mut sc, false);
        }
    }

    pub fn validate_call(
        &self,
        key: &Token,
        data: &Everything,
        sc: &mut ScopeContext,
        tooltipped: bool,
    ) {
        if !self.cached_compat(key, &[], sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
            self.cache.insert(key, &[], our_sc.clone());
            let mut vd = Validator::new(&self.block, data);
            validate_modifiers(&mut vd, &self.block, data, &mut our_sc, tooltipped);
            sc.expect_compatibility(&our_sc, key);
            self.cache.insert(key, &[], our_sc);
        }
    }

    pub fn macro_parms(&self) -> Vec<String> {
        self.block.macro_parms()
    }

    pub fn cached_compat(
        &self,
        key: &Token,
        args: &[(String, Token)],
        sc: &mut ScopeContext,
    ) -> bool {
        self.cache.perform(key, args, |our_sc| {
            sc.expect_compatibility(our_sc, key);
        })
    }

    pub fn validate_macro_expansion(
        &self,
        key: &Token,
        args: Vec<(String, Token)>,
        data: &Everything,
        sc: &mut ScopeContext,
        tooltipped: bool,
    ) {
        // Every invocation is treated as different even if the args are the same,
        // because we want to point to the correct one when reporting errors.
        if !self.cached_compat(key, &args, sc) {
            if let Some(block) = self.block.expand_macro(&args) {
                let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
                // Insert the dummy sc before continuing. That way, if we recurse, we'll hit
                // that dummy context instead of macro-expanding again.
                self.cache.insert(key, &args, our_sc.clone());
                let mut vd = Validator::new(&block, data);
                validate_modifiers(&mut vd, &block, data, &mut our_sc, tooltipped);
                sc.expect_compatibility(&our_sc, key);
                self.cache.insert(key, &args, our_sc);
            }
        }
    }
}