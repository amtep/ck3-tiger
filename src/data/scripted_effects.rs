use fnv::FnvHashMap;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::{Loc, Token};

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

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
        };

        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub key: Token,
    block: Block,
    sc: RefCell<Option<ScopeContext>>,
    cache: RefCell<FnvHashMap<Loc, ScopeContext>>,
}

impl Effect {
    pub fn new(key: Token, block: Block) -> Self {
        Self {
            key,
            block,
            sc: RefCell::new(None),
            cache: RefCell::new(FnvHashMap::default()),
        }
    }

    pub fn validate(&self, data: &Everything) {
        if self.block.source.is_none() {
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
            // TODO: decide what to do about "tooltipped" true vs false
            validate_normal_effect(&self.block, data, &mut sc, false);
            self.sc.replace(Some(sc));
        }
    }

    pub fn validate_scope_compatibility(&self, their_sc: &mut ScopeContext) {
        if let Some(our_sc) = self.sc.borrow().as_ref() {
            their_sc.expect_compatibility(our_sc);
        }
    }

    pub fn macro_parms(&self) -> Vec<String> {
        self.block.macro_parms()
    }

    pub fn cached_compat(&self, loc: &Loc, sc: &mut ScopeContext) -> bool {
        if let Some(our_sc) = self.cache.borrow().get(loc) {
            sc.expect_compatibility(our_sc);
            true
        } else {
            false
        }
    }

    // TODO: avoid duplicate error messages when invoking a macro effect many times
    pub fn validate_macro_expansion(
        &self,
        args: Vec<(String, Token)>,
        data: &Everything,
        sc: &mut ScopeContext,
        tooltipped: bool,
    ) {
        // arg[0].1.loc gives us the unique location of this macro call.
        // Every invocation is treated as different even if the args are the same,
        // because we want to point to the correct one when reporting errors.
        let loc = args[0].1.loc.clone();

        if !self.cached_compat(&loc, sc) {
            if let Some(block) = self.block.expand_macro(args) {
                let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
                // Insert the dummy sc before continuing. That way, if we recurse, we'll hit
                // that dummy context instead of macro-expanding again.
                self.cache.borrow_mut().insert(loc.clone(), our_sc.clone());
                validate_normal_effect(&block, data, &mut our_sc, tooltipped);
                sc.expect_compatibility(&our_sc);
                self.cache.borrow_mut().insert(loc, our_sc);
            }
        }
    }
}
