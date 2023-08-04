use std::path::{Path, PathBuf};
use std::sync::RwLock;

use fnv::FnvHashMap;

use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, exact_dup_error, BANNED_NAMES};
use crate::pdxfile::PdxFile;
use crate::report::{err, old_warn, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::{validate_non_dynamic_script_value, validate_script_value};
use crate::token::{Loc, Token};

#[derive(Debug, Default)]
pub struct ScriptValues {
    scope_overrides: FnvHashMap<String, Scopes>,
    script_values: FnvHashMap<String, ScriptValue>,
}

impl ScriptValues {
    fn load_item(&mut self, key: &Token, bv: &BV) {
        if let Some(other) = self.script_values.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                if other.bv.equivalent(bv) {
                    exact_dup_error(key, &other.key, "script value");
                } else {
                    dup_error(key, &other.key, "script value");
                }
            }
        }
        if BANNED_NAMES.contains(&key.as_str()) {
            let msg = "scriptedvalue has the same name as an important builtin";
            err(ErrorKey::NameConflict).strong().msg(msg).loc(key).push();
        } else {
            let scope_override = self.scope_overrides.get(key.as_str()).copied();
            self.script_values
                .insert(key.to_string(), ScriptValue::new(key.clone(), bv.clone(), scope_override));
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.script_values.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.script_values.values() {
            item.validate(data);
        }
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if let Some(item) = self.script_values.get(key.as_str()) {
            item.validate_call(key, data, sc);
        }
    }

    pub fn validate_non_dynamic_call(&self, key: &Token, data: &Everything) {
        if let Some(item) = self.script_values.get(key.as_str()) {
            item.validate_non_dynamic_call(data);
        }
    }
}

impl FileHandler<Block> for ScriptValues {
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
                            old_warn(part, ErrorKey::Config, &msg);
                        }
                    }
                }
                self.scope_overrides.insert(key.as_str().to_string(), scopes);
            }
        }
    }

    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/script_values")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, fullpath)
    }

    fn handle_file(&mut self, _entry: &FileEntry, block: Block) {
        for (key, bv) in block.iter_assignments_and_definitions_warn() {
            self.load_item(key, bv);
        }
    }
}

#[derive(Debug)]
pub struct ScriptValue {
    key: Token,
    bv: BV,
    cache: RwLock<FnvHashMap<Loc, ScopeContext>>,
    scope_override: Option<Scopes>,
}

impl ScriptValue {
    pub fn new(key: Token, bv: BV, scope_override: Option<Scopes>) -> Self {
        Self { key, bv, cache: RwLock::new(FnvHashMap::default()), scope_override }
    }

    pub fn cached_compat(&self, key: &Token, sc: &mut ScopeContext) -> bool {
        if let Some(our_sc) = self.cache.read().unwrap().get(&key.loc) {
            sc.expect_compatibility(our_sc, key);
            true
        } else {
            false
        }
    }

    pub fn validate(&self, data: &Everything) {
        // For some reason, script values can be set to bools as well
        if let Some(token) = self.bv.get_value() {
            if token.is("yes") || token.is("no") {
                return;
            }
        }
        let mut sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
        sc.set_strict_scopes(false);
        if self.scope_override.is_some() {
            sc.set_no_warn(true);
        }
        self.validate_call(&self.key, data, &mut sc);
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if !self.cached_compat(key, sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            our_sc.set_strict_scopes(false);
            if self.scope_override.is_some() {
                our_sc.set_no_warn(true);
            }
            self.cache.write().unwrap().insert(key.loc.clone(), our_sc.clone());
            validate_script_value(&self.bv, data, &mut our_sc);
            if let Some(scopes) = self.scope_override {
                our_sc = ScopeContext::new_unrooted(scopes, key);
                our_sc.set_strict_scopes(false);
            }
            sc.expect_compatibility(&our_sc, key);
            self.cache.write().unwrap().insert(key.loc.clone(), our_sc);
        }
    }

    pub fn validate_non_dynamic_call(&self, data: &Everything) {
        validate_non_dynamic_script_value(&self.bv, data);
    }
}
