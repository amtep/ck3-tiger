use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug, Default)]
pub struct Lifestyles {
    lifestyles: FnvHashMap<String, Lifestyle>,
}

impl Lifestyles {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.lifestyles.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "lifestyle");
            }
        }

        self.lifestyles
            .insert(key.to_string(), Lifestyle::new(key.clone(), block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.lifestyles.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        let mut vec = self.lifestyles.values().collect::<Vec<&Lifestyle>>();
        vec.sort_unstable_by_key(|item| &item.key.loc);
        for item in vec {
            item.validate(data);
        }
    }
}

impl FileHandler for Lifestyles {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/lifestyles")
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
pub struct Lifestyle {
    key: Token,
    block: Block,
}

impl Lifestyle {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let loca = format!("{}_name", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("{}_desc", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);
        let loca = format!("{}_highlight_desc", self.key);
        data.localization.verify_exists_implied(&loca, &self.key);

        let mut vd = Validator::new(&self.block, data);

        if let Some(block) = vd.field_block("is_highlighted") {
            validate_normal_trigger(block, data, Scopes::Character, false);
        }

        if let Some(block) = vd.field_block("is_valid") {
            validate_normal_trigger(block, data, Scopes::Character, true);
        }

        if let Some(block) = vd.field_block("is_valid_showing_failures_only") {
            validate_normal_trigger(block, data, Scopes::Character, true);
        }

        if let Some(token) = vd.field_value("icon") {
            let pathname = format!("gfx/interface/icons/lifestyles/{}.dds", token);
            data.fileset.verify_exists_implied(&pathname, token);
        } else {
            let pathname = format!("gfx/interface/icons/lifestyles/{}.dds", self.key);
            data.fileset.verify_exists_implied(&pathname, &self.key);
        }

        vd.field_numeric("xp_per_level");
        vd.field_numeric("base_xp_gain");

        vd.warn_remaining();
    }
}
