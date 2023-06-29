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
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::{validate_cost, validate_duration, validate_modifiers_with_base};

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

        for (key, block) in block.iter_definitions_warn() {
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

        if let Some(block) = self.block.get_field_block("widget") {
            // Add builtin scopes added by known widget controllers
            if let Some(controller) = block.get_field_value("controller") {
                if controller.is("decision_option_list_controller") {
                    for block in block.get_field_blocks("item") {
                        if let Some(token) = block.get_field_value("value") {
                            sc.define_name(token.as_str(), Scopes::Bool, token.clone());
                        }
                    }
                } else if controller.is("create_holy_order") {
                    sc.define_name("ruler", Scopes::Character, controller.clone());
                } else if controller.is("revoke_holy_order_lease") {
                    sc.define_name("barony", Scopes::LandedTitle, controller.clone());
                }
            }
        }

        vd.req_field_warn("picture");
        vd.field_item("picture", Item::File);
        vd.field_item("extra_picture", Item::File);
        vd.field_bool("major");
        vd.field_integer("sort_order");
        vd.field_bool("is_invisible");
        vd.field_bool("ai_goal");
        vd.field_integer("ai_check_interval");
        if self.block.get_field_bool("ai_goal").unwrap_or(false) {
            vd.advice_field("ai_check_interval", "not needed if ai_goal = yes");
        }
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_item("confirm_click_sound", Item::Sound);

        if !vd.field_validated("selection_tooltip", |bv, data| {
            validate_desc(bv, data, &mut sc);
        }) {
            let loca = format!("{}_tooltip", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if !vd.field_validated("title", |bv, data| {
            validate_desc(bv, data, &mut sc);
        }) {
            data.localization.verify_exists(&self.key);
        }

        if !vd.field_validated("desc", |bv, data| {
            validate_desc(bv, data, &mut sc);
        }) {
            let loca = format!("{}_desc", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if !vd.field_validated("confirm_text", |bv, data| {
            validate_desc(bv, data, &mut sc);
        }) {
            let loca = format!("{}_confirm", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        vd.field_validated_block("is_shown", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("is_valid_showing_failures_only", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::FailuresOnly);
        });
        vd.field_validated_block("is_valid", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        // cost can have multiple definitions and they will be combined
        // however, two costs of the same type are not summed
        vd.field_validated_blocks("cost", |b, data| validate_cost(b, data, &mut sc));
        check_cost(&self.block.get_field_blocks("cost"));
        vd.field_validated_blocks("minimum_cost", |b, data| validate_cost(b, data, &mut sc));
        check_cost(&self.block.get_field_blocks("minimum_cost"));

        vd.field_validated_block("effect", |b, data| {
            validate_normal_effect(b, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("ai_potential", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block("should_create_alert", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field("widget"); // TODO
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
                    let msg = "This value of the gold cost overrides the previously set cost.";
                    warn(gold, ErrorKey::Conflict, msg);
                }
                seen_gold = true;
            }
            if let Some(prestige) = cost.get_field("prestige") {
                if seen_prestige {
                    let msg = "This value of the prestige cost overrides the previously set cost.";
                    warn(prestige, ErrorKey::Conflict, msg);
                }
                seen_prestige = true;
            }
            if let Some(piety) = cost.get_field("piety") {
                if seen_piety {
                    let msg = "This value of the piety cost overrides the previously set cost.";
                    warn(piety, ErrorKey::Conflict, msg);
                }
                seen_piety = true;
            }
        }
    }
}
