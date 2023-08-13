use std::path::PathBuf;
use std::sync::RwLock;

use fnv::FnvHashMap;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::report::{error, ErrorKey};
use crate::scopes::{scope_iterator, Scopes};
use crate::token::{Loc, Token};
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Debug, Default)]
pub struct ScriptedLists {
    lists: FnvHashMap<String, List>,
}

impl ScriptedLists {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.lists.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "scripted list");
            }
        }
        self.lists.insert(key.to_string(), List::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.lists.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.lists.values().map(|item| &item.key)
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if let Some(item) = self.lists.get(key.as_str()) {
            item.validate_call(key, data, sc);
        }
    }

    pub fn base(&self, item: &Token) -> Option<&Token> {
        self.lists.get(item.as_str()).and_then(|item| item.block.get_field_value("base"))
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.lists.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for ScriptedLists {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_lists")
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
pub struct List {
    pub key: Token,
    block: Block,
    cache: RwLock<FnvHashMap<Loc, ScopeContext>>,
}

impl List {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block, cache: RwLock::new(FnvHashMap::default()) }
    }

    fn cached_compat(&self, key: &Token, sc: &mut ScopeContext) -> bool {
        if let Some(our_sc) = self.cache.read().unwrap().get(&key.loc) {
            sc.expect_compatibility(our_sc, key);
            true
        } else {
            false
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("base");
        vd.req_field("conditions");

        if let Some(token) = vd.field_value("base") {
            let mut sc = ScopeContext::new(Scopes::all(), token);
            sc.set_strict_scopes(false);
            if let Some((_, outscope)) = scope_iterator(token, data, &mut sc) {
                let mut sc = ScopeContext::new_unrooted(outscope, token);
                sc.set_strict_scopes(false);
                vd.field_validated_block("conditions", |block, data| {
                    validate_trigger(block, data, &mut sc, Tooltipped::No);
                });
            } else {
                error(token, ErrorKey::MissingItem, "no such base list");
            }
        }
    }

    fn validate_conditions(block: &Block, data: &Everything, sc: &mut ScopeContext) {
        if let Some(token) = block.get_field_value("base") {
            if let Some((_, outscope)) = scope_iterator(token, data, sc) {
                if let Some(block) = block.get_field_block("conditions") {
                    sc.open_scope(outscope, token.clone());
                    validate_trigger(block, data, sc, Tooltipped::No);
                    sc.close();
                }
            }
        }
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if !self.cached_compat(key, sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            our_sc.set_strict_scopes(false);
            self.cache.write().unwrap().insert(key.loc.clone(), our_sc.clone());
            Self::validate_conditions(&self.block, data, &mut our_sc);
            sc.expect_compatibility(&our_sc, key);
            self.cache.write().unwrap().insert(key.loc.clone(), our_sc);
        }
    }
}
