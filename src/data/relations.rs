use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Relations {
    relations: FnvHashMap<String, Relation>,
}

impl Relations {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.relations.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "relation");
            }
        }
        self.relations
            .insert(key.to_string(), Relation::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.relations.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.relations.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Relations {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_relations")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_item(key.clone(), block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Relation {
    key: Token,
    block: Block,
}

impl Relation {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value_item("corresponding", Item::Relation);
        vd.field_bool("title_grant_target");
        vd.field_list("opposites");
        if let Some(list) = self.block.get_field_list("opposites") {
            for token in list {
                data.verify_exists(Item::Relation, &token);
            }
        }
        vd.field_list("relation_aliases");
        if let Some(list) = self.block.get_field_list("relation_aliases") {
            for token in list {
                data.verify_exists(Item::Relation, &token);
            }
        }
        vd.field_integer("opinion");
        vd.field_numeric("fertility");
        for (key, _) in vd.integer_values() {
            let val = key.as_str().parse::<i32>().unwrap();
            if !(0..=15).contains(&val) {
                error(key, ErrorKey::Validation, "flag value out of range");
            }
        }
        vd.field_value("secret");
        vd.field_bool("special_guest");
        vd.field_bool("hidden");
    }
}
