use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::warn_info;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_optional_duration_int;

#[derive(Clone, Debug, Default)]
pub struct Musics {
    musics: FnvHashMap<String, Music>,
}

impl Musics {
    pub fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.musics.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(key, &other.key, "music");
            }
        }
        self.musics.insert(key.to_string(), Music::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.musics.contains_key(key)
    }

    pub fn verify_exists_implied(&self, key: &str, item: &Token) {
        if !self.musics.contains_key(key) {
            let msg = if key == item.as_str() {
                "music not defined in music/ or dlc/*/music/".to_string()
            } else {
                format!("music {key} not defined in music/ or dlc/*/music/")
            };
            let info = "this could be due to a missing DLC";
            warn_info(item, ErrorKey::MissingSound, &msg, info);
        }
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.musics.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Musics {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !(entry.path().starts_with("dlc") && entry.path().parent().unwrap().ends_with("music"))
            && !entry.path().starts_with("music")
        {
            return;
        }
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Music {
    key: Token,
    block: Block,
}

impl Music {
    pub fn new(key: &Token, block: &Block) -> Self {
        Music {
            key: key.clone(),
            block: block.clone(),
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());

        vd.field_item("music", Item::Sound);
        vd.field_item("group", Item::Music); // Take settings from this item
        vd.field_integer("pause_factor");

        vd.field_bool("mood");
        vd.field_bool("is_prioritized_mood");
        vd.field_bool("can_be_interrupted");

        validate_optional_duration_int(&mut vd);
        vd.field_integer("calls");

        vd.field_bool("trigger_prio_override");
        vd.field_validated_block("is_valid", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_list_numeric_exactly("subsequent_playback_chance", 3);
    }
}
