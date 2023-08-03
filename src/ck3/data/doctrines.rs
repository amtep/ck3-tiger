use std::path::{Path, PathBuf};

use fnv::{FnvHashMap, FnvHashSet};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_traits;

#[derive(Clone, Debug, Default)]
pub struct Doctrines {
    groups: FnvHashMap<String, DoctrineGroup>,
    doctrines: FnvHashMap<String, Doctrine>,
    parameters: FnvHashSet<String>, // only the boolean parameters
}

impl Doctrines {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.groups.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "doctrine group");
            }
        }

        for (doctrine, block) in block.iter_definitions() {
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
                        self.parameters.insert(k.to_string());
                    }
                }
            }
            self.doctrines.insert(
                doctrine.to_string(),
                Doctrine::new(doctrine.clone(), block.clone(), key.clone()),
            );
        }

        self.groups.insert(key.to_string(), DoctrineGroup::new(key, block));
    }

    pub fn validate(&self, data: &Everything) {
        for group in self.groups.values() {
            group.validate(data);
        }
        for doctrine in self.doctrines.values() {
            doctrine.validate(data);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.doctrines.contains_key(key)
    }

    pub fn parameter_exists(&self, key: &str) -> bool {
        self.parameters.contains(key)
    }

    pub fn unreformed(&self, key: &str) -> bool {
        self.doctrines.get(key).map_or(false, Doctrine::unreformed)
    }
}

impl FileHandler<Block> for Doctrines {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/religion/doctrines")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, fullpath)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct DoctrineGroup {
    key: Token,
    block: Block,
}

impl DoctrineGroup {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
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
    group: Token,
}

impl Doctrine {
    pub fn new(key: Token, block: Block, group: Token) -> Self {
        Self { key, block, group }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Faith, &self.key);
        sc.define_list("selected_doctrines", Scopes::Doctrine, &self.key);

        let icon_path =
            data.get_defined_string_warn(&self.key, "NGameIcons|FAITH_DOCTRINE_ICON_PATH");
        if let Some(icon_path) = icon_path {
            if let Some(icon) = vd.field_value("icon") {
                let path = format!("{icon_path}/{icon}.dds");
                data.verify_exists_implied(Item::File, &path, icon);
            } else if data.doctrines.groups[self.group.as_str()].needs_icon(data) {
                let path = format!("{icon_path}/{}.dds", &self.key);
                data.verify_exists_implied(Item::File, &path, &self.key);
            }
        } else {
            vd.field_value("icon");
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
