use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_cost;

#[derive(Clone, Debug, Default)]
pub struct CourtPositions {
    courtpos: FnvHashMap<String, CourtPosition>,
}

impl CourtPositions {
    pub fn load_item(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.courtpos.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "court position");
            }
        }
        self.courtpos
            .insert(key.to_string(), CourtPosition::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.courtpos.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.courtpos.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for CourtPositions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/court_positions/types")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };

        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_item(key.clone(), block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct CourtPosition {
    key: Token,
    block: Block,
}

impl CourtPosition {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        data.verify_exists(Item::Localization, &self.key);
        let loca = format!("{}_desc", self.key);
        data.verify_exists_implied(Item::Localization, &loca, &self.key);

        let mut vd = Validator::new(&self.block, data);
        vd.advice_field("skill", "`skill` was removed in 1.8");
        vd.field_integer("max_available_positions");
        vd.field_item("category", Item::CourtPositionCategory);
        vd.field_choice("minimum_rank", &["county", "duchy", "kingdom", "empire"]);
        vd.field_bool("is_travel_related");
        vd.field_script_value_rooted("opinion", Scopes::None);
        vd.field_validated_block("aptitude_level_breakpoints", validate_breakpoints);
        vd.field_script_value_rooted("aptitude", Scopes::Character);
        if let Some((key, block)) = vd.definition("is_shown") {
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            validate_normal_trigger(block, data, &mut sc, false);
        }

        if let Some((key, block)) = vd.definition("valid_position") {
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            validate_normal_trigger(block, data, &mut sc, true);
        }
        if let Some((key, block)) = vd.definition("is_shown_character") {
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            validate_normal_trigger(block, data, &mut sc, false);
        }
        if let Some((key, block)) = vd.definition("valid_character") {
            let mut sc = ScopeContext::new_root(Scopes::None, key);
            validate_normal_trigger(block, data, &mut sc, true);
        }

        if let Some((key, block)) = vd.definition("revoke_cost") {
            // guessing that root is the liege here
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            validate_cost(block, data, &mut sc);
        };

        if let Some((key, block)) = vd.definition("salary") {
            let mut sc = ScopeContext::new_root(Scopes::None, key);
            validate_cost(block, data, &mut sc);
        };

        if let Some((key, block)) = vd.definition("base_employer_modifier") {
            let vd = Validator::new(block, data);
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            validate_modifs(block, data, ModifKinds::Character, &mut sc, vd);
        }

        if let Some(block) = vd.field_block("scaling_employer_modifiers") {
            validate_scaling_employer_modifiers(block, data);
        }

        if let Some((key, block)) = vd.definition("modifier") {
            let vd = Validator::new(block, data);
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            validate_modifs(block, data, ModifKinds::Character, &mut sc, vd);
        }

        vd.field_item("custom_employer_modifier_description", Item::Localization);
        vd.field_item("custom_employee_modifier_description", Item::Localization);

        if let Some((key, block)) = vd.definition("search_for_courtier") {
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            validate_normal_effect(block, data, &mut sc, false);
        }

        if let Some((key, block)) = vd.definition("on_court_position_received") {
            let mut sc = ScopeContext::new_root(Scopes::None, key);
            validate_normal_effect(block, data, &mut sc, false);
        }
        if let Some((key, block)) = vd.definition("on_court_position_revoked") {
            let mut sc = ScopeContext::new_root(Scopes::None, key);
            validate_normal_effect(block, data, &mut sc, false);
        }
        if let Some((key, block)) = vd.definition("on_court_position_invalidated") {
            let mut sc = ScopeContext::new_root(Scopes::None, key);
            validate_normal_effect(block, data, &mut sc, false);
        }

        vd.field_script_value_rooted("candidate_score", Scopes::None);
    }
}

fn validate_breakpoints(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_tokens_integers_exactly(4);
}

fn validate_scaling_employer_modifiers(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for field in &[
        "aptitude_level_1",
        "aptitude_level_2",
        "aptitude_level_3",
        "aptitude_level_4",
        "aptitude_level_5",
    ] {
        if let Some((key, b)) = vd.definition(field) {
            let mut sc = ScopeContext::new_root(Scopes::Character, key);
            let vd = Validator::new(b, data);
            validate_modifs(b, data, ModifKinds::Character, &mut sc, vd);
        };
    }
}
