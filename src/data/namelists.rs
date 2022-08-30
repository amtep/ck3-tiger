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
pub struct Namelists {
    lists: FnvHashMap<String, List>,
}

impl Namelists {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.lists.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind && will_log(&key, ErrorKey::Duplicate) {
                error(
                    &key,
                    ErrorKey::Duplicate,
                    "namelist redefines an existing namelist",
                );
                info(
                    &other.key,
                    ErrorKey::Duplicate,
                    "the other namelist is here",
                );
            }
        }
        self.lists
            .insert(key.to_string(), List::new(key, block.clone()));
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.lists.values() {
            let _pause = LogPauseRaii::new(item.key.loc.kind == FileKind::VanillaFile);
            item.validate(data);
        }
    }
}

impl FileHandler for Namelists {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/culture/name_lists")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

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

        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_item(key.clone(), block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct List {
    key: Token,
    block: Block,
}

impl List {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_validated_block("mercenary_names", validate_mercenary_names);
        vd.field_validated_block("male_names", validate_name_list);
        vd.field_validated_block("female_names", validate_name_list);
        vd.field_validated_block("dynasty_names", validate_dynasty_names);
        vd.field_validated_block("cadet_dynasty_names", validate_dynasty_names);
        vd.field_value_loca("dynasty_of_location_prefix");

        vd.field_bool("always_use_patronym");
        vd.field_value_loca("patronym_prefix_male");
        vd.field_value_loca("patronym_prefix_male_vowel");
        vd.field_value_loca("patronym_suffix_male");
        vd.field_value_loca("patronym_prefix_female");
        vd.field_value_loca("patronym_prefix_female_vowel");
        vd.field_value_loca("patronym_suffix_female");

        vd.field_bool("founder_named_dynasties");
        vd.field_value_loca("bastard_dynasty_prefix");

        // TODO: these should sum to <= 100
        vd.field_integer("father_name_chance");
        vd.field_integer("mat_grf_name_chance");
        vd.field_integer("pat_grf_name_chance");

        vd.field_integer("mother_name_chance");
        vd.field_integer("mat_grm_name_chance");
        vd.field_integer("pat_grm_name_chance");

        vd.field_choice("grammar_transform", &["french"]);
        vd.field_bool("dynasty_name_first");

        vd.warn_remaining();
    }
}

fn validate_name_list(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.localization.verify_exists(token);
    }
    for (_, block) in vd.integer_blocks() {
        let mut vd = Validator::new(block, data);
        for token in vd.values() {
            data.localization.verify_exists(&token);
        }
        vd.warn_remaining();
    }
    vd.warn_remaining();
}

fn validate_mercenary_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_value_loca("name");
        vd.field_value("coat_of_arms");
        vd.warn_remaining();
    }
    vd.warn_remaining();
}

fn validate_dynasty_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.localization.verify_exists(token);
    }
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        for token in vd.values() {
            data.localization.verify_exists(token);
        }
        vd.warn_remaining();
    }
    vd.warn_remaining();
}
