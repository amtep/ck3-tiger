use std::path::{Path, PathBuf};

use fnv::{FnvHashMap, FnvHashSet};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{old_warn, warn2, ErrorKey};
use crate::token::Token;
use crate::util::SmartJoin;

#[derive(Clone, Debug, Default)]
pub struct Assets {
    assets: FnvHashMap<String, Asset>,
    attributes: FnvHashSet<String>,
    blend_shapes: FnvHashSet<String>,
    textures: FnvHashMap<String, FileEntry>,
}

impl Assets {
    pub fn load_item(&mut self, key: &Token, block: &Block) {
        if key.is("pdxmesh") {
            for (key, block) in block.iter_definitions() {
                if key.is("blend_shape") {
                    if let Some(id) = block.get_field_value("id") {
                        self.blend_shapes.insert(id.to_string());
                    }
                }
            }
        }
        if key.is("entity") {
            for (key, block) in block.iter_definitions() {
                if key.is("attribute") {
                    if let Some(name) = block.get_field_value("name") {
                        self.attributes.insert(name.to_string());
                    }
                }
            }
        }
        if let Some(name) = block.get_field_value("name") {
            if let Some(other) = self.assets.get(name.as_str()) {
                if other.key.loc.kind >= name.loc.kind {
                    dup_error(name, &other.key, "asset");
                }
            }
            self.assets.insert(name.to_string(), Asset::new(key.clone(), block.clone()));
        }
    }

    pub fn asset_exists(&self, key: &str) -> bool {
        self.assets.contains_key(key)
    }

    pub fn mesh_exists(&self, key: &str) -> bool {
        if let Some(asset) = self.assets.get(key) {
            asset.key.is("pdxmesh")
        } else {
            false
        }
    }

    pub fn entity_exists(&self, key: &str) -> bool {
        if let Some(asset) = self.assets.get(key) {
            asset.key.is("entity")
        } else {
            false
        }
    }

    pub fn blend_shape_exists(&self, key: &str) -> bool {
        self.blend_shapes.contains(key)
    }

    pub fn attribute_exists(&self, key: &str) -> bool {
        self.attributes.contains(key)
    }

    pub fn texture_exists(&self, key: &str) -> bool {
        self.textures.contains_key(key)
    }

    pub fn get_texture(&self, key: &str) -> Option<&FileEntry> {
        self.textures.get(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.assets.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Assets {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("gfx/models")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        let name = entry.filename().to_string_lossy();

        if name.ends_with(".dds") {
            if let Some(other) = self.textures.get(&name.to_string()) {
                if other.kind() >= entry.kind() {
                    warn2(
                        other,
                        ErrorKey::DuplicateItem,
                        "texture file is redefined by another file",
                        entry,
                        "the other file is here",
                    );
                }
            }
            self.textures.insert(name.to_string(), entry.clone());
            return;
        }

        if !name.ends_with(".asset") {
            return;
        }

        let Some(block) = PdxFile::read_optional_bom(entry, fullpath) else { return; };
        for (key, block) in block.iter_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Asset {
    key: Token,
    block: Block,
}

impl Asset {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate_mesh(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value("name");
        vd.req_field("file");
        if let Some(token) = vd.field_value("file") {
            let path = self.key.loc.pathname.smart_join_parent(token.as_str());
            data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
        }
        vd.field_numeric("scale");
        vd.field_numeric("cull_distance");

        vd.field_validated_blocks("lod_percentages", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_blocks("lod", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_field("index");
                vd.req_field("percent");
                vd.field_integer("index");
                vd.field_numeric("percent");
            });
        });

        vd.field_validated_blocks("meshsettings", validate_meshsettings);
        vd.field_validated_blocks("blend_shape", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname.smart_join_parent(token.as_str());
                data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
            }
            vd.field_value("data"); // TODO
        });

        vd.field_validated_blocks("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname.smart_join_parent(token.as_str());
                data.fileset.verify_exists_implied_crashes(&path.to_string_lossy(), token);
            }
        });
        vd.field_validated_blocks("additive_animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname.smart_join_parent(token.as_str());
                data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
            }
        });

        vd.field_validated_blocks("import", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("type"); // TODO
            vd.field_item("name", Item::Asset);
        });
    }

    pub fn validate_entity(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value("name");
        vd.field_item("pdxmesh", Item::Pdxmesh);
        vd.field_bool("get_state_from_parent");
        vd.field_numeric("scale");
        vd.field_numeric("cull_radius");
        vd.field_validated_blocks("attribute", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.req_field_one_of(&["blend_shape", "additive_animation"]);
            vd.field_item("name", Item::GeneAttribute);
            vd.field_item("additive_animation", Item::GeneAttribute);
            vd.field_item("blend_shape", Item::BlendShape);
            vd.field_numeric("default");
        });
        vd.field_validated_blocks("meshsettings", validate_meshsettings);
        vd.field_validated_blocks("game_data", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_blocks("portrait_entity_user_data", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_validated_blocks("portrait_accessory", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_item("pattern_mask", Item::File);
                    vd.field_item("variation", Item::AccessoryVariation);
                });
                vd.field_validated_blocks("color_mask_remap_interval", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_validated_blocks("interval", |block, data| {
                        let mut vd = Validator::new(block, data);
                        vd.req_tokens_numbers_exactly(2);
                    });
                });
                vd.field_validated_blocks("portrait_decal", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_value("body_part"); // TODO
                });
                vd.field_item("coa_mask", Item::File);
            });
            vd.field_validated_blocks("throne_entity_user_data", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("animation", Item::PortraitAnimation);
                vd.field_bool("use_throne_transform");
            });
            vd.field_validated_blocks("court_entity_user_data", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_bool("coat_of_arms");
            });
        });
        vd.field_validated_blocks("state", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.field_value("name");
            vd.field_numeric("state_time");
            vd.field_bool("looping");
            vd.field_numeric("animation_speed");
            vd.field_value("next_state"); // TODO
            vd.field("chance"); // TODO: can be integer or block
            vd.field_value("animation"); // TODO
            vd.field_numeric("animation_blend_time");
            vd.field_numeric("time_offset");
            vd.field_validated_blocks("start_event", validate_event);
            vd.field_validated_blocks("event", validate_event);
            vd.field_validated_bvs("propagate_state", |bv, data| {
                match bv {
                    BV::Value(_token) => (), // TODO
                    BV::Block(block) => {
                        let mut vd = Validator::new(block, data);
                        // TODO
                        for (_key, bv) in vd.unknown_fields() {
                            bv.expect_value();
                        }
                    }
                }
            });
        });
        vd.field_value("default_state"); // TODO: must be a state name
        vd.field_validated_blocks("locator", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.field_value("name");
            vd.field_validated_blocks("position", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(3);
            });
            vd.field_validated_blocks("rotation", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(3);
            });
            vd.field_numeric("scale");
        });
        vd.field_validated_blocks("attach", |block, data| {
            let mut vd = Validator::new(block, data);
            for (_key, token) in vd.unknown_value_fields() {
                // TODO: what are the keys here?
                data.verify_exists(Item::Asset, token);
            }
        });
    }

    pub fn validate_animation_set(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value("name");
        vd.req_field("reference_skeleton");
        vd.field_items("reference_skeleton", Item::Pdxmesh);
        vd.field_validated_blocks("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname.smart_join_parent(token.as_str());
                data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
            }
        });
    }

    pub fn validate(&self, data: &Everything) {
        if self.key.is("pdxmesh") {
            self.validate_mesh(data);
        } else if self.key.is("entity") {
            self.validate_entity(data);
        } else if self.key.is("skeletal_animation_set") {
            self.validate_animation_set(data);
        } else if self.key.is("arrowType") {
            // TODO: arrowType
        } else {
            old_warn(&self.key, ErrorKey::UnknownField, "unknown asset type");
        }
    }
}

fn validate_event(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_numeric("time");
    vd.field_numeric("life");
    vd.field_numeric("entity_fade_speed");
    vd.field_value("state"); // TODO
    vd.field_value("node"); // TODO
    vd.field_value("particle"); // TODO
    vd.field_bool("keep_particle");
    vd.field_bool("keep_sound");
    vd.field_bool("keep_entity");
    vd.field_bool("trigger_once");
    vd.field_bool("use_parent_nodes");
    vd.field_integer("skip_forward");
    vd.field_value("attachment_id"); // TODO
    vd.field_value("remove_attachment"); // TODO
    vd.field_item("entity", Item::Entity);
    vd.field_validated_blocks("soundparameter", |block, data| {
        let mut vd = Validator::new(block, data);
        for (_key, token) in vd.unknown_value_fields() {
            // TODO: what are the keys here?
            token.expect_number();
        }
    });
    vd.field_validated_blocks("sound", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("soundeffect", Item::Sound);
        vd.field_bool("stop_on_state_change");
    });
    vd.field_value("light"); // TODO
}

fn validate_meshsettings(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_value("name");
    vd.field_integer("index"); // TODO: do these need to be consecutive?
    vd.field_item("texture_diffuse", Item::TextureFile);
    vd.field_item("texture_normal", Item::TextureFile);
    vd.field_item("texture_specular", Item::TextureFile);
    vd.field_validated_block("texture", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("file");
        vd.req_field("index");
        vd.field_item("file", Item::TextureFile);
        vd.field_integer("index");
        vd.field_bool("srgb");
    });
    vd.field_value("shader"); // TODO
    if let Some(token) = vd.field_value("shader_file") {
        // Filter out builtin shaders
        if !token.starts_with("gfx/FX/jomini/") {
            data.verify_exists(Item::File, token);
        }
    }
    vd.field_value("subpass");
    vd.field_value("shadow_shader");
}
