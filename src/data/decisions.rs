use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::validate_normal_effect;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_cooldown, validate_cost, validate_modifiers_with_base};

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: FnvHashMap<String, Decision>,
}

impl Decisions {
    pub fn load_decision(&mut self, key: Token, block: &Block) {
        if let Some(other) = self.decisions.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "decision");
            }
        }
        self.decisions
            .insert(key.to_string(), Decision::new(key, block.clone()));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.decisions.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.decisions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Decisions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/decisions")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };

        for (key, block) in block.iter_pure_definitions_warn() {
            self.load_decision(key.clone(), block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decision {
    key: Token,
    block: Block,
}

impl Decision {
    pub fn new(key: Token, block: Block) -> Self {
        Decision { key, block }
    }

    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, self.key.clone());

        vd.req_field_warn("picture");
        if let Some(token) = vd.field_value("picture") {
            data.fileset.verify_exists(token);
        }
        if let Some(token) = vd.field_value("extra_picture") {
            data.fileset.verify_exists(token);
        }
        vd.field_bool("major");
        vd.field_integer("sort_order");
        vd.field_bool("is_invisible");
        vd.field_bool("ai_goal");
        vd.field_integer("ai_check_interval");
        if self.block.get_field_bool("ai_goal").unwrap_or(false) {
            vd.advice_field("ai_check_interval", "not needed if ai_goal = yes");
        }
        vd.field_validated_block_sc("cooldown", &mut sc, validate_cooldown);

        // kind of looks like a filename but it isn't.
        vd.field_value("confirm_click_sound");

        if let Some(bv) = vd.field("selection_tooltip") {
            validate_desc(bv, data, &mut sc);
        } else {
            let loca = format!("{}_tooltip", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if let Some(bv) = vd.field("title") {
            validate_desc(bv, data, &mut sc);
        } else {
            data.localization.verify_exists(&self.key);
        }

        if let Some(bv) = vd.field("desc") {
            validate_desc(bv, data, &mut sc);
        } else {
            let loca = format!("{}_desc", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if let Some(bv) = vd.field("confirm_text") {
            validate_desc(bv, data, &mut sc);
        } else {
            let loca = format!("{}_confirm", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        vd.field_validated_block("is_shown", |b, data| {
            validate_normal_trigger(b, data, &mut sc, false);
        });
        vd.field_validated_block("is_valid_showing_failures_only", |b, data| {
            validate_normal_trigger(b, data, &mut sc, true);
        });
        vd.field_validated_block("is_valid", |b, data| {
            validate_normal_trigger(b, data, &mut sc, true);
        });

        // cost can have multiple definitions and they will be combined
        // however, two costs of the same type are not summed
        vd.field_validated_blocks("cost", |b, data| validate_cost(b, data, &mut sc));
        check_cost(&self.block.get_field_blocks("cost"));
        vd.field_validated_blocks("minimum_cost", |b, data| validate_cost(b, data, &mut sc));
        check_cost(&self.block.get_field_blocks("minimum_cost"));

        vd.field_validated_block("effect", |b, data| {
            validate_normal_effect(b, data, &mut sc, true);
        });
        vd.field_validated_block("ai_potential", |b, data| {
            validate_normal_trigger(b, data, &mut sc, false);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block("should_create_alert", |b, data| {
            validate_normal_trigger(b, data, &mut sc, false);
        });
        vd.field("widget");
    }
}

fn check_cost(blocks: &[&Block]) {
    let mut seen_gold = false;
    let mut seen_prestige = false;
    let mut seen_piety = false;
    if blocks.len() > 1 {
        for cost in blocks {
            if let Some(gold) = cost.get_field("gold") {
                if seen_gold {
                    warn(
                        gold,
                        ErrorKey::Conflict,
                        "This value of the gold cost overrides the previously set cost.",
                    );
                }
                seen_gold = true;
            }
            if let Some(prestige) = cost.get_field("prestige") {
                if seen_prestige {
                    warn(
                        prestige,
                        ErrorKey::Conflict,
                        "This value of the prestige cost overrides the previously set cost.",
                    );
                }
                seen_prestige = true;
            }
            if let Some(piety) = cost.get_field("piety") {
                if seen_piety {
                    warn(
                        piety,
                        ErrorKey::Conflict,
                        "This value of the piety cost overrides the previously set cost.",
                    );
                }
                seen_piety = true;
            }
        }
    }
}
