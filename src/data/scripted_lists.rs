use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::scopes::scope_iterator;
use crate::token::Token;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug, Default)]
pub struct ScriptedLists {
    lists: FnvHashMap<String, List>,
}

impl ScriptedLists {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.lists.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "scripted list");
            }
        }
        self.lists
            .insert(key.to_string(), List::new(key.clone(), block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.lists.contains_key(key)
    }

    pub fn base(&self, item: &Token) -> Option<&Token> {
        self.lists
            .get(item.as_str())
            .and_then(|item| item.block.get_field_value("base"))
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.lists.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for ScriptedLists {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_lists")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let block = match PdxFile::read(entry, fullpath) {
            Some(block) => block,
            None => return,
        };

        for (key, b) in block.iter_pure_definitions_warn() {
            self.load_item(key, b);
        }
    }
}

#[derive(Clone, Debug)]
pub struct List {
    pub key: Token,
    block: Block,
}

impl List {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("base");
        vd.req_field("conditions");

        if let Some(token) = vd.field_value("base") {
            if let Some((_, outscope)) = scope_iterator(token, data) {
                if let Some(block) = vd.field_block("conditions") {
                    validate_normal_trigger(block, data, outscope, true);
                }
            } else {
                error(token, ErrorKey::MissingItem, "no such base list");
            }
        }

        vd.warn_remaining();
    }
}
