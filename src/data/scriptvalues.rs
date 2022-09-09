use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::{scope_iterator, scope_prefix, scope_to_scope, scope_value, Scopes};
use crate::token::Token;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_prefix_reference;

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
    /// TODO: calculate possible scopes for each script value
    scopes: Scopes,
}

impl ScriptValue {
    pub fn new(key: Token, bv: BlockOrValue) -> Self {
        Self {
            key,
            bv,
            scopes: Scopes::all(),
        }
    }

    fn validate_inner(mut vd: Validator, data: &Everything, mut scopes: Scopes) -> Scopes {
        vd.field_value_item("desc", Item::Localization);
        vd.field_value_item("format", Item::Localization);
        vd.field_validated("value", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.warn_past_known(
            "value",
            "Setting value here will overwrite the previous calculations",
        );
        vd.field_validated_bvs("add", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.field_validated_bvs("subtract", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.field_validated_bvs("multiply", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        // TODO: warn if not sure that divide by zero is impossible?
        vd.field_validated_bvs("divide", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.field_validated_bvs("modulo", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.field_validated_bvs("min", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.field_validated_bvs("max", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.field_bool("round");
        vd.field_bool("ceiling");
        vd.field_bool("floor");
        vd.field_validated_blocks("fixed_range", |b, data| {
            scopes = Self::validate_minmax_range(b, data, scopes);
        });
        vd.field_validated_blocks("integer_range", |b, data| {
            scopes = Self::validate_minmax_range(b, data, scopes);
        });
        // TODO: check that these actually follow each other
        vd.field_validated_blocks("if", |b, data| scopes = Self::validate_if(b, data, scopes));
        vd.field_validated_blocks("else_if", |b, data| {
            scopes = Self::validate_if(b, data, scopes);
        });
        vd.field_validated_blocks("else", |b, data| {
            scopes = Self::validate_block(b, data, scopes);
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
                    if let Some((inscope, outscope)) = scope_iterator(&it_name, data) {
                        if it_type.is("any") {
                            let msg = format!("cannot use `{}` in a script value", key);
                            error(key, ErrorKey::Validation, &msg);
                        }
                        if !inscope.intersects(scopes | Scopes::None) {
                            let msg = format!(
                                "iterator is for {} but scope seems to be {}",
                                inscope, scopes
                            );
                            warn(key, ErrorKey::Scopes, &msg);
                        } else if inscope != Scopes::None {
                            scopes &= inscope;
                        }
                        Self::validate_iterator(
                            &it_type,
                            &it_name,
                            bv.get_block().unwrap(),
                            data,
                            outscope,
                        );
                        continue;
                    }
                }
            }

            let mut first = true;
            let mut part_scopes = scopes;
            for part in key.split('.') {
                if let Some((prefix, arg)) = part.split_once(':') {
                    if let Some((inscope, outscope)) = scope_prefix(prefix.as_str()) {
                        if inscope == Scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(&part, ErrorKey::Validation, &msg);
                        }
                        if !inscope.intersects(part_scopes | Scopes::None) {
                            let msg = format!(
                                "{}: is for {} but scope seems to be {}",
                                prefix, inscope, part_scopes
                            );
                            warn(&part, ErrorKey::Scopes, &msg);
                        } else if first && inscope != Scopes::None {
                            scopes &= inscope;
                        }
                        validate_prefix_reference(&prefix, &arg, data);
                        part_scopes = outscope;
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        continue 'outer;
                    }
                } else if part.is("root") || part.is("prev") || part.is("this") {
                    if !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(&part, ErrorKey::Validation, &msg);
                    }
                    // TODO: accurate scope analysis for root, prev, this
                    part_scopes = Scopes::all();
                } else if let Some((inscope, outscope)) = scope_to_scope(part.as_str()) {
                    if inscope == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(&part, ErrorKey::Validation, &msg);
                    }
                    if !inscope.intersects(part_scopes | Scopes::None) {
                        let msg = format!(
                            "{} is for {} but scope seems to be {}",
                            part, inscope, part_scopes
                        );
                        warn(&part, ErrorKey::Scopes, &msg);
                    } else if first && inscope != Scopes::None {
                        scopes &= inscope;
                    }
                    part_scopes = outscope;
                // TODO: warn if trying to use iterator here
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    continue 'outer;
                }
                first = false;
            }
            Self::validate_block(bv.get_block().unwrap(), data, part_scopes);
        }
        vd.warn_remaining();
        scopes
    }

    fn validate_iterator(
        it_type: &Token,
        it_name: &Token,
        block: &Block,
        data: &Everything,
        mut scopes: Scopes,
    ) {
        let mut vd = Validator::new(block, data);
        vd.field_block("limit"); // TODO: validate trigger
        if it_type.is("ordered") {
            vd.field_validated_bv("order_by", |bv, data| {
                scopes = Self::validate_bv(bv, data, scopes);
            });
            vd.field_integer("position");
            vd.field_integer("min");
            vd.field_validated_bv("max", |bv, data| {
                scopes = Self::validate_bv(bv, data, scopes);
            });
            vd.field_bool("check_range_bounds");
        } else if it_type.is("random") {
            vd.field_block("weight"); // TODO: validate modifier
        }

        if it_name.is("in_list") || it_name.is("in_local_list") || it_name.is("in_global_list") {
            let have_list = vd.field_value("list").is_some();
            let have_var = vd.field_value("variable").is_some();
            if have_list == have_var {
                error(
                    block,
                    ErrorKey::Validation,
                    "must have one of `list =` or `variable =`",
                );
            }
        } else if it_name.is("in_de_facto_hierarchy") || it_name.is("in_de_jure_hierarchy") {
            if let Some(block) = vd.field_block("filter") {
                scopes = validate_normal_trigger(block, data, scopes);
            }
            if let Some(block) = vd.field_block("continue") {
                scopes = validate_normal_trigger(block, data, scopes);
            }
        } else if it_name.is("county_in_region") {
            vd.field_value_item("region", Item::Region);
        } else if it_name.is("court_position_holder") {
            vd.req_field("type");
            vd.field_value("type");
        } else if it_name.is("relation") {
            vd.req_field("type");
            vd.field_value_item("type", Item::Relation)
        } else if it_name.is("claim") {
            vd.field_choice("explicit", &["yes", "no", "all"]);
            vd.field_choice("pressed", &["yes", "no", "all"]);
        }
        Self::validate_inner(vd, data, scopes);
    }

    fn validate_minmax_range(block: &Block, data: &Everything, mut scopes: Scopes) -> Scopes {
        let mut vd = Validator::new(block, data);
        vd.req_field("min");
        vd.req_field("max");
        vd.field_validated_bvs("min", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.field_validated_bvs("max", |bv, data| {
            scopes = Self::validate_bv(bv, data, scopes);
        });
        vd.warn_remaining();
        scopes
    }

    fn validate_if(block: &Block, data: &Everything, scopes: Scopes) -> Scopes {
        let mut vd = Validator::new(block, data);
        vd.field_block("limit"); // TODO: validate trigger
        Self::validate_inner(vd, data, scopes)
    }

    fn validate_block(block: &Block, data: &Everything, scopes: Scopes) -> Scopes {
        let vd = Validator::new(block, data);
        Self::validate_inner(vd, data, scopes)
    }

    pub fn validate_value(t: &Token, data: &Everything, mut scopes: Scopes) -> Scopes {
        if t.as_str().parse::<i32>().is_ok() || t.as_str().parse::<f64>().is_ok() {
            // numeric literal is always valid
        } else {
            let part_vec = t.split('.');
            let mut part_scopes = scopes;
            for i in 0..part_vec.len() {
                let first = i == 0;
                let last = i + 1 == part_vec.len();
                let part = &part_vec[i];

                if let Some((prefix, arg)) = part.split_once(':') {
                    if let Some((inscope, outscope)) = scope_prefix(prefix.as_str()) {
                        if inscope == Scopes::None && !first {
                            let msg = format!("`{}:` makes no sense except as first part", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        if last && !outscope.contains(Scopes::Value) {
                            let msg =
                                format!("expected a numeric formula instead of `{}:` ", prefix);
                            warn(part, ErrorKey::Validation, &msg);
                        }
                        if !inscope.intersects(part_scopes | Scopes::None) {
                            let msg = format!(
                                "{}: is for {} but scope seems to be {}",
                                prefix, inscope, part_scopes
                            );
                            warn(part, ErrorKey::Scopes, &msg);
                        } else if first && inscope != Scopes::None {
                            scopes &= inscope;
                        }
                        validate_prefix_reference(&prefix, &arg, data);
                        part_scopes = outscope;
                    } else {
                        let msg = format!("unknown prefix `{}:`", prefix);
                        error(part, ErrorKey::Validation, &msg);
                        return scopes;
                    }
                } else if part.is("root") || part.is("prev") || part.is("this") {
                    if !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    } else if last {
                        let msg = format!("`{}` makes no sense as script value", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    // TODO: accurate scope analysis for root, prev, this
                    part_scopes = Scopes::all();
                } else if let Some((inscope, outscope)) = scope_to_scope(part.as_str()) {
                    if inscope == Scopes::None && !first {
                        let msg = format!("`{}` makes no sense except as first part", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if last && !outscope.intersects(Scopes::Value | Scopes::Bool) {
                        let msg = format!("expected a numeric formula instead of `{}` ", part);
                        warn(part, ErrorKey::Validation, &msg);
                    }
                    if !inscope.intersects(part_scopes | Scopes::None) {
                        let msg = format!(
                            "{} is for {} but scope seems to be {}",
                            part, inscope, part_scopes
                        );
                        warn(part, ErrorKey::Scopes, &msg);
                    } else if first && inscope != Scopes::None {
                        scopes &= inscope;
                    }
                    part_scopes = outscope;
                } else if last {
                    if let Some(inscope) = scope_value(part, data) {
                        if inscope == Scopes::None && !first {
                            let msg = format!("`{}` makes no sense except as first part", part);
                            warn(part, ErrorKey::Validation, &msg);
                        } else if !inscope.intersects(part_scopes | Scopes::None) {
                            let msg = format!(
                                "{} is for {} but scope seems to be {}",
                                part, inscope, part_scopes
                            );
                            warn(part, ErrorKey::Scopes, &msg);
                        } else if first && inscope != Scopes::None {
                            scopes &= inscope;
                        }
                    } else {
                        data.verify_exists(Item::ScriptValue, part);
                    }
                } else {
                    let msg = format!("unknown token `{}`", part);
                    error(part, ErrorKey::Validation, &msg);
                    return scopes;
                }
            }
        }
        scopes
    }

    pub fn validate_bv(bv: &BlockOrValue, data: &Everything, mut scopes: Scopes) -> Scopes {
        match bv {
            BlockOrValue::Token(t) => Self::validate_value(t, data, scopes),
            BlockOrValue::Block(b) => {
                let mut vd = Validator::new(b, data);
                if let Some((None, _, _)) = b.iter_items().next() {
                    // It's a range like { 1 5 }
                    let vec = vd.values();
                    if vec.len() == 2 {
                        for v in vec {
                            scopes = Self::validate_value(v, data, scopes);
                        }
                    } else {
                        warn(b, ErrorKey::Validation, "invalid script value range");
                    }
                    vd.warn_remaining();
                    scopes
                } else {
                    Self::validate_inner(vd, data, scopes)
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
        // TODO: record calculated scope
        Self::validate_bv(&self.bv, data, self.scopes);
    }
}
