use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{error, ErrorKey};
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct CharacterInteractionCategories {
    categories: FnvHashMap<String, Category>,
}

impl CharacterInteractionCategories {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.categories.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "interaction category");
            }
        }
        self.categories.insert(key.to_string(), Category::new(key, block));
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

impl FileHandler<Block> for CharacterInteractionCategories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/character_interaction_categories")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, fullpath)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }

    fn finalize(&mut self) {
        let mut taken = vec![None; self.categories.len()];
        for item in self.categories.values() {
            if let Some(index) = item.index {
                let bad_range;
                if let Ok(i_taken) = usize::try_from(index) {
                    bad_range = i_taken >= taken.len();
                    if let Some(other) = taken[i_taken] {
                        let msg = format!("index duplicates the index of {other}");
                        error(&item.key, ErrorKey::DuplicateItem, &msg);
                    } else {
                        taken[i_taken] = Some(&item.key);
                    }
                } else {
                    bad_range = true;
                }
                if bad_range {
                    let msg = "index needs to be from 0 to the number of categories";
                    error(&item.key, ErrorKey::Range, msg);
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
        vd.req_field("index");
        vd.req_field("desc");
        vd.field_integer("index");
        vd.field_item("desc", Item::Localization);
        vd.field_bool("default");
    }
}
