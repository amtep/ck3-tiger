use std::path::PathBuf;

use fnv::{FnvHashMap, FnvHashSet};

use crate::block::{Block, BV};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::game::Game;
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{warn, Confidence, ErrorKey, Severity};
use crate::token::Token;
use crate::util::SmartJoin;
use crate::validate::validate_numeric_range;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct Assets {
    assets: FnvHashMap<&'static str, Asset>,
    attributes: FnvHashSet<Token>,
    blend_shapes: FnvHashSet<Token>,
    textures: FnvHashMap<String, (FileEntry, Token)>,
}

impl Assets {
    pub fn load_item(&mut self, key: &Token, block: &Block) {
        if key.is("pdxmesh") {
            for (key, block) in block.iter_definitions() {
                if key.is("blend_shape") {
                    if let Some(id) = block.get_field_value("id") {
                        self.blend_shapes.insert(id.clone());
                    }
                }
            }
        } else if key.is("entity") {
            for (key, block) in block.iter_definitions() {
                if key.is("attribute") {
                    if let Some(name) = block.get_field_value("name") {
                        self.attributes.insert(name.clone());
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
            self.assets.insert(name.as_str(), Asset::new(key.clone(), name.clone(), block.clone()));
        }
    }

    pub fn asset_exists(&self, key: &str) -> bool {
        self.assets.contains_key(key)
    }

    pub fn iter_asset_keys(&self) -> impl Iterator<Item = &Token> {
        self.assets.values().map(|item| &item.name)
    }

    pub fn mesh_exists(&self, key: &str) -> bool {
        if let Some(asset) = self.assets.get(key) {
            asset.key.is("pdxmesh")
        } else {
            false
        }
    }

    pub fn iter_mesh_keys(&self) -> impl Iterator<Item = &Token> {
        self.assets.values().filter(|item| item.key.is("pdxmesh")).map(|item| &item.name)
    }

    pub fn entity_exists(&self, key: &str) -> bool {
        if let Some(asset) = self.assets.get(key) {
            asset.key.is("entity")
        } else {
            false
        }
    }

    pub fn iter_entity_keys(&self) -> impl Iterator<Item = &Token> {
        self.assets.values().filter(|item| item.key.is("entity")).map(|item| &item.name)
    }

    pub fn blend_shape_exists(&self, key: &str) -> bool {
        self.blend_shapes.contains(key)
    }

    pub fn iter_blend_shape_keys(&self) -> impl Iterator<Item = &Token> {
        self.blend_shapes.iter()
    }

    pub fn attribute_exists(&self, key: &str) -> bool {
        self.attributes.contains(key)
    }

    pub fn iter_attribute_keys(&self) -> impl Iterator<Item = &Token> {
        self.attributes.iter()
    }

    pub fn texture_exists(&self, key: &str) -> bool {
        self.textures.contains_key(key)
    }

    pub fn iter_texture_keys(&self) -> impl Iterator<Item = &Token> {
        self.textures.values().map(|(_, token)| token)
    }

    pub fn get_texture(&self, key: &str) -> Option<&FileEntry> {
        self.textures.get(key).map(|(entry, _)| entry)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.assets.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Option<Block>> for Assets {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("gfx/models")
    }

    /// TODO: should probably simplify this `FileHandler` by keeping the textures in a separate `FileHandler`.
    fn load_file(&self, entry: &FileEntry) -> Option<Option<Block>> {
        let name = entry.filename().to_string_lossy();

        if name.ends_with(".dds") {
            Some(None)
        } else if name.ends_with(".asset") {
            PdxFile::read_optional_bom(entry).map(Some)
        } else {
            None
        }
    }

    fn handle_file(&mut self, entry: &FileEntry, loaded: Option<Block>) {
        let name = entry.filename().to_string_lossy();
        if name.ends_with(".dds") {
            if let Some((other, _)) = self.textures.get(&*name) {
                if other.kind() >= entry.kind() {
                    warn(ErrorKey::DuplicateItem)
                        .msg("texture file is redefined by another file")
                        .loc(other)
                        .loc_msg(entry, "the other file is here")
                        .push();
                }
            }
            let entry_token = Token::new(&entry.filename().to_string_lossy(), entry.into());
            self.textures.insert(name.to_string(), (entry.clone(), entry_token));
            return;
        }

        let block = loaded.expect("internal error");
        for (key, block) in block.iter_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Asset {
    key: Token,
    name: Token,
    block: Block,
}

impl Asset {
    pub fn new(key: Token, name: Token, block: Block) -> Self {
        Self { key, name, block }
    }

    pub fn validate_mesh(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value("name");
        vd.req_field("file");
        if let Some(token) = vd.field_value("file") {
            let path = self.key.loc.pathname().smart_join_parent(token.as_str());
            data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
        }
        vd.field_numeric("scale");
        vd.field_numeric("cull_distance");

        vd.multi_field_validated_block("lod_percentages", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_validated_block("lod", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_field("index");
                vd.req_field("percent");
                vd.field_integer("index");
                vd.field_numeric("percent");
            });
        });

        vd.multi_field_validated_block("meshsettings", validate_meshsettings);
        vd.multi_field_validated_block("blend_shape", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname().smart_join_parent(token.as_str());
                data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
            }
            vd.field_value("data"); // TODO
        });

        vd.multi_field_validated_block("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname().smart_join_parent(token.as_str());
                data.fileset.verify_exists_implied_crashes(&path.to_string_lossy(), token);
            }
        });
        vd.multi_field_validated_block("additive_animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname().smart_join_parent(token.as_str());
                data.verify_exists_implied(Item::File, &path.to_string_lossy(), token);
            }
        });

        vd.multi_field_validated_block("import", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("type"); // TODO
            vd.field_item("name", Item::Asset);
        });
    }

    pub fn validate_entity(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.set_case_sensitive(false);

        vd.field_value("name");
        vd.field_item("pdxmesh", Item::Pdxmesh);
        vd.field_item("clone", Item::Entity);
        vd.field_bool("get_state_from_parent");
        vd.field_numeric("scale");
        vd.field_numeric("cull_radius");
        vd.multi_field_validated_block("attribute", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.req_field_one_of(&["blend_shape", "additive_animation"]);
            vd.field_item("name", Item::GeneAttribute);
            vd.field_item("additive_animation", Item::GeneAttribute);
            vd.field_item("blend_shape", Item::BlendShape);
            vd.field_numeric("default");
        });
        vd.multi_field_validated_block("meshsettings", validate_meshsettings);
        vd.multi_field_validated_block("game_data", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_validated_block("portrait_entity_user_data", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.multi_field_validated_block("portrait_accessory", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_item("pattern_mask", Item::File);
                    vd.field_item("variation", Item::AccessoryVariation);
                });
                vd.multi_field_validated_block("color_mask_remap_interval", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.multi_field_validated_block("interval", |block, data| {
                        validate_numeric_range(
                            block,
                            data,
                            0.0,
                            1.0,
                            Severity::Warning,
                            Confidence::Weak,
                        );
                    });
                });
                vd.multi_field_validated_block("portrait_decal", |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.field_value("body_part"); // TODO
                });
                vd.field_item("coa_mask", Item::File);
            });
            vd.multi_field_validated_block("throne_entity_user_data", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("animation", Item::PortraitAnimation);
                vd.field_bool("use_throne_transform");
            });
            vd.multi_field_validated_block("court_entity_user_data", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_bool("coat_of_arms");
            });
        });
        vd.multi_field_validated_block("state", |block, data| {
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
            vd.field_validated("time_offset", validate_time_offset);
            vd.multi_field_validated_block("start_event", validate_event);
            vd.multi_field_validated_block("event", validate_event);
            vd.multi_field_validated("propagate_state", |bv, data| {
                match bv {
                    BV::Value(_token) => (), // TODO
                    BV::Block(block) => {
                        let mut vd = Validator::new(block, data);
                        // TODO
                        vd.unknown_value_fields(|_, _| ());
                    }
                }
            });
        });
        vd.field_value("default_state"); // TODO: must be a state name
        vd.multi_field_validated_block("locator", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.field_value("name");
            vd.multi_field_validated_block("position", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(3);
            });
            vd.multi_field_validated_block("rotation", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(3);
            });
            vd.field_numeric("scale");
        });
        vd.multi_field_validated_block("attach", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|_, token| {
                // TODO: what are the keys here?
                data.verify_exists(Item::Asset, token);
            });
        });
    }

    pub fn validate_animation_set(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value("name");
        vd.req_field("reference_skeleton");
        vd.multi_field_item("reference_skeleton", Item::Pdxmesh);
        vd.multi_field_validated_block("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("id");
            vd.req_field("type");
            vd.field_value("id");
            if let Some(token) = vd.field_value("type") {
                let path = self.key.loc.pathname().smart_join_parent(token.as_str());
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
            warn(ErrorKey::UnknownField).msg("unknown asset type").loc(&self.key).push();
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
    vd.multi_field_validated_block("soundparameter", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|_, token| {
            // TODO: what are the keys here?
            token.expect_number();
        });
    });
    vd.multi_field_validated_block("sound", |block, data| {
        let mut vd = Validator::new(block, data);
        if let Some(token) = vd.field_value("soundeffect") {
            if !token.is("") {
                data.verify_exists(Item::Sound, token);
            }
        }
        vd.field_bool("stop_on_state_change");
    });
    vd.field_value("light"); // TODO
}

fn validate_meshsettings(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_value("name");
    vd.field_integer("index"); // TODO: do these need to be consecutive?
    vd.field_bool("shadow_only");
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
        data.verify_exists(Item::File, token);
    }
    vd.field_value("subpass");
    vd.field_value("shadow_shader");
    if Game::is_vic3() {
        vd.field_list("additional_shader_defines");
    }
}

fn validate_time_offset(bv: &BV, data: &Everything) {
    match bv {
        BV::Value(token) => {
            _ = token.expect_number();
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.req_tokens_numbers_exactly(2);
        }
    }
}
