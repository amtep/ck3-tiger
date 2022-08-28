use fnv::{FnvHashMap, FnvHashSet};
use std::path::{Path, PathBuf};

use crate::block::{Block, Date, DefinitionItem, Token};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::pdxfile::PdxFile;
use crate::provinces::ProvId;
use crate::religions::Religions;
use crate::titles::Titles;

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

    pub fn check_pod_faiths(&self, religions: &Religions, titles: &Titles) {
        let mut warned = FnvHashSet::default();

        for bookmark in [
            Date::new(1230, 1, 4),
            Date::new(1230, 1, 5),
            Date::new(1230, 1, 6),
            Date::new(1375, 7, 5),
            Date::new(1510, 1, 3),
        ] {
            for (provid, provhist) in &self.provinces {
                if let Some(capital) = titles.capital_of(*provid) {
                    let religion = provhist.block.get_field_at_date("religion", bookmark);
                    if let Some(religion) = religion.and_then(|v| v.into_value()) {
                        if let Some(faith) = religions.faiths.get(religion.as_str()) {
                            if faith.kind() == FileKind::VanillaFile && !warned.contains(provid) {
                                let msg = format!(
                                    "Vanilla religion in prov {} (county {}) at {}",
                                    provhist.key, capital, bookmark
                                );
                                warn(religion, ErrorKey::PrincesOfDarkness, &msg);
                                warned.insert(provid);
                            }
                        } else {
                            warn(religion, ErrorKey::PrincesOfDarkness, "unknown faith");
                        }
                    } else {
                        warn(&provhist.key, ErrorKey::PrincesOfDarkness, "no religion");
                    }
                }
            }
        }
    }
}

impl FileHandler for ProvinceHistories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/provinces")
    }

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
