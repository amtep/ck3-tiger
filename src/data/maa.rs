use fnv::{FnvHashMap, FnvHashSet};
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_cost, validate_maa_stats};

#[derive(Clone, Debug, Default)]
pub struct MenAtArmsTypes {
    menatarmsbasetypes: FnvHashSet<String>,
    menatarmstypes: FnvHashMap<String, MenAtArmsType>,
}

impl MenAtArmsTypes {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.menatarmstypes.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "men-at-arms type");
            }
        }

        if let Some(base) = block.get_field_value("type") {
            self.menatarmsbasetypes.insert(base.to_string());
        }

        self.menatarmstypes
            .insert(key.to_string(), MenAtArmsType::new(key, block));
    }

    pub fn base_exists(&self, key: &str) -> bool {
        self.menatarmsbasetypes.contains(key)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.menatarmstypes.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.menatarmstypes.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for MenAtArmsTypes {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/men_at_arms_types")
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
pub struct MenAtArmsType {
    key: Token,
    block: Block,
}

impl MenAtArmsType {
    pub fn new(key: Token, block: Block) -> Self {
        MenAtArmsType { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("type");
        vd.field_item("type", Item::MenAtArmsBase);

        if let Some(icon_path) =
            data.get_defined_string_warn(&self.key, "NGameIcons|REGIMENTYPE_ICON_PATH")
        {
            if let Some(icon) = vd.field_value("icon") {
                let path = format!("{icon_path}/{icon}.dds");
                data.fileset.verify_exists_implied(&path, icon);
            } else if let Some(base) = self.block.get_field_value("type") {
                let base_path = format!("{icon_path}/{base}.dds");
                let path = format!("{icon_path}/{}.dds", self.key);
                if !data.fileset.exists(&base_path) {
                    data.fileset.verify_exists_implied(&path, &self.key);
                }
            }
        } else {
            vd.field_value("icon");
        }

        // TODO: "Mutually exclusive with being unlocked by innovation"
        vd.field_validated_block("can_recruit", |b, data| {
            let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());
            validate_normal_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        validate_maa_stats(&mut vd);
        vd.field_integer("siege_tier");
        vd.field_bool("fights_in_main_phase");

        vd.field_validated_block("buy_cost", |b, data| {
            let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());
            validate_cost(b, data, &mut sc);
        });
        vd.field_validated_block("low_maintenance_cost", |b, data| {
            let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());
            validate_cost(b, data, &mut sc);
        });
        vd.field_validated_block("high_maintenance_cost", |b, data| {
            let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());
            validate_cost(b, data, &mut sc);
        });

        vd.field_validated_block("terrain_bonus", validate_terrain_bonus);
        vd.field_validated_block("winter_bonus", validate_winter_bonus);
        vd.field_validated_block("era_bonus", validate_era_bonus);
        vd.field_validated_block("counters", validate_counters);

        vd.field_numeric("stack");
        vd.field_numeric("hired_stack_size");
        vd.field_integer("max_sub_regiments"); // undocumented

        vd.field_script_value_rooted("ai_quality", Scopes::Character);
        vd.field_bool("allowed_in_hired_troops"); // undocumented
        vd.field_bool("fallback_in_hired_troops_if_unlocked");
        vd.field_bool("mercenary_fallback"); // undocumented
        vd.field_bool("holy_order_fallback"); // undocumented
    }
}

pub fn validate_terrain_bonus(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, bv) in vd.unknown_keys() {
        if let Some(block) = bv.expect_block() {
            data.verify_exists(Item::Terrain, key);
            let mut vd = Validator::new(block, data);
            validate_maa_stats(&mut vd);
        }
    }
}

pub fn validate_winter_bonus(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, bv) in vd.unknown_keys() {
        if let Some(block) = bv.expect_block() {
            if !(key.is("harsh_winter") || key.is("normal_winter")) {
                warn(key, ErrorKey::Validation, "unknown winter type");
            }
            let mut vd = Validator::new(block, data);
            validate_maa_stats(&mut vd);
        }
    }
}

fn validate_era_bonus(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for (key, bv) in vd.unknown_keys() {
        if let Some(block) = bv.expect_block() {
            data.verify_exists(Item::CultureEra, key);
            let mut vd = Validator::new(block, data);
            validate_maa_stats(&mut vd);
        }
    }
}

fn validate_counters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for key in &data.menatarmstypes.menatarmsbasetypes {
        vd.field_numeric(key);
    }
}
