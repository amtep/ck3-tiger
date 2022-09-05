use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info};
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

    pub fn verify_exists(&self, item: &Token) {
        if !self.effects.contains_key(item.as_str()) {
            error(
                item,
                ErrorKey::MissingItem,
                "effect not defined in common/scripted_effects/",
            );
        }
    }

    pub fn verify_exists_opt(&self, item: Option<&Token>) {
        if let Some(item) = item {
            self.verify_exists(item);
        }
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
