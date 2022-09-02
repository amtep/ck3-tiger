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
            ScriptValue::validate(&item.bv, data);
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
}

impl ScriptValue {
    pub fn new(key: Token, bv: BlockOrValue) -> Self {
        Self { key, bv }
    }

    fn validate_inner(vd: &mut Validator) {
        vd.field_value_loca("desc");
        vd.field_value_loca("format");
        vd.field_validated("value", Self::validate);
        vd.warn_past_known(
            "value",
            "Setting value here will overwrite the previous calculations",
        );
        vd.field_validated_bvs("add", Self::validate);
        vd.field_validated_bvs("subtract", Self::validate);
        vd.field_validated_bvs("multiply", Self::validate);
        // TODO: warn if not sure that divide by zero is impossible?
        vd.field_validated_bvs("divide", Self::validate);
        vd.field_validated_bvs("modulo", Self::validate);
        vd.field_validated_bvs("min", Self::validate);
        vd.field_validated_bvs("max", Self::validate);
        vd.field_bool("round");
        vd.field_bool("ceiling");
        vd.field_bool("floor");
        vd.field_validated_blocks("fixed_range", Self::validate_minmax_range);
        vd.field_validated_blocks("integer_range", Self::validate_minmax_range);
        // TODO: check that these actually follow each other
        vd.field_validated_blocks("if", Self::validate_if);
        vd.field_validated_blocks("else_if", Self::validate_if);
        vd.field_validated_blocks("else", Self::validate_else);
    }

    fn validate_minmax_range(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.req_field("min");
        vd.req_field("max");
        vd.field_validated_bvs("min", Self::validate);
        vd.field_validated_bvs("max", Self::validate);
        vd.warn_remaining();
    }

    fn validate_if(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_block("limit"); // TODO: validate trigger
        Self::validate_inner(&mut vd);
        vd.warn_remaining();
    }

    fn validate_else(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        Self::validate_inner(&mut vd);
        vd.warn_remaining();
    }

    pub fn validate_value(t: &Token, data: &Everything) {
        if t.as_str().parse::<i32>().is_ok() || t.as_str().parse::<f64>().is_ok() {
            // numeric literal is always valid
        } else {
            data.scriptvalues.verify_exists(t);
        }
    }

    pub fn validate(bv: &BlockOrValue, data: &Everything) {
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
                } else {
                    Self::validate_inner(&mut vd);
                }
                vd.warn_remaining();
            }
        }
    }
}
