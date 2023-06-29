use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
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

        let Some(block) = PdxFile::read(entry, fullpath) else { return };

        for (key, b) in block.iter_definitions_warn() {
            self.load_item(key.clone(), b);
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
        }

        let loca = format!("game_concept_{}", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("game_concept_{}_desc", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);

        let mut vd = Validator::new(&self.block, data);
        vd.field_list("alias");
        if let Some(aliases) = self.block.get_field_list("alias") {
            for alias in aliases {
                let loca = format!("game_concept_{alias}");
                data.localization.verify_exists_implied(&loca, &alias);
            }
        }

        vd.field_item("parent", Item::GameConcept);
        if let Some(token) = vd.field_value("texture") {
            // TODO: check the file's resolution and check it against framesize and frame keys
            if !token.is("piety") {
                data.fileset.verify_exists(token);
            }
        }
        if let Some(texture) = self.block.get_field_value("texture") {
            vd.field_validated_block("framesize", validate_framesize);
            vd.field_integer("frame");
            if self.block.has_key("framesize") != self.block.has_key("frame") {
                let msg = "`framesize` and `frame` should be specified together";
                warn(&self.key, ErrorKey::Validation, msg);
            }
            if let Some(frame) = self.block.get_field_integer("frame") {
                if let Some(b) = self.block.get_field_block("framesize") {
                    let tokens = b.get_values();
                    if tokens.len() == 2 {
                        if let Ok(width) = tokens[0].as_str().parse::<u32>() {
                            if let Ok(height) = tokens[1].as_str().parse::<u32>() {
                                data.dds
                                    .validate_frame(texture, width, height, frame as u32);
                            }
                        }
                    }
                }
            }
        } else {
            vd.advice_field("framesize", "not needed without texture");
            vd.advice_field("frame", "not needed without texture");
        }
        vd.field_item("requires_dlc_flag", Item::DlcFeature);
        vd.field_bool("shown_in_encyclopedia");
    }
}
