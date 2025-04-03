use std::path::PathBuf;

use crate::block::{Block, BV};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
#[cfg(feature = "ck3")]
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
#[cfg(feature = "ck3")]
use crate::report::Severity;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Defines {
    defines: TigerHashMap<String, Define>,
}

impl Defines {
    pub fn load_item(&mut self, group: Token, name: Token, bv: &BV) {
        let key = format!("{}|{}", &group, &name);
        if let Some(other) = self.defines.get(&key) {
            if other.name.loc.kind >= name.loc.kind && !bv.equivalent(&other.bv) {
                dup_error(&name, &other.name, "define");
            }
        }
        self.defines.insert(key, Define::new(group, name, bv.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.defines.contains_key(key)
    }

    // TODO: figure out some way to represent the group as well
    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.defines.values().map(|item| &item.name)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.defines.values() {
            item.validate(data);
        }
    }

    #[cfg(any(feature = "ck3", feature = "vic3"))]
    pub fn get_bv(&self, key: &str) -> Option<&BV> {
        self.defines.get(key).map(|d| &d.bv)
    }
}

impl FileHandler<Block> for Defines {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/defines")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        // TODO HOI4: Hoi4 has a toplevel group
        for (group, block) in block.drain_definitions_warn() {
            for (name, bv) in block.iter_assignments_and_definitions_warn() {
                self.load_item(group.clone(), name.clone(), bv);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Define {
    #[allow(dead_code)] // TODO
    group: Token,
    name: Token,
    bv: BV,
}

impl Define {
    pub fn new(group: Token, name: Token, bv: BV) -> Self {
        Self { group, name, bv }
    }

    #[allow(clippy::unused_self)]
    #[allow(unused_variables)] // because only ck3 uses `data`
    pub fn validate(&self, data: &Everything) {
        // TODO: validate that each define is the right 'type',
        // such as a path, a number, or a block of numeric values

        #[cfg(feature = "ck3")]
        if self.group.is("NGameIcons") && self.name.is("PIETY_GROUPS") {
            if let Some(icon_path) =
                data.get_defined_string_warn(&self.name, "NGameIcons|PIETY_LEVEL_PATH")
            {
                if let Some(groups) = self.bv.expect_block() {
                    for icon_group in groups.iter_values_warn() {
                        for nr in &["00", "01", "02", "03", "04", "05"] {
                            let pathname = format!("{icon_path}/icon_piety_{icon_group}_{nr}.dds");
                            data.verify_exists_implied_max_sev(
                                Item::File,
                                &pathname,
                                icon_group,
                                Severity::Warning,
                            );
                        }
                    }
                }
            }
        }
    }
}
