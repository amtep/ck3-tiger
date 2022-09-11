use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug, Default)]
pub struct Terrains {
    terrains: FnvHashMap<String, Terrain>,
}

impl Terrains {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.terrains.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "terrain");
            }
        }
        self.terrains
            .insert(key.to_string(), Terrain::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.terrains.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.terrains.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Terrains {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/terrain_types")
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
pub struct Terrain {
    key: Token,
    block: Block,
}

impl Terrain {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::None, self.key.clone());
        vd.req_field("color");

        vd.field_numeric("movement_speed");
        vd.field_validated_block("color", validate_color);

        vd.field_validated_block("attacker_modifier", |b, data| {
            validate_combat_modifier(b, data, &mut sc)
        });
        vd.field_validated_block("defender_modifier", |b, data| {
            validate_combat_modifier(b, data, &mut sc)
        });
        vd.field_block("attacker_combat_effects"); // TODO
        vd.field_block("defender_combat_effects"); // TODO

        vd.field_numeric("combat_width");
        vd.field_bool("is_desert");
        vd.field_bool("is_jungle");
        vd.field_numeric("audio_parameter"); // ??

        vd.field_validated_block("province_modifier", |b, data| {
            validate_province_modifier(b, data, &mut sc)
        });

        vd.warn_remaining();
    }
}

pub fn validate_combat_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let vd = Validator::new(block, data);
    validate_modifs(block, data, ModifKinds::Terrain, sc, vd);
}

pub fn validate_province_modifier(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let vd = Validator::new(block, data);
    validate_modifs(block, data, ModifKinds::Province, sc, vd);
}
