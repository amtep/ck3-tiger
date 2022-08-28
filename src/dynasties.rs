use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, Token};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, will_log, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;

#[derive(Clone, Debug, Default)]
pub struct Dynasties {
    pub dynasties: FnvHashMap<String, Dynasty>,
}

impl Dynasties {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.dynasties.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind && will_log(key, ErrorKey::Duplicate) {
                error(
                    key,
                    ErrorKey::Duplicate,
                    "dynasty redefines an existing dynasty",
                );
                info(&other.key, ErrorKey::Duplicate, "the other dynasty is here");
            }
        }
        self.dynasties
            .insert(key.to_string(), Dynasty::new(key.clone(), block.clone()));
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        for dynasty in self.dynasties.values() {
            dynasty.check_have_locas(locas);
        }
    }

    pub fn verify_have_dynasty(&self, dynn: &Token) {
        if !self.dynasties.contains_key(dynn.as_str()) {
            error(
                dynn,
                ErrorKey::MissingDynasty,
                "dynasty not defined in common/dynasties/",
            );
        }
    }
}

impl FileHandler for Dynasties {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/dynasties")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

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
pub struct Dynasty {
    key: Token,
    block: Block,
}

impl Dynasty {
    pub fn new(key: Token, block: Block) -> Self {
        Self::validate(&block);
        Self { key, block }
    }

    pub fn validate(block: &Block) {
        let mut vd = Validator::new(block);

        vd.req_field_value("name");
        vd.opt_field_value("prefix");
        vd.opt_field_value("motto");
        vd.opt_field_value("culture");
        vd.warn_remaining();
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        if let Some(loca) = self.block.get_field_value("name") {
            locas.verify_have_key(loca.as_str(), loca, "dynasty");
        }
        if let Some(loca) = self.block.get_field_value("prefix") {
            locas.verify_have_key(loca.as_str(), loca, "dynasty");
        }
        if let Some(loca) = self.block.get_field_value("motto") {
            locas.verify_have_key(loca.as_str(), loca, "dynasty");
        }
    }
}
