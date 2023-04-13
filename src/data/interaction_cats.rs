use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct InteractionCategories {
    categories: FnvHashMap<String, Category>,
}

impl InteractionCategories {
    pub fn load_interaction(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.categories.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "interaction category");
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

impl FileHandler for InteractionCategories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/character_interaction_categories")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_interaction(key.clone(), block);
        }
    }

    fn finalize(&mut self) {
        let mut taken = vec![None; self.categories.len()];
        for item in self.categories.values() {
            if let Some(index) = item.index {
                if index >= (taken.len() as i64) || index < 0 {
                    error(
                        &item.key,
                        ErrorKey::Range,
                        "index needs to be from 0 to the number of categories",
                    );
                } else if let Some(other) = taken[index as usize] {
                    let msg = format!("index duplicates the index of {other}");
                    error(&item.key, ErrorKey::Duplicate, &msg);
                } else {
                    taken[index as usize] = Some(&item.key);
                }
            }
            // if no index, the item will warn about that in validate
        }
    }
}

#[derive(Clone, Debug)]
pub struct Category {
    key: Token,
    block: Block,
    index: Option<i64>,
}

impl Category {
    pub fn new(key: Token, block: Block) -> Self {
        let index = block.get_field_integer("index");
        Category { key, block, index }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_integer("index");
        vd.field_value_item("desc", Item::Localization);
        vd.field_bool("default");
    }
}
