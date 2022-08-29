use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::dynasties::Dynasties;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, will_log, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Houses {
    pub houses: FnvHashMap<String, House>,
}

impl Houses {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.houses.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind && will_log(key, ErrorKey::Duplicate) {
                error(
                    key,
                    ErrorKey::Duplicate,
                    "house redefines an existing house",
                );
                info(&other.key, ErrorKey::Duplicate, "the other house is here");
            }
        }
        self.houses
            .insert(key.to_string(), House::new(key.clone(), block.clone()));
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        for house in self.houses.values() {
            house.check_have_locas(locas);
        }
    }

    pub fn check_have_dynasties(&self, dynasties: &Dynasties) {
        for house in self.houses.values() {
            house.check_have_dynasty(dynasties);
        }
    }

    pub fn verify_have_house(&self, house: &Token) {
        if !self.houses.contains_key(house.as_str()) {
            error(
                house,
                ErrorKey::MissingItem,
                "house not defined in common/dynasty_houses/",
            );
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
pub struct House {
    key: Token,
    block: Block,
}

impl House {
    pub fn new(key: Token, block: Block) -> Self {
        Self::validate(&block);
        Self { key, block }
    }

    pub fn validate(block: &Block) {
        let mut vd = Validator::new(block);

        vd.req_field_value("name");
        vd.opt_field_value("prefix");
        vd.opt_field_value("motto");
        vd.req_field_value("dynasty");
        vd.warn_remaining();
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        if let Some(loca) = self.block.get_field_value("name") {
            locas.verify_have_key(loca.as_str(), loca, "house");
        }
        if let Some(loca) = self.block.get_field_value("prefix") {
            locas.verify_have_key(loca.as_str(), loca, "house");
        }
        if let Some(loca) = self.block.get_field_value("motto") {
            locas.verify_have_key(loca.as_str(), loca, "house");
        }
    }

    pub fn check_have_dynasty(&self, dynasties: &Dynasties) {
        if let Some(dynasty) = self.block.get_field_value("dynasty") {
            dynasties.verify_have_dynasty(dynasty);
        }
    }
}
