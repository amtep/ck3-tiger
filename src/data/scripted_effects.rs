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
use crate::token::Token;

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

    pub fn validate_scope_compatibility(&self, key: &str, sc: &mut ScopeContext) {
        if let Some(item) = self.effects.get(key) {
            item.validate_scope_compatibility(sc);
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
}

impl Effect {
    pub fn new(key: Token, block: Block) -> Self {
        Self {
            key,
            block,
            sc: RefCell::new(None),
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
}
