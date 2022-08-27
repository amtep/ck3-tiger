use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, DefinitionItem, Token};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, will_log, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind, Fileset};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;

#[derive(Clone, Debug, Default)]
pub struct GameConcepts {
    concepts: FnvHashMap<String, Concept>,
}

impl GameConcepts {
    pub fn load_concept(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.concepts.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind && will_log(&key, ErrorKey::Duplicate) {
                error(
                    &key,
                    ErrorKey::Duplicate,
                    "game concept redefines an existing game concept",
                );
                info(&other.key, ErrorKey::Duplicate, "the other concept is here");
            }
        }
        self.concepts
            .insert(key.to_string(), Concept::new(key, block.clone()));
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        for concept in self.concepts.values() {
            concept.check_have_locas(locas);
        }
    }

    pub fn check_have_files(&self, fileset: &Fileset) {
        for concept in self.concepts.values() {
            concept.check_have_files(fileset);
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

        let _pause = LogPauseRaii::new(entry.kind() == FileKind::VanillaFile);

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
                    self.load_concept(key.clone(), b);
                }
            }
        }
    }

    fn finalize(&mut self) {
        for concept in self.concepts.values() {
            let _pause = LogPauseRaii::new(concept.key.loc.kind == FileKind::VanillaFile);
            if let Some(parent) = concept.block.get_field_value("parent") {
                if !self.concepts.contains_key(parent.as_str()) {
                    error(parent, ErrorKey::Validation, "game concept not found");
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
        let concept = Self { key, block };
        concept.validate();
        concept
    }

    pub fn validate(&self) {
        fn validate_framesize(block: &Block) {
            let mut vd = Validator::new(block);
            vd.req_tokens_integers_exactly(2);
            vd.warn_remaining();
        }

        let mut vd = Validator::new(&self.block);
        vd.opt_field_list("alias");
        vd.opt_field_value("parent");
        vd.opt_field_value("texture");
        if self.block.get_field_value("texture").is_some() {
            vd.opt_field_validated_block("framesize", validate_framesize);
            vd.opt_field_value("frame");
        } else {
            vd.advice_field("framesize", "not needed without texture");
            vd.advice_field("frame", "not needed without texture");
        }
        vd.opt_field_value("requires_dlc_flag");
        vd.warn_remaining();
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        let _pause = LogPauseRaii::new(self.key.loc.kind == FileKind::VanillaFile);
        let loca = format!("game_concept_{}", self.key);
        locas.verify_have_key(&loca, &self.key, "game concept");
        let loca = format!("game_concept_{}_desc", self.key);
        locas.verify_have_key(&loca, &self.key, "game concept");

        if let Some(aliases) = self.block.get_field_list("alias") {
            for alias in aliases {
                let loca = format!("game_concept_{}", alias);
                locas.verify_have_key(&loca, &alias, "game concept");
            }
        }
    }

    pub fn check_have_files(&self, fileset: &Fileset) {
        let _pause = LogPauseRaii::new(self.key.loc.kind == FileKind::VanillaFile);
        if let Some(texture) = self.block.get_field_value("texture") {
            if !texture.is("piety") {
                fileset.verify_have_file(texture);
                // TODO: check the file's resolution and check it against framesize and frame keys
            }
        }
    }
}
