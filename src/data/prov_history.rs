use fnv::{FnvHashMap, FnvHashSet};
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, Date, DefinitionItem};
use crate::data::provinces::ProvId;
use crate::data::religions::Religions;
use crate::data::titles::Titles;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct ProvinceHistories {
    provinces: FnvHashMap<ProvId, ProvinceHistory>,
}

impl ProvinceHistories {
    fn load_history(&mut self, id: ProvId, key: &Token, b: &Block) {
        if let Some(province) = self.provinces.get_mut(&id) {
            // Multiple entries are valid but could easily be a mistake.
            if province.key.loc.kind >= key.loc.kind {
                dup_error(key, &province.key, "province");
            }
            province.block.append(&mut b.clone());
        } else {
            self.provinces
                .insert(id, ProvinceHistory::new(key.clone(), b.clone()));
        }
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.provinces.values() {
            item.validate(data);
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
                    if let Some(religion) = religion.and_then(BlockOrValue::into_value) {
                        if !religions.is_modded_faith(&religion) && !warned.contains(provid) {
                            let msg = format!(
                                "Vanilla or unknown religion in prov {} (county {}) at {}",
                                provhist.key, capital, bookmark
                            );
                            warn(religion, ErrorKey::PrincesOfDarkness, &msg);
                            warned.insert(provid);
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

    fn validate_common(vd: &mut Validator, data: &Everything) {
        vd.field_value("culture");
        if let Some(token) = vd.field_value("religion") {
            data.religions.verify_faith_exists(token);
        }
        vd.field_choice(
            "holding",
            &[
                "none",
                "castle_holding",
                "church_holding",
                "city_holding",
                "tribal_holding",
                "auto",
            ],
        );
        vd.field_list("buildings");
        vd.field_values("special_building_slot");
        vd.field_values("special_building");
        vd.field_value("duchy_capital_building"); // TODO: check if duchy capital
    }

    fn validate_history(block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        Self::validate_common(&mut vd, data);
        vd.warn_remaining();
    }

    fn validate(&self, data: &Everything) {
        // TODO: verify that all county-capital provinces have a culture and religion
        // This needs province mappings to be loaded too
        let mut vd = Validator::new(&self.block, data);
        Self::validate_common(&mut vd, data);
        vd.field_value("terrain");
        vd.validate_history_blocks(Self::validate_history);
        vd.warn_remaining();
    }
}
