use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct CourtPositionCategories {
    categories: FnvHashMap<String, Category>,
}

impl CourtPositionCategories {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.categories.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "court position category");
            }
        }
        self.categories
            .insert(key.to_string(), Category::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.categories.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.categories.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for CourtPositionCategories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/court_positions/categories")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };

        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_item(key.clone(), block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Category {
    key: Token,
    block: Block,
}

impl Category {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
    }
}
