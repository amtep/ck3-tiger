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
pub struct Houses {
    houses: FnvHashMap<String, House>,
}

impl Houses {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.houses.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "house");
            }
        }
        self.houses
            .insert(key.to_string(), House::new(key.clone(), block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.houses.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.houses.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Houses {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/dynasty_houses")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
        };

        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }
}

#[derive(Clone, Debug)]
pub struct House {
    key: Token,
    block: Block,
}

impl House {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("name");
        vd.req_field("dynasty");

        vd.field_value_item("name", Item::Localization);
        vd.field_value_item("prefix", Item::Localization);
        vd.field_value_item("motto", Item::Localization);
        vd.field_value_item("dynasty", Item::Dynasty);
        vd.field_value("forced_coa_religiongroup");
    }
}
