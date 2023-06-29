use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, Date};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::{Loc, Token};

#[derive(Clone, Debug, Default)]
pub struct CultureHistories {
    histories: FnvHashMap<String, CultureHistory>,
}

impl CultureHistories {
    pub fn load_item(&mut self, key: Token, block: Block) {
        self.histories
            .insert(key.to_string(), CultureHistory::new(key, block));
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.histories.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for CultureHistories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/cultures")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        let name = entry.filename().to_string_lossy();
        if let Some(key) = name.strip_suffix(".txt") {
            let Some(block) = PdxFile::read_cp1252(entry, fullpath) else { return };
            let token = Token::new(key.to_string(), Loc::for_entry(entry));
            self.load_item(token, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct CultureHistory {
    key: Token,
    block: Block,
}

impl CultureHistory {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate_history(&self, _date: Date, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_items("discover_innovation", Item::Innovation);
        vd.field_validated_blocks("add_innovation_progress", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("culture_innovation", Item::Innovation);
            vd.field_numeric_range("progress", 0.0, 100.0);
        });
        vd.field_item("join_era", Item::CultureEra);
        vd.field_numeric_range("progress_era", 0.0, 100.0);
    }

    pub fn validate(&self, data: &Everything) {
        if self.key.starts_with("heritage_") {
            data.verify_exists(Item::CultureHeritage, &self.key);
        } else {
            data.verify_exists(Item::Culture, &self.key);
        }

        let mut vd = Validator::new(&self.block, data);
        vd.validate_history_blocks(|date, block, data| self.validate_history(date, block, data));
    }
}
