use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::error_info;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxFile;
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug, Default)]
pub struct Terrains {
    terrains: FnvHashMap<String, Terrain>,
    modif_char_keys: Vec<String>,
    modif_prov_keys: Vec<String>,
}

impl Terrains {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.terrains.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "terrain");
            }
        } else {
            let m = format!("{}_advantage", key);
            self.modif_char_keys.push(m);
            let m = format!("{}_attrition_mult", key);
            self.modif_char_keys.push(m);
            let m = format!("{}_cancel_negative_supply", key);
            self.modif_char_keys.push(m);
            let m = format!("{}_max_combat_roll", key);
            self.modif_char_keys.push(m);
            let m = format!("{}_min_combat_roll", key);
            self.modif_char_keys.push(m);

            let m = format!("{}_construction_gold_cost", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_construction_piety_cost", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_construction_prestige_cost", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_development_growth", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_development_growth_factor", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_holding_construction_gold_cost", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_holding_construction_piety_cost", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_holding_construction_prestige_cost", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_levy_size", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_supply_limit", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_supply_limit_mult", key);
            self.modif_prov_keys.push(m);
            let m = format!("{}_tax_mult", key);
            self.modif_prov_keys.push(m);
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

    pub fn iter_modif_char_keys(&self) -> impl Iterator<Item = &String> {
        self.modif_char_keys.iter()
    }

    pub fn iter_modif_prov_keys(&self) -> impl Iterator<Item = &String> {
        self.modif_prov_keys.iter()
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
            Ok(block) => block,
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{:#}", e),
                );
                return;
            }
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
        vd.req_field("color");

        vd.field_numeric("movement_speed");
        vd.field_validated_block("color", validate_color);

        vd.field_validated_block("attacker_modifier", validate_combat_modifier);
        vd.field_validated_block("defender_modifier", validate_combat_modifier);
        vd.field_block("attacker_combat_effects"); // TODO
        vd.field_block("defender_combat_effects"); // TODO

        vd.field_numeric("combat_width");
        vd.field_bool("is_desert");
        vd.field_bool("is_jungle");
        vd.field_numeric("audio_parameter"); // ??

        vd.field_validated_block("province_modifier", validate_province_modifier);

        vd.warn_remaining();
    }
}

pub fn validate_combat_modifier(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    validate_modifs(block, data, ModifKinds::Terrain, &mut vd);
    vd.warn_remaining();
}

pub fn validate_province_modifier(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    validate_modifs(block, data, ModifKinds::Province, &mut vd);
    vd.warn_remaining();
}
