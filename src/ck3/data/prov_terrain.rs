use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{warn, ErrorKey};
use crate::token::{Loc, Token};
use crate::validator::Validator;
use crate::Severity;

use super::provinces::ProvId;

const DEFAULT_TERRAINS: &[&str] = &["default_land", "default_sea", "default_coastal_sea"];

#[derive(Clone, Debug, Default)]
pub struct ProvinceTerrains {
    provinces: FnvHashMap<ProvId, ProvinceTerrain>,
    file_loc: Option<Loc>,
    defaults: [Option<Token>; DEFAULT_TERRAINS.len()],
}

impl ProvinceTerrains {
    fn load_item(&mut self, id: ProvId, key: Token, value: Token) {
        if let Some(province) = self.provinces.get_mut(&id) {
            if province.key.loc.kind >= key.loc.kind {
                dup_error(&key, &province.key, "province");
            }
            *province = ProvinceTerrain::new(key, value);
        } else {
            self.provinces.insert(id, ProvinceTerrain::new(key, value));
        }
    }

    pub fn validate(&self, data: &Everything) {
        for (provid, item) in &self.provinces {
            item.validate(*provid, data);
        }

        // If no file was handled, for example when using `replace_paths`, self.file_loc is None.
        // TODO: Find a way to denote `ErrorLoc` error report when no loc is available.
        if self.file_loc.is_some() {
            for (name, terrains) in DEFAULT_TERRAINS.iter().zip(&self.defaults) {
                if let Some(terrain) = terrains {
                    data.verify_exists(Item::Terrain, terrain);
                } else {
                    let msg = format!("missing default terrain: {name}");
                    warn(ErrorKey::Validation)
                        .msg(msg)
                        // SAFETY: self.file_loc is `Some`
                        .loc(self.file_loc.unwrap())
                        .push();
                }
            }
        }
    }
}

impl FileHandler<Block> for ProvinceTerrains {
    fn subpath(&self) -> std::path::PathBuf {
        PathBuf::from("common/province_terrain")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with("province_terrain.txt") {
            // Omit _province_properties.txt
            return None;
        }

        PdxFile::read_detect_encoding(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        self.file_loc = Some(block.loc);
        for (key, value) in block.drain_assignments_warn() {
            if let Ok(id) = key.as_str().parse() {
                self.load_item(id, key, value);
            } else if let Some(index) = DEFAULT_TERRAINS.iter().position(|&x| x == key.as_str()) {
                if let Some(default) = &self.defaults[index] {
                    if default.loc.kind >= key.loc.kind {
                        dup_error(&key, default, "default terrain");
                    }
                } else {
                    self.defaults[index] = Some(value);
                }
            } else {
                let msg = "unexpected key, expected only province ids or default terrains";
                warn(ErrorKey::Validation).msg(msg).loc(key).push();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProvinceTerrain {
    key: Token,
    terrain: Token,
}

impl ProvinceTerrain {
    fn new(key: Token, terrain: Token) -> Self {
        Self { key, terrain }
    }

    fn validate(&self, provid: ProvId, data: &Everything) {
        data.provinces_ck3.verify_exists_provid(provid, &self.key, Severity::Error);
        data.verify_exists(Item::Terrain, &self.terrain);
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProvinceProperties {
    provinces: FnvHashMap<ProvId, ProvinceProperty>,
}

impl ProvinceProperties {
    fn load_item(&mut self, id: ProvId, key: Token, mut block: Block) {
        if let Some(province) = self.provinces.get_mut(&id) {
            // Multiple entries are valid but could easily be a mistake.
            if province.key.loc.kind >= key.loc.kind {
                dup_error(&key, &province.key, "province");
            }
            province.block.append(&mut block);
        } else {
            self.provinces.insert(id, ProvinceProperty::new(key, block));
        }
    }

    pub fn validate(&self, data: &Everything) {
        for (provid, item) in &self.provinces {
            item.validate(*provid, data);
        }
    }
}

impl FileHandler<Block> for ProvinceProperties {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/province_terrain")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with("province_properties.txt") {
            // Omit _province_terrain.txt
            return None;
        }
        PdxFile::read_detect_encoding(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            if let Ok(id) = key.as_str().parse() {
                self.load_item(id, key, block);
            } else {
                let msg = "unexpected key, expected only province ids";
                warn(ErrorKey::Validation).msg(msg).loc(key).push();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProvinceProperty {
    key: Token,
    block: Block,
}

impl ProvinceProperty {
    fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    fn validate(&self, provid: ProvId, data: &Everything) {
        data.provinces_ck3.verify_exists_provid(provid, &self.key, Severity::Error);
        let mut vd = Validator::new(&self.block, data);
        if data.provinces_ck3.is_sea_or_river(provid) {
            vd.field_validated_value("winter_severity_bias", |_, mut vd| {
                vd.maybe_is("0.0");
            });
            vd.ban_field("mild_winter_factor_override", || "sea and river province");
            vd.ban_field("normal_winter_factor_override", || "sea and river province");
            vd.ban_field("harsh_winter_factor_override", || "sea and river province");
        } else {
            vd.field_numeric_range("winter_severity_bias", 0.0..=1.0);
            vd.field_numeric_range("mild_winter_factor_override", 0.0..=1.0);
            vd.field_numeric_range("normal_winter_factor_override", 0.0..=1.0);
            vd.field_numeric_range("harsh_winter_factor_override", 0.0..=1.0);
        }
    }
}
