use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_advice, dup_error};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;
use crate::validate::validate_color;

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
                                    dup_advice(&key, &other.key, "coa template");
                                } else {
                                    dup_error(&key, &other.key, "coa template");
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
                        dup_advice(&key, &other.key, "coat of arms");
                    } else {
                        dup_error(&key, &other.key, "coat of arms");
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
            BV::Block(block) => self.validate_block(block, data),
        }
    }

    pub fn validate_block(&self, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        if let Some(token) = vd.field_value("pattern") {
            if let Some((_, token)) = token.split_once('"') {
                data.verify_exists(Item::CoaList, &token);
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
                    data.verify_exists(Item::CoaList, &token);
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
                let pathname = format!("gfx/coat_of_arms/textured_emblems/{token}");
                data.verify_exists_implied(Item::File, &pathname, token);
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct CoaTemplate {
    key: Token,
    block: Block,
}

impl CoaTemplate {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        if let Some(token) = vd.field_value("pattern") {
            if let Some((_, token)) = token.split_once('"') {
                data.verify_exists(Item::CoaList, &token);
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

        vd.field_validated_blocks("colored_emblem", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("texture");
            if let Some(token) = vd.field_value("texture") {
                let pathname = format!("gfx/coat_of_arms/colored_emblems/{token}");
                data.verify_exists_implied(Item::File, &pathname, token);
            }

            vd.field_validated("color1", |bv, data| {
                validate_coa_color(bv, Some(&self.block), data);
            });
            vd.field_validated("color2", |bv, data| {
                validate_coa_color(bv, Some(&self.block), data);
            });
            vd.field_validated("color3", |bv, data| {
                validate_coa_color(bv, Some(&self.block), data);
            });
            vd.field_validated("color4", |bv, data| {
                validate_coa_color(bv, Some(&self.block), data);
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
                let pathname = format!("gfx/coat_of_arms/textured_emblems/{token}");
                data.verify_exists_implied(Item::File, &pathname, token);
            }
        });
    }
}

fn validate_coa_color(bv: &BV, block: Option<&Block>, data: &Everything) {
    match bv {
        BV::Value(color) => {
            if let Some((_, token)) = color.split_once('"') {
                data.verify_exists(Item::CoaList, &token);
            } else if color.is("color1")
                || color.is("color2")
                || color.is("color3")
                || color.is("color4")
            {
                if let Some(block) = block {
                    if !block.has_key(color.as_str()) {
                        let msg = format!("setting to {color} but {color} is not defined");
                        error(color, ErrorKey::Validation, &msg);
                    }
                } else {
                    let msg = format!("setting to {color} only works in an emblem");
                    error(color, ErrorKey::Validation, &msg);
                }
            } else {
                data.verify_exists(Item::NamedColor, color);
            }
        }
        BV::Block(block) => validate_color(block, data),
    }
}
