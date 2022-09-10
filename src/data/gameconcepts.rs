use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, DefinitionItem};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct GameConcepts {
    concepts: FnvHashMap<String, Concept>,
    aliases: FnvHashMap<String, String>,
}

impl GameConcepts {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.concepts.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "game concept");
            }
        }
        if let Some(list) = block.get_field_list("alias") {
            for token in list {
                self.aliases.insert(token.to_string(), key.to_string());
            }
        }
        self.concepts
            .insert(key.to_string(), Concept::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.concepts.contains_key(key) || self.aliases.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.concepts.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for GameConcepts {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/game_concepts")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
        };

        for def in block.iter_definitions_warn() {
            match def {
                DefinitionItem::Keyword(key) => error_info(
                    key,
                    ErrorKey::Validation,
                    "unexpected token",
                    "Did you forget an = ?",
                ),
                DefinitionItem::Assignment(key, _) => {
                    error(key, ErrorKey::Validation, "unexpected assignment");
                }
                DefinitionItem::Definition(key, b) => {
                    self.load_item(key.clone(), b);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Concept {
    key: Token,
    block: Block,
}

impl Concept {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        fn validate_framesize(block: &Block, data: &Everything) {
            let mut vd = Validator::new(block, data);
            vd.req_tokens_integers_exactly(2);
            vd.warn_remaining();
        }

        let loca = format!("game_concept_{}", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("game_concept_{}_desc", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);

        let mut vd = Validator::new(&self.block, data);
        vd.field_list("alias");
        if let Some(aliases) = self.block.get_field_list("alias") {
            for alias in aliases {
                let loca = format!("game_concept_{}", alias);
                data.localization.verify_exists_implied(&loca, &alias);
            }
        }

        vd.field_value_item("parent", Item::GameConcept);
        if let Some(token) = vd.field_value("texture") {
            // TODO: check the file's resolution and check it against framesize and frame keys
            if !token.is("piety") {
                data.fileset.verify_exists(token);
            }
        }
        if self.block.get_field_value("texture").is_some() {
            vd.field_validated_block("framesize", validate_framesize);
            vd.field_value("frame");
        } else {
            vd.advice_field("framesize", "not needed without texture");
            vd.advice_field("frame", "not needed without texture");
        }
        vd.field_value("requires_dlc_flag");
        vd.field_bool("shown_in_encyclopedia");
        vd.warn_remaining();
    }
}
