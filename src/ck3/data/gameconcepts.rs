use std::path::PathBuf;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{TigerHashMap, dup_error};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{ErrorKey, warn};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct GameConcepts {
    concepts: TigerHashMap<&'static str, Concept>,
    aliases: TigerHashMap<&'static str, &'static str>,
}

impl GameConcepts {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.concepts.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "game concept");
            }
        }
        self.concepts.insert(key.as_str(), Concept::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.concepts.contains_key(key) || self.aliases.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.concepts.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.concepts.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for GameConcepts {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/game_concepts")
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
        for (key, concept) in &self.concepts {
            for token in concept.block.get_multi_field_list("alias") {
                self.aliases.insert(token.as_str(), key);
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
        }

        let loca = format!("game_concept_{}", self.key);
        data.verify_exists_implied(Item::Localization, &loca, &self.key);
        let loca = format!("game_concept_{}_desc", self.key);
        data.verify_exists_implied(Item::Localization, &loca, &self.key);

        let mut vd = Validator::new(&self.block, data);
        vd.multi_field_validated_list("alias", |alias, data| {
            let loca = format!("game_concept_{alias}");
            data.verify_exists_implied(Item::Localization, &loca, alias);
        });

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
                warn(ErrorKey::Validation).msg(msg).loc(&self.key).push();
            }
            if let Some(frame) = self.block.get_field_integer("frame") {
                if let Some(b) = self.block.get_field_block("framesize") {
                    let tokens: Vec<&Token> = b.iter_values().collect();
                    if tokens.len() == 2 {
                        if let Ok(width) = tokens[0].as_str().parse::<u32>() {
                            if let Ok(height) = tokens[1].as_str().parse::<u32>() {
                                #[allow(clippy::cast_possible_truncation)] // TODO
                                #[allow(clippy::cast_sign_loss)]
                                data.dds.validate_frame(texture, width, height, frame as u32);
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
