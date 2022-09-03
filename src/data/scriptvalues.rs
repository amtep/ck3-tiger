use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::scopes::{scope_prefix, scope_to_scope, Scopes};
use crate::token::Token;

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

    pub fn verify_exists(&self, item: &Token) {
        if !self.scriptvalues.contains_key(item.as_str()) {
            error(
                item,
                ErrorKey::MissingItem,
                "script value not defined in common/script_values/",
            );
        }
    }

    pub fn verify_exists_opt(&self, item: Option<&Token>) {
        if let Some(item) = item {
            self.verify_exists(item);
        }
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.scriptvalues.values() {
            item.validate(data);
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

        let block = match PdxFile::read(entry.path(), entry.kind(), fullpath) {
            Ok(block) => block,
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{:#}", e),
                );
                return;
            }
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
    scopes: Scopes,
}

impl ScriptValue {
    pub fn new(key: Token, bv: BlockOrValue) -> Self {
        Self {
            key,
            bv,
            scopes: crate::scopes::ALL,
        }
    }

    fn validate_inner(mut vd: Validator, data: &Everything) {
        vd.field_value_loca("desc");
        vd.field_value_loca("format");
        vd.field_validated("value", Self::validate_bv);
        vd.warn_past_known(
            "value",
            "Setting value here will overwrite the previous calculations",
        );
        vd.field_validated_bvs("add", Self::validate_bv);
        vd.field_validated_bvs("subtract", Self::validate_bv);
        vd.field_validated_bvs("multiply", Self::validate_bv);
        // TODO: warn if not sure that divide by zero is impossible?
        vd.field_validated_bvs("divide", Self::validate_bv);
        vd.field_validated_bvs("modulo", Self::validate_bv);
        vd.field_validated_bvs("min", Self::validate_bv);
        vd.field_validated_bvs("max", Self::validate_bv);
        vd.field_bool("round");
        vd.field_bool("ceiling");
        vd.field_bool("floor");
        vd.field_validated_blocks("fixed_range", Self::validate_minmax_range);
        vd.field_validated_blocks("integer_range", Self::validate_minmax_range);
        // TODO: check that these actually follow each other
        vd.field_validated_blocks("if", Self::validate_if);
        vd.field_validated_blocks("else_if", Self::validate_if);
        vd.field_validated_blocks("else", Self::validate_block);

        'outer: for (key, bv) in vd.unknown_keys() {
            if let Some(token) = bv.get_value() {
                error(token, ErrorKey::Validation, "expected block, found value");
                continue;
            }
            // Here we just warn about syntactical correctness.
            // Semantic correctness is done in the separate scopes pass.
            let mut first = true;
            for part in key.split('.') {
                if let Some((prefix, _)) = part.split_once(':') {
                    // TODO: check valid values for all specific prefixes
                    if let Some((inscope, _)) = scope_prefix(prefix.as_str()) {
                        if inscope == crate::scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        continue 'outer;
                    }
                } else if part.is("root") || part.is("prev") || part.is("this") {
                    if !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                } else if let Some((inscope, _)) = scope_to_scope(part.as_str()) {
                    if inscope == crate::scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    continue 'outer;
                }
                first = false;
            }
            Self::validate_block(bv.get_block().unwrap(), data);
        }
        vd.warn_remaining();
    }

    fn validate_minmax_range(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("min");
        vd.req_field("max");
        vd.field_validated_bvs("min", Self::validate_bv);
        vd.field_validated_bvs("max", Self::validate_bv);
        vd.warn_remaining();
    }

    fn validate_if(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_block("limit"); // TODO: validate trigger
        Self::validate_inner(vd, data);
    }

    fn validate_block(block: &Block, data: &Everything) {
        let vd = Validator::new(block, data);
        Self::validate_inner(vd, data);
    }

    pub fn validate_value(t: &Token, data: &Everything) {
        if t.as_str().parse::<i32>().is_ok() || t.as_str().parse::<f64>().is_ok() {
            // numeric literal is always valid
        } else {
            let part_vec = t.split('.');
            for i in 0..part_vec.len() {
                let first = i == 0;
                let last = i + 1 == part_vec.len();
                let part = &part_vec[i];

                if let Some((prefix, _)) = part.split_once(':') {
                    // TODO: check valid values for all specific prefixes
                    if let Some((inscope, outscope)) = scope_prefix(prefix.as_str()) {
                        if inscope == crate::scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        if last && (outscope & crate::scopes::Value) == 0 {
                            let msg = format!("expected a numeric value instead of `{}:` ", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        return;
                    }
                } else if part.is("root") || part.is("prev") || part.is("this") {
                    if !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    } else if last {
                        let msg = format!("`{}` makes no sense as script value", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                } else if let Some((inscope, outscope)) = scope_to_scope(part.as_str()) {
                    if inscope == crate::scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if last && (outscope & crate::scopes::Value) == 0 {
                        let msg = format!("expected a numeric value instead of `{}` ", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                } else if last {
                    data.scriptvalues.verify_exists(part);
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    return;
                }
            }
        }
    }

    pub fn validate_bv(bv: &BlockOrValue, data: &Everything) {
        match bv {
            BlockOrValue::Token(t) => Self::validate_value(t, data),
            BlockOrValue::Block(b) => {
                let mut vd = Validator::new(b, data);
                if let Some((None, _, _)) = b.iter_items().next() {
                    // It's a range like { 1 5 }
                    let vec = vd.values();
                    if vec.len() == 2 {
                        for v in vec {
                            Self::validate_value(v, data);
                        }
                    } else {
                        warn(b, ErrorKey::Validation, "invalid script value range");
                    }
                    vd.warn_remaining();
                } else {
                    Self::validate_inner(vd, data);
                }
            }
        }
    }

    pub fn validate(&self, data: &Everything) {
        Self::validate_bv(&self.bv, data);
    }
}
