use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::{Block, DefinitionItem, Token};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn, LogPauseRaii};
use crate::everything::FileHandler;
use crate::fileset::{FileEntry, FileKind};
use crate::pdxfile::PdxFile;
use crate::provinces::ProvId;

#[derive(Clone, Debug, Default)]
pub struct ProvinceHistories {
    provinces: FnvHashMap<ProvId, ProvinceHistory>,
}

impl ProvinceHistories {
    fn load_history(&mut self, id: ProvId, key: &Token, b: &Block) {
        if let Some(province) = self.provinces.get_mut(&id) {
            // Multiple entries are valid but could easily be a mistake.
            if province.key.loc.kind == key.loc.kind {
                warn(
                    key,
                    ErrorKey::Duplicate,
                    &format!("there are two entries for province id {}", id),
                );
            }
            province.block.append(&mut b.clone());
        } else {
            self.provinces
                .insert(id, ProvinceHistory::new(key.clone(), b.clone()));
        }
    }
}

impl FileHandler for ProvinceHistories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/provinces")
    }

    fn config(&mut self, _config: &Block) {}

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let block = match PdxFile::read_cp1252(entry.path(), entry.kind(), fullpath) {
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
                    if let Ok(id) = key.as_str().parse() {
                        self.load_history(id, key, b);
                    } else {
                        warn(
                            key,
                            ErrorKey::Validation,
                            "unexpected key, expected only province ids",
                        );
                    }
                }
            }
        }
    }

    fn finalize(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct ProvinceHistory {
    key: Token,
    block: Block,
}

impl ProvinceHistory {
    fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }
}
