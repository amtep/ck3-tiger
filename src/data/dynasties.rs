use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::error_info;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Dynasties {
    dynasties: FnvHashMap<String, Dynasty>,
}

impl Dynasties {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.dynasties.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "dynasty");
            }
        }
        self.dynasties
            .insert(key.to_string(), Dynasty::new(key.clone(), block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.dynasties.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.dynasties.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Dynasties {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/dynasties")
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

        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Dynasty {
    key: Token,
    block: Block,
}

impl Dynasty {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("name");
        vd.field_value_item("name", Item::Localization);
        vd.field_value_item("prefix", Item::Localization);
        vd.field_value_item("motto", Item::Localization);
        vd.field_value_item("culture", Item::Culture);
        vd.field_value("forced_coa_religiongroup");
        vd.warn_remaining();
    }
}
