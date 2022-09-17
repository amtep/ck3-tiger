use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Namelists {
    lists: FnvHashMap<String, List>,
}

impl Namelists {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.lists.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "name list");
            }
        }
        self.lists
            .insert(key.to_string(), List::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.lists.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.lists.values() {
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

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
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
        vd.field_value_item("dynasty_of_location_prefix", Item::Localization);

        vd.field_bool("always_use_patronym");
        vd.field_value_item("patronym_prefix_male", Item::Localization);
        vd.field_value_item("patronym_prefix_male_vowel", Item::Localization);
        vd.field_value_item("patronym_suffix_male", Item::Localization);
        vd.field_value_item("patronym_prefix_female", Item::Localization);
        vd.field_value_item("patronym_prefix_female_vowel", Item::Localization);
        vd.field_value_item("patronym_suffix_female", Item::Localization);

        vd.field_bool("founder_named_dynasties");
        vd.field_value_item("bastard_dynasty_prefix", Item::Localization);

        // TODO: these should sum to <= 100
        vd.field_integer("father_name_chance");
        vd.field_integer("mat_grf_name_chance");
        vd.field_integer("pat_grf_name_chance");

        vd.field_integer("mother_name_chance");
        vd.field_integer("mat_grm_name_chance");
        vd.field_integer("pat_grm_name_chance");

        vd.field_choice("grammar_transform", &["french"]);
        vd.field_bool("dynasty_name_first");
    }
}

fn validate_name_list(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.verify_exists(Item::Localization, token);
    }
    for (_, block) in vd.integer_blocks() {
        let mut vd = Validator::new(block, data);
        for token in vd.values() {
            data.verify_exists(Item::Localization, token);
        }
    }
}

fn validate_mercenary_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_value_item("name", Item::Localization);
        vd.field_value("coat_of_arms");
    }
}

fn validate_dynasty_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        data.verify_exists(Item::Localization, token);
    }
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        for token in vd.values() {
            data.verify_exists(Item::Localization, token);
        }
    }
}
