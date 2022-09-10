use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Effects {
    effects: FnvHashMap<String, Effect>,
}

impl Effects {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.effects.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "scripted effect");
            }
        }
        self.effects
            .insert(key.to_string(), Effect::new(key.clone(), block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.effects.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.effects.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Effects {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_effects")
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
pub struct Effect {
    pub key: Token,
    block: Block,
}

impl Effect {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, _data: &Everything) {}
}
