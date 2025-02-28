use std::path::PathBuf;

use crate::block::Block;
use crate::ck3::validate::validate_traits;
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{TigerHashMap, TigerHashSet, dup_error};
use crate::item::Item;
use crate::modif::{ModifKinds, validate_modifs};
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct Doctrines {
    categories: TigerHashMap<&'static str, DoctrineCategory>,
    doctrines: TigerHashMap<&'static str, Doctrine>,
    parameters: TigerHashSet<Token>, // only the boolean parameters
}

impl Doctrines {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.categories.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "doctrine category");
            }
        }
        self.categories.insert(key.as_str(), DoctrineCategory::new(key, block));
    }

    pub fn validate(&self, data: &Everything) {
        for category in self.categories.values() {
            category.validate(data);
        }
        for doctrine in self.doctrines.values() {
            doctrine.validate(data);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.doctrines.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.doctrines.values().map(|item| &item.key)
    }

    pub fn category_exists(&self, key: &str) -> bool {
        self.categories.contains_key(key)
    }

    pub fn category(&self, key: &str) -> Option<&Token> {
        self.doctrines.get(key).map(|d| &d.category)
    }

    pub fn number_of_picks(&self, category: &str) -> Option<&Token> {
        self.categories.get(category).and_then(|c| c.picks.as_ref())
    }

    pub fn iter_category_keys(&self) -> impl Iterator<Item = &Token> {
        self.categories.values().map(|item| &item.key)
    }

    pub fn parameter_exists(&self, key: &str) -> bool {
        self.parameters.contains(key)
    }

    pub fn iter_parameter_keys(&self) -> impl Iterator<Item = &Token> {
        self.parameters.iter()
    }

    pub fn unreformed(&self, key: &str) -> bool {
        self.doctrines.get(key).is_some_and(Doctrine::unreformed)
    }
}

impl FileHandler<Block> for Doctrines {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/religion/doctrines")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }

    fn finalize(&mut self) {
        for category in self.categories.values() {
            for (doctrine, block) in category.block.iter_definitions() {
                // skip definitions that are not doctrines
                if doctrine.is("is_available_on_create") || doctrine.is("name") {
                    continue;
                }

                if let Some(other) = self.doctrines.get(doctrine.as_str()) {
                    if other.key.loc.kind >= doctrine.loc.kind {
                        dup_error(doctrine, &other.key, "doctrine");
                    }
                }

                if let Some(b) = block.get_field_block("parameters") {
                    for (k, v) in b.iter_assignments() {
                        if v.is("yes") || v.is("no") {
                            self.parameters.insert(k.clone());
                        }
                    }
                }
                self.doctrines.insert(
                    doctrine.as_str(),
                    Doctrine::new(doctrine.clone(), block.clone(), category.key.clone()),
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DoctrineCategory {
    key: Token,
    block: Block,
    picks: Option<Token>,
}

impl DoctrineCategory {
    pub fn new(key: Token, block: Block) -> Self {
        let picks = block.get_field_value("number_of_picks").cloned();
        Self { key, block, picks }
    }

    pub fn needs_icon(&self, data: &Everything) -> bool {
        if let Some(group) = self.block.get_field_value("group") {
            if group.is("special") || group.is("not_creatable") {
                return false;
            }
        }

        if let Some(icon_path) =
            data.get_defined_string_warn(&self.key, "NGameIcons|FAITH_DOCTRINE_GROUP_ICON_PATH")
        {
            let path = format!("{icon_path}/{}.dds", &self.key);
            data.mark_used(Item::File, &path);
            return !data.fileset.exists(&path);
        }
        true
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Faith, &self.key);

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("{}_name", self.key);
            data.verify_exists_implied(Item::Localization, &loca, &self.key);
        }

        // doc says "grouping" but that's wrong
        vd.field_value("group");

        vd.field_integer("number_of_picks");

        vd.field_validated_block("is_available_on_create", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });

        // Any remaining definitions are doctrines, so accept them all.
        // They are validated separately.
        vd.unknown_block_fields(|_, _| ());
    }
}

#[derive(Clone, Debug)]
pub struct Doctrine {
    key: Token,
    block: Block,
    category: Token,
}

impl Doctrine {
    pub fn new(key: Token, block: Block, category: Token) -> Self {
        Self { key, block, category }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Faith, &self.key);
        sc.define_list("selected_doctrines", Scopes::Doctrine, &self.key);

        if let Some(icon) = vd.field_value("icon") {
            data.verify_icon("NGameIcons|FAITH_DOCTRINE_ICON_PATH", icon, ".dds");
        } else if data.doctrines.categories[self.category.as_str()].needs_icon(data) {
            data.verify_icon("NGameIcons|FAITH_DOCTRINE_ICON_PATH", &self.key, ".dds");
        }

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("{}_name", self.key);
            data.verify_exists_implied(Item::Localization, &loca, &self.key);
        }

        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{}_desc", self.key);
            data.verify_exists_implied(Item::Localization, &loca, &self.key);
        }

        vd.field_bool("visible");
        vd.field_validated_block("parameters", validate_parameters);
        vd.field_script_value("piety_cost", &mut sc);
        vd.field_validated_block("is_shown", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("can_pick", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_validated_block("character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        // Not documented, but used in vanilla
        vd.field_validated_block("clergy_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        // In the docs but not used in vanilla
        vd.field_validated_block("doctrine_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("doctrine", Item::Doctrine);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_validated_block("traits", validate_traits);
    }

    fn unreformed(&self) -> bool {
        if let Some(block) = self.block.get_field_block("parameters") {
            return block.field_value_is("unreformed", "yes");
        }
        false
    }
}

fn validate_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.unknown_value_fields(|_, value| {
        if value.is("yes") || value.is("no") {
            return;
        }
        value.expect_number();
    });
}
