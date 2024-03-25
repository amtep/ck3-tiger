use std::path::PathBuf;

use crate::block::Block;
use crate::ck3::validate::{validate_cost, validate_maa_stats};
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::pdxfile::PdxFile;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct MenAtArmsTypes {
    menatarmsbasetypes: TigerHashSet<Token>,
    menatarmstypes: TigerHashMap<&'static str, MenAtArmsType>,
    menatarmsbasetypes_lc: TigerHashSet<Lowercase<'static>>,
}

impl MenAtArmsTypes {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.menatarmstypes.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "men-at-arms type");
            }
        }

        self.menatarmstypes.insert(key.as_str(), MenAtArmsType::new(key, block));
    }

    pub fn base_exists(&self, key: &str) -> bool {
        self.menatarmsbasetypes.contains(key)
    }

    pub fn base_exists_lc(&self, key: &Lowercase) -> bool {
        self.menatarmsbasetypes_lc.contains(key)
    }

    pub fn iter_base_keys(&self) -> impl Iterator<Item = &Token> {
        self.menatarmsbasetypes.iter()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.menatarmstypes.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.menatarmstypes.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.menatarmstypes.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for MenAtArmsTypes {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/men_at_arms_types")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }

    fn finalize(&mut self) {
        for maa in self.menatarmstypes.values() {
            if let Some(base) = maa.block.get_field_value("type") {
                self.menatarmsbasetypes.insert(base.clone());
                self.menatarmsbasetypes_lc.insert(Lowercase::new(base.as_str()));
            }
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
        if let Some(base) = self.block.get_field_value("type") {
            let modif = format!("stationed_{base}_damage_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_damage_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_pursuit_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_pursuit_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_screen_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_screen_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_toughness_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_toughness_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_siege_value_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("stationed_{base}_siege_value_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);

            let modif = format!("{base}_damage_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_damage_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_pursuit_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_pursuit_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_screen_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_screen_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_toughness_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_toughness_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_recruitment_cost_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_siege_value_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_siege_value_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_maintenance_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_max_size_add");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
            let modif = format!("{base}_max_size_mult");
            data.verify_exists_implied(Item::ModifierFormat, &modif, base);
        }

        data.verify_exists(Item::Localization, &self.key);
        let loca = format!("{}_flavor", &self.key);
        data.verify_exists_implied(Item::Localization, &loca, &self.key);

        if let Some(icon_path) =
            data.get_defined_string_warn(&self.key, "NGameIcons|REGIMENTYPE_ICON_PATH")
        {
            if let Some(icon) = vd.field_value("icon") {
                let path = format!("{icon_path}/{icon}.dds");
                data.verify_exists_implied(Item::File, &path, icon);
            } else if let Some(base) = self.block.get_field_value("type") {
                let base_path = format!("{icon_path}/{base}.dds");
                let path = format!("{icon_path}/{}.dds", self.key);
                data.mark_used(Item::File, &base_path);
                if !data.fileset.exists(&base_path) {
                    data.verify_exists_implied(Item::File, &path, &self.key);
                }
            }
        } else {
            vd.field_value("icon");
        }

        if let Some(icon_path) =
            data.get_defined_string_warn(&self.key, "NGameIcons|REGIMENTYPE_HORIZONTAL_IMAGE_PATH")
        {
            if let Some(icon) = self.block.get_field_value("icon") {
                let path = format!("{icon_path}/{icon}.dds");
                data.verify_exists_implied(Item::File, &path, icon);
            } else if let Some(base) = self.block.get_field_value("type") {
                let base_path = format!("{icon_path}/{base}.dds");
                let path = format!("{icon_path}/{}.dds", self.key);
                data.mark_used(Item::File, &base_path);
                if !data.fileset.exists(&base_path) {
                    data.verify_exists_implied(Item::File, &path, &self.key);
                }
            }
        }

        if let Some(icon_path) =
            data.get_defined_string_warn(&self.key, "NGameIcons|REGIMENTYPE_VERTICAL_IMAGE_PATH")
        {
            if let Some(icon) = self.block.get_field_value("icon") {
                let path = format!("{icon_path}/{icon}.dds");
                data.verify_exists_implied(Item::File, &path, icon);
            } else if let Some(base) = self.block.get_field_value("type") {
                let base_path = format!("{icon_path}/{base}.dds");
                let path = format!("{icon_path}/{}.dds", self.key);
                data.mark_used(Item::File, &base_path);
                if !data.fileset.exists(&base_path) {
                    data.verify_exists_implied(Item::File, &path, &self.key);
                }
            }
        }

        // TODO: "Mutually exclusive with being unlocked by innovation"
        vd.field_validated_key_block("can_recruit", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_integer("max");
        validate_maa_stats(&mut vd);
        vd.field_integer("siege_tier");
        vd.field_bool("fights_in_main_phase");

        for field in &["buy_cost", "low_maintenance_cost", "high_maintenance_cost"] {
            vd.field_validated_key_block(field, |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                validate_cost(block, data, &mut sc);
            });
        }

        vd.field_validated_block("terrain_bonus", validate_terrain_bonus);
        vd.field_validated_block("winter_bonus", validate_winter_bonus);
        vd.field_validated_block("era_bonus", validate_era_bonus);
        vd.field_validated_block("counters", validate_counters);

        vd.field_numeric("stack");
        vd.field_numeric("hired_stack_size");
        vd.field_integer("max_sub_regiments");

        vd.field_script_value_rooted("ai_quality", Scopes::Character);
        vd.field_bool("allowed_in_hired_troops");
        vd.field_bool("fallback_in_hired_troops_if_unlocked");
        vd.field_bool("mercenary_fallback");
        vd.field_bool("holy_order_fallback");
    }
}

pub fn validate_terrain_bonus(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::Terrain, key);
        let mut vd = Validator::new(block, data);
        validate_maa_stats(&mut vd);
    });
}

pub fn validate_winter_bonus(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_block_fields(|key, block| {
        if !(key.is("harsh_winter") || key.is("normal_winter")) {
            warn(ErrorKey::Validation).msg("unknown winter type").loc(key).push();
        }
        let mut vd = Validator::new(block, data);
        validate_maa_stats(&mut vd);
    });
}

fn validate_era_bonus(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::CultureEra, key);
        let mut vd = Validator::new(block, data);
        validate_maa_stats(&mut vd);
    });
}

fn validate_counters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for key in &data.menatarmstypes.menatarmsbasetypes {
        vd.field_numeric(key.as_str());
    }
}
