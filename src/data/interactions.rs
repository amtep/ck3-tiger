use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Interactions {
    interactions: FnvHashMap<String, Interaction>,
}

impl Interactions {
    pub fn load_interaction(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.interactions.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "interaction");
            }
        }
        self.interactions
            .insert(key.to_string(), Interaction::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.interactions.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.interactions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Interactions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/character_interactions")
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
            self.load_interaction(key.clone(), block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Interaction {
    key: Token,
    block: Block,
}

impl Interaction {
    pub fn new(key: Token, block: Block) -> Self {
        Interaction { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        // TODO: actually validate the fields

        if let Some(name) = self.block.get_field_value("icon") {
            let pathname = format!("gfx/interface/icons/character_interactions/{}.dds", name);
            data.fileset.verify_exists_implied(&pathname, name);
        }

        // TODO: The ai_ name check is a heuristic. It would be better to check if the
        // is_shown trigger requires scope:actor to be is_ai = yes. But that's a long way off.
        if !self.key.as_str().starts_with("ai_") {
            data.localization.verify_exists(&self.key);
        }
        if self.block.get_field_value("extra_icon").is_some()
            && self
                .block
                .get_field_block("should_use_extra_icon")
                .is_some()
        {
            data.localization.verify_exists_implied(
                &format!("{}_extra_icon", self.key),
                self.block.get_key("extra_icon").unwrap(),
            );
        }
        if let Some(pathname) = self.block.get_field_value("extra_icon") {
            data.fileset.verify_exists(pathname);
        }
        if let Some(desc) = self.block.get_field("desc") {
            validate_desc(desc, data);
        }
        if let Some(desc) = self.block.get_field("prompt") {
            validate_desc(desc, data);
        }
        if let Some(desc) = self.block.get_field("notification_text") {
            validate_desc(desc, data);
        }
        if let Some(desc) = self.block.get_field("on_decline_summary") {
            validate_desc(desc, data);
        }
        if let Some(key) = self.block.get_field_value("answer_accept_key") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("answer_reject_key") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("highlighted_reason") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("options_heading") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("pre_answer_maybe_breakdown_key") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("pre_answer_maybe_key") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("pre_answer_no_key") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("pre_answer_yes_key") {
            data.localization.verify_exists(key);
        }
        if let Some(key) = self.block.get_field_value("send_name") {
            data.localization.verify_exists(key);
        }
        for b in self.block.get_field_blocks("send_option") {
            if let Some(key) = b.get_field_value("localization") {
                data.localization.verify_exists(key);
            }
        }
    }
}
