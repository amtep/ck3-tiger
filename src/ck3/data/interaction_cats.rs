use std::path::PathBuf;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct CharacterInteractionCategories {
    categories: TigerHashMap<&'static str, Category>,
}

impl CharacterInteractionCategories {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.categories.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "interaction category");
            }
        }
        self.categories.insert(key.as_str(), Category::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.categories.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.categories.values().map(|item| &item.key)
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

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
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
                if index >= 0 {
                    let index = usize::try_from(index).expect("internal error");
                    if index < taken.len() {
                        if let Some(other) = taken[index] {
                            let msg = format!("index duplicates the index of {other}");
                            err(ErrorKey::DuplicateItem).msg(msg).loc(&item.key).push();
                        } else {
                            taken[index] = Some(&item.key);
                        }
                        continue;
                    }
                }
                let msg = "index needs to be from 0 to the number of categories";
                err(ErrorKey::Range).msg(msg).loc(&item.key).push();
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
