use fnv::FnvHashMap;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, scope_value, Scopes};
use crate::token::Token;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_inside_iterator, validate_prefix_reference};

#[derive(Clone, Debug, Default)]
pub struct ScriptValues {
    scriptvalues: FnvHashMap<String, ScriptValue>,
}

impl ScriptValues {
    fn load_item(&mut self, key: &Token, bv: &BlockOrValue) {
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

    pub fn validate_scope_compatibility(&self, key: &str, sc: &mut ScopeContext) {
        if let Some(item) = self.scriptvalues.get(key) {
            item.validate_scope_compatibility(sc);
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

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
        };

        for (key, bv) in block.iter_bv_definitions_warn() {
            self.load_item(key, bv);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptValue {
    key: Token,
    bv: BlockOrValue,
    sc: RefCell<Option<ScopeContext>>,
}

impl ScriptValue {
    pub fn new(key: Token, bv: BlockOrValue) -> Self {
        Self {
            key,
            bv,
            sc: RefCell::new(None),
        }
    }

    fn validate_inner(mut vd: Validator, data: &Everything, sc: &mut ScopeContext) {
        vd.field_value_item("desc", Item::Localization);
        vd.field_value_item("format", Item::Localization);
        vd.field_validated("value", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.warn_past_known(
            "value",
            "Setting value here will overwrite the previous calculations",
        );
        vd.field_validated_bvs("add", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.field_validated_bvs("subtract", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.field_validated_bvs("multiply", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        // TODO: warn if not sure that divide by zero is impossible?
        vd.field_validated_bvs("divide", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.field_validated_bvs("modulo", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.field_validated_bvs("min", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.field_validated_bvs("max", |bv, data| {
            Self::validate_bv(bv, data, sc);
        });
        vd.field_bool("round");
        vd.field_bool("ceiling");
        vd.field_bool("floor");
        vd.field_validated_blocks("fixed_range", |b, data| {
            Self::validate_minmax_range(b, data, sc);
        });
        vd.field_validated_blocks("integer_range", |b, data| {
            Self::validate_minmax_range(b, data, sc);
        });
        // TODO: check that these actually follow each other
        vd.field_validated_blocks("if", |b, data| Self::validate_if(b, data, sc));
        vd.field_validated_blocks("else_if", |b, data| {
            Self::validate_if(b, data, sc);
        });
        vd.field_validated_blocks("else", |b, data| {
            Self::validate_block(b, data, sc);
        });

        'outer: for (key, bv) in vd.unknown_keys() {
            if let Some(token) = bv.get_value() {
                error(token, ErrorKey::Validation, "expected block, found value");
                continue;
            }

            if let Some((it_type, it_name)) = key.split_once('_') {
                if it_type.is("every")
                    || it_type.is("ordered")
                    || it_type.is("random")
                    || it_type.is("any")
                {
                    if let Some((inscopes, outscope)) = scope_iterator(&it_name, data) {
                        if it_type.is("any") {
                            let msg = format!("cannot use `{}` in a script value", key);
                            error(key, ErrorKey::Validation, &msg);
                        }
                        sc.expect(inscopes, key);
                        sc.open_scope(outscope, key.clone());
                        Self::validate_iterator(
                            &it_type,
                            &it_name,
                            bv.get_block().unwrap(),
                            data,
                            sc,
                        );
                        sc.close();
                        continue;
                    }
                }
            }

            let mut first = true;
            sc.open_builder();
            for part in key.split('.') {
                if let Some((prefix, arg)) = part.split_once(':') {
                    if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                        if inscopes == Scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(&part, ErrorKey::Validation, &msg);
                        }
                        sc.expect(inscopes, &prefix);
                        validate_prefix_reference(&prefix, &arg, data);
                        sc.replace(outscope, part);
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        sc.close();
                        continue 'outer;
                    }
                } else if part.is("root")
                    || part.is("prev")
                    || part.is("this")
                    || part.is("ROOT")
                    || part.is("PREV")
                    || part.is("THIS")
                {
                    if !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(&part, ErrorKey::Validation, &msg);
                    }
                    if part.is("root") || part.is("ROOT") {
                        sc.replace_root();
                    } else if part.is("prev") || part.is("PREV") {
                        sc.replace_prev(&part);
                    } else {
                        sc.replace_this();
                    }
                } else if let Some((inscopes, outscope)) = scope_to_scope(part.as_str()) {
                    if inscopes == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(&part, ErrorKey::Validation, &msg);
                    }
                    sc.expect(inscopes, &part);
                    sc.replace(outscope, part);
                // TODO: warn if trying to use iterator here
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    sc.close();
                    continue 'outer;
                }
                first = false;
            }
            Self::validate_block(bv.get_block().unwrap(), data, sc);
            sc.close();
        }
        vd.warn_remaining();
    }

    fn validate_iterator(
        it_type: &Token,
        it_name: &Token,
        block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("limit", |block, data| {
            validate_normal_trigger(block, data, sc, false);
        });
        // TODO: accept these fields for all list types, and warn if it's the wrong one
        if it_type.is("ordered") {
            vd.field_validated_bv("order_by", |bv, data| {
                Self::validate_bv(bv, data, sc);
            });
            vd.field_integer("position");
            vd.field_integer("min");
            vd.field_validated_bv("max", |bv, data| {
                Self::validate_bv(bv, data, sc);
            });
            vd.field_bool("check_range_bounds");
        } else if it_type.is("random") {
            vd.field_block("weight"); // TODO: validate modifier
        }

        validate_inside_iterator(
            it_name.as_str(),
            it_type.as_str(),
            block,
            data,
            sc,
            &mut vd,
            false,
        );

        Self::validate_inner(vd, data, sc);
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
        vd.warn_remaining();
    }

    fn validate_if(block: &Block, data: &Everything, sc: &mut ScopeContext) {
        let mut vd = Validator::new(block, data);
        vd.field_block("limit"); // TODO: validate trigger
        Self::validate_inner(vd, data, sc)
    }

    fn validate_block(block: &Block, data: &Everything, sc: &mut ScopeContext) {
        let vd = Validator::new(block, data);
        Self::validate_inner(vd, data, sc)
    }

    pub fn validate_value(t: &Token, data: &Everything, sc: &mut ScopeContext) {
        if t.as_str().parse::<i32>().is_ok() || t.as_str().parse::<f64>().is_ok() {
            // numeric literal is always valid
        } else {
            let part_vec = t.split('.');
            sc.open_builder();
            for i in 0..part_vec.len() {
                let first = i == 0;
                let last = i + 1 == part_vec.len();
                let part = &part_vec[i];

                if let Some((prefix, arg)) = part.split_once(':') {
                    if let Some((inscopes, outscope)) = scope_prefix(prefix.as_str()) {
                        if inscopes == Scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        if last && !outscope.contains(Scopes::Value) {
                            let msg =
                                format!("expected a numeric formula instead of `{}:` ", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        sc.expect(inscopes, &prefix);
                        validate_prefix_reference(&prefix, &arg, data);
                        sc.replace(outscope, part.clone());
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        sc.close();
                        return;
                    }
                } else if part.is("root")
                    || part.is("prev")
                    || part.is("this")
                    || part.is("ROOT")
                    || part.is("PREV")
                    || part.is("THIS")
                {
                    if !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    } else if last {
                        let msg = format!("`{}` makes no sense as script value", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if part.is("root") || part.is("ROOT") {
                        sc.replace_root();
                    } else if part.is("prev") || part.is("PREV") {
                        sc.replace_prev(part);
                    } else {
                        sc.replace_this();
                    }
                } else if let Some((inscopes, outscope)) = scope_to_scope(part.as_str()) {
                    if inscopes == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if last && !outscope.intersects(Scopes::Value | Scopes::Bool) {
                        let msg = format!("expected a numeric formula instead of `{}` ", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    sc.expect(inscopes, part);
                    sc.replace(outscope, part.clone());
                } else if last {
                    if let Some(inscopes) = scope_value(part, data) {
                        if inscopes == Scopes::None && !first {
                            let msg = format!("`{}` makes no sense except as first part", part);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        sc.expect(inscopes, part);
                        sc.replace(Scopes::Value, part.clone());
                    } else {
                        data.verify_exists(Item::ScriptValue, part);
                    }
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    sc.close();
                    return;
                }
            }
            sc.close();
        }
    }

    pub fn validate_bv(bv: &BlockOrValue, data: &Everything, sc: &mut ScopeContext) {
        match bv {
            BlockOrValue::Token(t) => Self::validate_value(t, data, sc),
            BlockOrValue::Block(b) => {
                let mut vd = Validator::new(b, data);
                if let Some((None, _, _)) = b.iter_items().next() {
                    // It's a range like { 1 5 }
                    let vec = vd.values();
                    if vec.len() == 2 {
                        for v in vec {
                            Self::validate_value(v, data, sc);
                        }
                    } else {
                        warn(b, ErrorKey::Validation, "invalid script value range");
                    }
                    vd.warn_remaining();
                } else {
                    Self::validate_inner(vd, data, sc);
                }
            }
        }
    }

    pub fn validate(&self, data: &Everything) {
        // For some reason, script values can be set to bools as well
        if let Some(token) = self.bv.get_value() {
            if token.is("yes") || token.is("no") {
                return;
            }
        }
        // TODO: if scripted values call each other, the system of scope contexts might
        // not settle down with just one loop over them.
        let mut sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
        Self::validate_bv(&self.bv, data, &mut sc);
        self.sc.replace(Some(sc));
    }

    pub fn validate_scope_compatibility(&self, their_sc: &mut ScopeContext) {
        if let Some(our_sc) = self.sc.borrow().as_ref() {
            their_sc.expect_compatibility(our_sc);
        }
    }
}
