use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_advice, dup_error};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_color, validate_possibly_named_color};

#[derive(Clone, Debug, Default)]
pub struct Coas {
    coas: FnvHashMap<String, Coa>,
    templates: FnvHashMap<String, Coa>,
}

impl Coas {
    pub fn load_item(&mut self, key: &Token, bv: &BV) {
        if key.is("template") {
            if let Some(block) = bv.expect_block() {
                for (key, block) in block.iter_pure_definitions_warn() {
                    if let Some(other) = self.templates.get(key.as_str()) {
                        if other.key.loc.kind >= key.loc.kind {
                            if let BV::Block(otherblock) = &other.bv {
                                if otherblock.equivalent(block) {
                                    dup_advice(key, &other.key, "coa template");
                                } else {
                                    dup_error(key, &other.key, "coa template");
                                }
                            }
                        }
                    }
                    self.templates.insert(
                        key.to_string(),
                        Coa::new(key.clone(), BV::Block(block.clone().condense_tag("list"))),
                    );
                }
            }
        } else {
            if let Some(other) = self.coas.get(key.as_str()) {
                if other.key.loc.kind >= key.loc.kind {
                    if other.bv.equivalent(bv) {
                        dup_advice(key, &other.key, "coat of arms");
                    } else {
                        dup_error(key, &other.key, "coat of arms");
                    }
                }
            }
            self.coas
                .insert(key.to_string(), Coa::new(key.clone(), bv.clone()));
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.coas.contains_key(key)
    }

    pub fn template_exists(&self, key: &str) -> bool {
        self.templates.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.coas.values() {
            item.validate(data);
        }
        for item in self.templates.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Coas {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/coat_of_arms/coat_of_arms/")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read_optional_bom(entry, fullpath) else { return };
        for (key, _, bv) in block.iter_items() {
            if let Some(key) = key {
                self.load_item(key, bv);
            } else {
                error_info(
                    bv,
                    ErrorKey::Validation,
                    "unexpected item",
                    "Did you forget an = ?",
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Coa {
    key: Token,
    bv: BV,
}

impl Coa {
    pub fn new(key: Token, bv: BV) -> Self {
        Self { key, bv }
    }

    pub fn validate(&self, data: &Everything) {
        match &self.bv {
            BV::Value(token) => data.verify_exists(Item::Coa, token),
            BV::Block(block) => validate_coa_layout(block, data),
        }
    }
}

pub fn validate_coa_layout(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    if let Some(token) = vd.field_value("pattern") {
        if let Some((_, token)) = token.split_once('"') {
            data.verify_exists(Item::CoaPatternList, &token);
        } else {
            let pathname = format!("gfx/coat_of_arms/patterns/{token}");
            data.verify_exists_implied(Item::File, &pathname, token);
        }
    }

    vd.field_validated("color1", |bv, data| {
        validate_coa_color(bv, None, data);
    });
    vd.field_validated("color2", |bv, data| {
        validate_coa_color(bv, None, data);
    });
    vd.field_validated("color3", |bv, data| {
        validate_coa_color(bv, None, data);
    });
    vd.field_validated("color4", |bv, data| {
        validate_coa_color(bv, None, data);
    });

    vd.field_validated_blocks("colored_emblem", |subblock, data| {
        let mut vd = Validator::new(subblock, data);
        vd.req_field("texture");
        if let Some(token) = vd.field_value("texture") {
            if let Some((_, token)) = token.split_once('"') {
                data.verify_exists(Item::CoaColoredEmblemList, &token);
            } else {
                let pathname = format!("gfx/coat_of_arms/colored_emblems/{token}");
                data.verify_exists_implied(Item::File, &pathname, token);
            }
        }

        vd.field_validated("color1", |bv, data| {
            validate_coa_color(bv, Some(block), data);
        });
        vd.field_validated("color2", |bv, data| {
            validate_coa_color(bv, Some(block), data);
        });
        vd.field_validated("color3", |bv, data| {
            validate_coa_color(bv, Some(block), data);
        });
        vd.field_validated("color4", |bv, data| {
            validate_coa_color(bv, Some(block), data);
        });
        vd.field_validated_blocks("instance", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_numeric_exactly("position", 2);
            vd.field_list_numeric_exactly("scale", 2);
            vd.field_numeric("rotation");
            vd.field_numeric("depth");
        });
        vd.field_validated_block("mask", |block, data| {
            let mut vd = Validator::new(block, data);
            for token in vd.values() {
                if let Some(mask) = token.expect_integer() {
                    if !(1..=3).contains(&mask) {
                        warn(token, ErrorKey::Range, "mask should be from 1 to 3");
                    }
                }
            }
        });
    });
    vd.field_validated_blocks("textured_emblem", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("texture");
        if let Some(token) = vd.field_value("texture") {
            if let Some((_, token)) = token.split_once('"') {
                data.verify_exists(Item::CoaTexturedEmblemList, &token);
            } else {
                let pathname = format!("gfx/coat_of_arms/textured_emblems/{token}");
                data.verify_exists_implied(Item::File, &pathname, token);
            }
        }
    });
}

fn validate_coa_color(bv: &BV, block: Option<&Block>, data: &Everything) {
    match bv {
        BV::Value(color) => {
            if let Some((_, token)) = color.split_once('"') {
                data.verify_exists(Item::CoaColorList, &token);
            } else if color.is("color1")
                || color.is("color2")
                || color.is("color3")
                || color.is("color4")
            {
                if let Some(block) = block {
                    if !block.has_key(color.as_str()) {
                        let msg = format!("setting to {color} but {color} is not defined");
                        error(color, ErrorKey::Colors, &msg);
                    }
                } else {
                    let msg = format!("setting to {color} only works in an emblem");
                    error(color, ErrorKey::Colors, &msg);
                }
            } else {
                data.verify_exists(Item::NamedColor, color);
            }
        }
        BV::Block(block) => validate_color(block, data),
    }
}

#[derive(Clone, Debug)]
pub struct CoaTemplateList {}

impl CoaTemplateList {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("coat_of_arms_template_lists") {
            for (key, block) in block.iter_pure_definitions_warn() {
                db.add(
                    Item::CoaTemplateList,
                    key.clone(),
                    block.clone(),
                    Box::new(Self {}),
                );
            }
        } else if key.is("colored_emblem_texture_lists") {
            for (key, block) in block.iter_pure_definitions_warn() {
                db.add(
                    Item::CoaColoredEmblemList,
                    key.clone(),
                    block.clone(),
                    Box::new(CoaColoredEmblemList {}),
                );
            }
        } else if key.is("color_lists") {
            for (key, block) in block.iter_pure_definitions_warn() {
                db.add(
                    Item::CoaColorList,
                    key.clone(),
                    block.clone(),
                    Box::new(CoaColorList {}),
                );
            }
        } else if key.is("pattern_texture_lists") {
            for (key, block) in block.iter_pure_definitions_warn() {
                db.add(
                    Item::CoaPatternList,
                    key.clone(),
                    block.clone(),
                    Box::new(CoaPatternList {}),
                );
            }
        } else if key.is("textured_emblem_texture_lists") {
            for (key, block) in block.iter_pure_definitions_warn() {
                db.add(
                    Item::CoaTexturedEmblemList,
                    key.clone(),
                    block.clone(),
                    Box::new(CoaTexturedEmblemList {}),
                );
            }
        } else {
            let msg = format!("unknown list type {key}");
            warn(key, ErrorKey::UnknownField, &msg);
        }
    }
}

impl DbKind for CoaTemplateList {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_coa_list(key, block, data, |bv, data| {
            if let Some(value) = bv.expect_value() {
                data.verify_exists(Item::CoaTemplate, value);
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct CoaColoredEmblemList {}

impl DbKind for CoaColoredEmblemList {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_coa_list(key, block, data, |bv, data| {
            if let Some(value) = bv.expect_value() {
                let pathname = format!("gfx/coat_of_arms/colored_emblems/{value}");
                data.verify_exists_implied(Item::File, &pathname, value);
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct CoaTexturedEmblemList {}

impl DbKind for CoaTexturedEmblemList {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_coa_list(key, block, data, |bv, data| {
            if let Some(value) = bv.expect_value() {
                let pathname = format!("gfx/coat_of_arms/textured_emblems/{value}");
                data.verify_exists_implied(Item::File, &pathname, value);
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct CoaColorList {}

impl DbKind for CoaColorList {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_coa_list(key, block, data, validate_possibly_named_color);
    }
}

#[derive(Clone, Debug)]
pub struct CoaPatternList {}

impl DbKind for CoaPatternList {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_coa_list(key, block, data, |bv, data| {
            if let Some(value) = bv.expect_value() {
                let pathname = format!("gfx/coat_of_arms/patterns/{value}");
                data.verify_exists_implied(Item::File, &pathname, value);
            }
        });
    }
}

fn validate_coa_list<F>(key: &Token, block: &Block, data: &Everything, f: F)
where
    F: Fn(&BV, &Everything),
{
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new_root(Scopes::Character, key.clone()); // TODO: may be unset
    sc.define_name("faith", Scopes::Faith, key.clone());
    sc.define_name("culture", Scopes::Culture, key.clone());
    sc.define_name("title", Scopes::LandedTitle, key.clone()); // TODO: may be unset

    // TODO: warn about duplicate values in the lists?

    for (_, bv) in vd.integer_keys() {
        f(bv, data);
    }

    vd.field_validated_blocks("special_selection", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_blocks("trigger", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        for (_, bv) in vd.integer_keys() {
            f(bv, data);
        }
        // special_selection can be nested. TODO: how far?
        vd.field_validated_blocks("special_selection", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_blocks("trigger", |block, data| {
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            });
            for (_, bv) in vd.integer_keys() {
                f(bv, data);
            }
        });
    });
}

#[derive(Clone, Debug)]
pub struct CoaDynamicDefinition {}

impl CoaDynamicDefinition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CoaDynamicDefinition, key, block, Box::new(Self {}));
    }
}

impl DbKind for CoaDynamicDefinition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::LandedTitle, key.clone());

        vd.field_validated_blocks("item", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |block, data| {
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_item("coat_of_arms", Item::Coa);
        });
    }
}
