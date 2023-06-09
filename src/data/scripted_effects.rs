use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::macrocache::MacroCache;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug, Default)]
pub struct Effects {
    effects: FnvHashMap<String, Effect>,
}

impl Effects {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.effects.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "scripted effect");
            }
        }
        self.effects
            .insert(key.to_string(), Effect::new(key.clone(), block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.effects.contains_key(key)
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

impl FileHandler for Effects {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_effects")
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
pub struct Effect {
    pub key: Token,
    block: Block,
    cache: MacroCache<ScopeContext>,
}

impl Effect {
    pub fn new(key: Token, block: Block) -> Self {
        Self {
            key,
            block,
            cache: MacroCache::default(),
        }
    }

    pub fn validate(&self, data: &Everything) {
        if self.block.source.is_none() {
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
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
        if !self.cached_compat(key, &[], sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
            self.cache.insert(key, &[], our_sc.clone());
            validate_normal_effect(&self.block, data, &mut our_sc, tooltipped);
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
        tooltipped: Tooltipped,
    ) {
        // Every invocation is treated as different even if the args are the same,
        // because we want to point to the correct one when reporting errors.
        if !self.cached_compat(key, &args, sc) {
            if let Some(block) = self.block.expand_macro(&args, key) {
                let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
                // Insert the dummy sc before continuing. That way, if we recurse, we'll hit
                // that dummy context instead of macro-expanding again.
                self.cache.insert(key, &args, our_sc.clone());
                validate_normal_effect(&block, data, &mut our_sc, tooltipped);
                sc.expect_compatibility(&our_sc, key);
                self.cache.insert(key, &args, our_sc);
            }
        }
    }
}
