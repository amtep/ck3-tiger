use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, will_log, LogPauseRaii};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Houses {
    houses: FnvHashMap<String, House>,
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

    pub fn verify_exists(&self, item: &Token) {
        if !self.houses.contains_key(item.as_str()) {
            error(
                item,
                ErrorKey::MissingItem,
                "house not defined in common/dynasty_houses/",
            );
        }
    }

    pub fn verify_exists_opt(&self, item: Option<&Token>) {
        item.map(|item| self.verify_exists(item));
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

        vd.field_value_loca("name");
        vd.field_value_loca("prefix");
        vd.field_value_loca("motto");
        if let Some(token) = vd.field_value("dynasty") {
            data.dynasties.verify_exists(token);
        }
        vd.field_value("forced_coa_religiongroup");
        vd.warn_remaining();
    }
}
