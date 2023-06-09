use fnv::FnvHashMap;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug, Default)]
pub struct Themes {
    themes: FnvHashMap<String, Theme>,
}

impl Themes {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.themes.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "event theme");
            }
        }
        self.themes.insert(key.to_string(), Theme::new(key, block));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.themes.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&Theme> {
        self.themes.get(key)
    }

    pub fn validate(&self, data: &Everything, sc: &mut ScopeContext) {
        for item in self.themes.values() {
            item.validate(data, sc);
        }
    }
}

impl FileHandler for Themes {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/event_themes")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_item(key.clone(), block.clone());
        }
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    key: Token,
    block: Block,
    validated_scopes: RefCell<Scopes>,
}

impl Theme {
    pub fn new(key: Token, block: Block) -> Self {
        let validated_scopes = RefCell::new(Scopes::empty());
        Self {
            key,
            block,
            validated_scopes,
        }
    }

    // Themes are unusual in that they are validated through the events that use them.
    // This means that unused themes are not validated, which is ok.
    // The purpose is to allow the triggers to be validated in the context of the scope
    // of the event that uses them.
    pub fn validate(&self, data: &Everything, sc: &mut ScopeContext) {
        // Check if the passed-in scope type has already been validated for
        if self.validated_scopes.borrow().contains(sc.scopes()) {
            return;
        }
        *self.validated_scopes.borrow_mut() |= sc.scopes();

        let mut vd = Validator::new(&self.block, data);

        vd.req_field("background");
        vd.req_field("icon");
        vd.req_field("sound");

        vd.field_validated_blocks("background", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            vd.field_item("reference", Item::EventBackground);
        });

        vd.field_validated_blocks("icon", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            vd.field_item("reference", Item::File);
        });

        vd.field_validated_blocks("sound", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            // TODO: figure out a way to get a list of all available sounds
            vd.field_value("reference");
        });
        // `transition` is not documented but presumably works the same way
        vd.field_validated_blocks("transition", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("trigger", |b, data| {
                validate_normal_trigger(b, data, sc, Tooltipped::No);
            });
            vd.field_item("reference", Item::EventTransition);
        });
    }
}
