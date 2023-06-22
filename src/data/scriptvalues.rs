use fnv::FnvHashMap;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TriBool};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::{scope_iterator, Scopes};
use crate::token::{Loc, Token};
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_normal_trigger, validate_target};
use crate::validate::{
    precheck_iterator_fields, validate_inside_iterator, validate_iterator_fields,
    validate_scope_chain, ListType,
};

#[derive(Clone, Debug, Default)]
pub struct ScriptValues {
    scriptvalues: FnvHashMap<String, ScriptValue>,
}

impl ScriptValues {
    fn load_item(&mut self, key: &Token, bv: &BV) {
        if let Some(other) = self.scriptvalues.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "script value");
            }
        }
        self.scriptvalues
            .insert(key.to_string(), ScriptValue::new(key.clone(), bv.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.scriptvalues.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.scriptvalues.values() {
            item.validate(data);
        }
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if let Some(item) = self.scriptvalues.get(key.as_str()) {
            item.validate_call(key, data, sc);
        }
    }
}

impl FileHandler for ScriptValues {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/script_values")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, bv) in block.iter_bv_definitions_warn() {
            self.load_item(key, bv);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptValue {
    key: Token,
    bv: BV,
    cache: RefCell<FnvHashMap<Loc, ScopeContext>>,
}

impl ScriptValue {
    pub fn new(key: Token, bv: BV) -> Self {
        Self {
            key,
            bv,
            cache: RefCell::new(FnvHashMap::default()),
        }
    }

    fn validate_inner(
        mut vd: Validator,
        data: &Everything,
        sc: &mut ScopeContext,
        mut have_value: TriBool,
    ) {
        vd.field_item("desc", Item::Localization);
        vd.field_item("format", Item::Localization);

        let mut seen_if;
        let mut next_seen_if = false;
        for (token, bv) in vd.unknown_fields() {
            seen_if = next_seen_if;
            next_seen_if = false;

            // save_temporary_scope_as is now allowed in script values
            if token.is("save_temporary_scope_as") {
                if let Some(name) = bv.expect_value() {
                    sc.save_current_scope(name.as_str());
                }
            } else if token.is("value") {
                if have_value == TriBool::True {
                    let msg = "setting value here will overwrite the previous calculations";
                    warn(token, ErrorKey::Logic, msg);
                }
                have_value = TriBool::True;
                Self::validate_bv(bv, data, sc);
            } else if token.is("add") || token.is("subtract") || token.is("min") || token.is("max")
            {
                have_value = TriBool::True;
                Self::validate_bv(bv, data, sc);
            } else if token.is("multiply") || token.is("divide") || token.is("modulo") {
                if have_value == TriBool::False {
                    let msg = format!("nothing to {token} yet");
                    warn(token, ErrorKey::Logic, &msg);
                }
                Self::validate_bv(bv, data, sc);
            } else if token.is("round") || token.is("ceiling") || token.is("floor") {
                if have_value == TriBool::False {
                    let msg = format!("nothing to {token} yet");
                    warn(token, ErrorKey::Logic, &msg);
                }
                if let Some(value) = bv.expect_value() {
                    if !value.is("yes") && !value.is("no") {
                        let msg = "expected yes or no";
                        warn(value, ErrorKey::Validation, msg);
                    }
                }
            } else if token.is("fixed_range") || token.is("integer_range") {
                if have_value == TriBool::True {
                    let msg = "using fixed_range here will overwrite the previous calculations";
                    warn(token, ErrorKey::Logic, msg);
                }
                if let Some(block) = bv.expect_block() {
                    Self::validate_minmax_range(block, data, sc);
                }
                have_value = TriBool::True;
            } else if token.is("if") {
                if let Some(block) = bv.expect_block() {
                    Self::validate_if(block, data, sc);
                }
                have_value = TriBool::Maybe;
                next_seen_if = true;
            } else if token.is("else_if") {
                if !seen_if {
                    let msg = "`else_if` without preceding `if`";
                    warn(token, ErrorKey::Validation, msg);
                }
                if let Some(block) = bv.expect_block() {
                    Self::validate_if(block, data, sc);
                }
                have_value = TriBool::Maybe;
                next_seen_if = true;
            } else if token.is("else") {
                if !seen_if {
                    let msg = "`else` without preceding `if`";
                    warn(token, ErrorKey::Validation, msg);
                }
                if let Some(block) = bv.expect_block() {
                    Self::validate_else(block, data, sc);
                }
                have_value = TriBool::Maybe;
            } else {
                if let Some((it_type, it_name)) = token.split_once('_') {
                    if it_type.is("every")
                        || it_type.is("ordered")
                        || it_type.is("random")
                        || it_type.is("any")
                    {
                        if let Some((inscopes, outscope)) = scope_iterator(&it_name, data) {
                            if it_type.is("any") {
                                let msg = "cannot use `any_` iterators in a script value";
                                error(token, ErrorKey::Validation, msg);
                            }
                            sc.expect(inscopes, token);
                            if let Some(block) = bv.expect_block() {
                                let ltype = ListType::try_from(it_type.as_str()).unwrap();
                                precheck_iterator_fields(ltype, block, data, sc);
                                sc.open_scope(outscope, token.clone());
                                Self::validate_iterator(ltype, &it_name, block, data, sc);
                                sc.close();
                                have_value = TriBool::Maybe;
                            }
                        }
                        continue;
                    }
                }

                // Check for target = { script_value } or target = compare_value
                sc.open_builder();
                if validate_scope_chain(token, data, sc) {
                    if let Some(block) = bv.expect_block() {
                        sc.finalize_builder();
                        let vd = Validator::new(block, data);
                        Self::validate_inner(vd, data, sc, have_value);
                        have_value = TriBool::Maybe;
                    }
                }
                sc.close();
            }
        }
    }

    fn validate_iterator(
        ltype: ListType,
        it_name: &Token,
        block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("limit", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });

        let mut tooltipped = Tooltipped::No;
        validate_iterator_fields("", ltype, data, sc, &mut vd, &mut tooltipped);

        validate_inside_iterator(
            it_name.as_str(),
            ltype,
            block,
            data,
            sc,
            &mut vd,
            Tooltipped::No,
        );

        Self::validate_inner(vd, data, sc, TriBool::Maybe);
    }

    fn validate_minmax_range(block: &Block, data: &Everything, sc: &mut ScopeContext) {
        let mut vd = Validator::new(block, data);
        vd.req_field("min");
        vd.req_field("max");
        vd.field_validated_bvs("min", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.field_validated_bvs("max", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
    }

    fn validate_if(block: &Block, data: &Everything, sc: &mut ScopeContext) {
        let mut vd = Validator::new(block, data);
        vd.req_field_warn("limit");
        vd.field_validated_block("limit", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        Self::validate_inner(vd, data, sc, TriBool::Maybe);
    }

    fn validate_else(block: &Block, data: &Everything, sc: &mut ScopeContext) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("limit", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        Self::validate_inner(vd, data, sc, TriBool::Maybe);
    }

    pub fn validate_bv(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
        match bv {
            BV::Value(t) => validate_target(t, data, sc, Scopes::Value | Scopes::Bool),
            BV::Block(b) => {
                let mut vd = Validator::new(b, data);
                if let Some((None, _, _)) = b.iter_items().next() {
                    // It's a range like { 1 5 }
                    let vec = vd.values();
                    if vec.len() == 2 {
                        for v in vec {
                            validate_target(v, data, sc, Scopes::Value | Scopes::Bool);
                        }
                    } else {
                        warn(b, ErrorKey::Validation, "invalid script value range");
                    }
                } else {
                    Self::validate_inner(vd, data, sc, TriBool::False);
                }
            }
        }
    }

    pub fn cached_compat(&self, key: &Token, sc: &mut ScopeContext) -> bool {
        if let Some(our_sc) = self.cache.borrow().get(&key.loc) {
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
        let mut sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
        self.validate_call(&self.key, data, &mut sc);
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if !self.cached_compat(key, sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
            self.cache
                .borrow_mut()
                .insert(key.loc.clone(), our_sc.clone());
            Self::validate_bv(&self.bv, data, &mut our_sc);
            sc.expect_compatibility(&our_sc, key);
            self.cache.borrow_mut().insert(key.loc.clone(), our_sc);
        }
    }
}
