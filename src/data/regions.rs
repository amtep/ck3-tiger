use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Regions {
    regions: FnvHashMap<String, Region>,
    counter: usize,
}

impl Regions {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.regions.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "geographical region");
            }
        }
        self.counter += 1;
        self.regions
            .insert(key.to_string(), Region::new(key, block, self.counter));
    }

    pub fn get(&self, key: &str) -> Option<&Region> {
        self.regions.get(key)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.regions.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.regions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Regions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map_data/geographical_regions")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_item(key.clone(), block.clone());
        }
    }
}

#[derive(Clone, Debug)]
pub struct Region {
    key: Token,
    block: Block,
    sequence: usize,
}

impl Region {
    pub fn new(key: Token, block: Block, sequence: usize) -> Self {
        Self {
            key,
            block,
            sequence,
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_bool("generate_modifiers");
        vd.field_validated_list("counties", |token, data| {
            if !token.starts_with("c_") {
                let msg = "only counties can be listed in the counties field";
                error(token, ErrorKey::Validation, msg);
            }
            data.verify_exists(Item::Title, token);
        });
        vd.field_validated_list("duchies", |token, data| {
            if !token.starts_with("d_") {
                let msg = "only duchies can be listed in the duchies field";
                error(token, ErrorKey::Validation, msg);
            }
            data.verify_exists(Item::Title, token);
        });
        vd.field_list_items("provinces", Item::Province);
        vd.field_validated_list("regions", |token, data| {
            if let Some(region) = data.regions.get(token.as_str()) {
                if region.sequence >= self.sequence {
                    let msg = format!(
                        "region {token} should be defined before the region that includes it"
                    );
                    warn(token, ErrorKey::Validation, &msg);
                }
            } else {
                let msg = format!(
                    "{} {} not defined in {}",
                    Item::Region,
                    token,
                    Item::Region.path()
                );
                let info = "this will cause a crash";
                error_info(token, ErrorKey::Crash, &msg, info);
            }
        });
    }
}
