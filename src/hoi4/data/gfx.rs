//! Process .gfx files, which contain sprite and mesh definitions.

use std::path::PathBuf;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct Gfx {
    meshes: TigerHashMap<&'static str, Mesh>,
    sprites: TigerHashMap<&'static str, Sprite>,
}

impl Gfx {
    pub fn load_sprite(&mut self, key: Token, block: Block) {
        if let Some(name) = block.get_field_value("name") {
            if let Some(other) = self.sprites.get(name.as_str()) {
                if other.key.loc.kind >= name.loc.kind {
                    dup_error(name, &other.key, "sprite");
                }
            }
            self.sprites
                .insert(name.as_str(), Sprite::new(key.clone(), name.clone(), block.clone()));
        }
    }

    pub fn load_mesh(&mut self, key: Token, block: Block) {
        if let Some(name) = block.get_field_value("name") {
            if let Some(other) = self.meshes.get(name.as_str()) {
                if other.key.loc.kind >= name.loc.kind {
                    dup_error(name, &other.key, "pdxmesh");
                }
            }
            self.meshes.insert(name.as_str(), Mesh::new(key.clone(), name.clone(), block.clone()));
        }
    }

    pub fn mesh_exists(&self, key: &str) -> bool {
        self.meshes.contains_key(key)
    }

    pub fn iter_mesh_keys(&self) -> impl Iterator<Item = &Token> {
        self.meshes.values().map(|item| &item.name)
    }

    pub fn sprite_exists(&self, key: &str) -> bool {
        self.sprites.contains_key(key)
    }

    pub fn iter_sprite_keys(&self) -> impl Iterator<Item = &Token> {
        self.sprites.values().map(|item| &item.name)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.meshes.values() {
            item.validate(data);
        }
        for item in self.sprites.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Gfx {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        let name = entry.filename().to_string_lossy();

        if name.ends_with(".gfx") {
            PdxFile::read_optional_bom(entry, parser)
        } else {
            None
        }
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, mut block) in block.drain_definitions_warn() {
            if key.lowercase_is("spritetypes") {
                for (key, block) in block.drain_definitions_warn() {
                    if key.lowercase_is("spritetype") || key.lowercase_is("corneredtilespritetype")
                    {
                        self.load_sprite(key, block);
                    } else {
                        let msg = format!("unknown key {key}");
                        err(ErrorKey::UnknownField).msg(msg).loc(key).push();
                    }
                }
            } else if key.lowercase_is("objecttypes") {
                for (key, block) in block.drain_definitions_warn() {
                    if key.lowercase_is("pdxmesh") {
                        self.load_mesh(key, block);
                    } else {
                        let msg = format!("unknown key {key}");
                        err(ErrorKey::UnknownField).msg(msg).loc(key).push();
                    }
                }
            } else {
                let msg = format!("unknown key {key}");
                err(ErrorKey::UnknownField).msg(msg).loc(key).push();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sprite {
    key: Token,
    name: Token,
    block: Block,
}

impl Sprite {
    pub fn new(key: Token, name: Token, block: Block) -> Self {
        Self { key, name, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.set_case_sensitive(false);

        vd.field_value("name");
        vd.field_item("texturefile", Item::File);
        vd.field_bool("legacy_lazy_load");

        if self.key.lowercase_is("corneredtilespritetype") {
            vd.field_validated_block("bordersize", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.set_case_sensitive(false);

                vd.field_integer("x");
                vd.field_integer("y");
            });
            vd.field_item("effectfile", Item::File);
        }
    }
}

fn validate_meshsettings(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_value("name");
    vd.field_integer("index"); // TODO: do these need to be consecutive?
    vd.field_item("texture_diffuse", Item::File);
    vd.field_item("texture_normal", Item::File);
    vd.field_item("texture_specular", Item::File);
    vd.field_value("shader"); // TODO
}

#[derive(Clone, Debug)]
pub struct Mesh {
    key: Token,
    name: Token,
    block: Block,
}

impl Mesh {
    pub fn new(key: Token, name: Token, block: Block) -> Self {
        Self { key, name, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value("name");
        vd.req_field("file");
        vd.field_item("file", Item::File);
        vd.field_numeric("scale");

        vd.multi_field_validated_block("meshsettings", validate_meshsettings);

        vd.multi_field_validated_block("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            vd.field_value("type"); // TODO
        });

        vd.multi_field_validated_block("variant", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("pdxmesh");
            vd.field_numeric("weight");
            vd.field_item("pdxmesh", Item::Pdxmesh);
        });
    }
}
