use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, DefinitionItem};
use crate::data::localization::Localization;
use crate::desc::verify_desc_locas;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, will_log, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind, Fileset};
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: FnvHashMap<String, Decision>,
}

impl Decisions {
    pub fn load_decision(&mut self, key: Token, block: &Block, values: Vec<(Token, Token)>) {
        if let Some(other) = self.decisions.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind && will_log(&key, ErrorKey::Duplicate) {
                error(
                    &key,
                    ErrorKey::Duplicate,
                    "decision redefines an existing decision",
                );
                info(
                    &other.key,
                    ErrorKey::Duplicate,
                    "the other decision is here",
                );
            }
        }
        let strkey = key.to_string();
        let decision = Decision::new(key, block.clone(), values);
        decision.validate();
        self.decisions.insert(strkey, decision);
    }

    pub fn check_have_locas(&self, locs: &Localization) {
        for decision in self.decisions.values() {
            decision.check_have_locas(locs);
        }
    }

    pub fn check_have_files(&self, fileset: &Fileset) {
        for decision in self.decisions.values() {
            decision.check_have_files(fileset);
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

        let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

        let block = match PdxFile::read(entry.path(), entry.kind(), fullpath) {
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

        let mut decision_values: Vec<(Token, Token)> = Vec::new();

        for def in block.iter_definitions_warn() {
            match def {
                DefinitionItem::Keyword(key) => error_info(
                    key,
                    ErrorKey::Validation,
                    "unexpected token",
                    "Did you forget an = ?",
                ),
                DefinitionItem::Assignment(key, value) => {
                    if key.as_str().starts_with('@') {
                        decision_values.push((key.clone(), value.clone()));
                    } else {
                        error(
                            key,
                            ErrorKey::Validation,
                            "unknown setting in decision file",
                        );
                    }
                }
                DefinitionItem::Definition(key, b) => {
                    self.load_decision(key.clone(), b, decision_values.clone());
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decision {
    key: Token,
    values: Vec<(Token, Token)>,
    block: Block,
}

impl Decision {
    pub fn new(key: Token, block: Block, values: Vec<(Token, Token)>) -> Self {
        Decision { key, values, block }
    }

    pub fn check_have_locas(&self, locs: &Localization) {
        match self.block.get_field("title") {
            Some(v) => verify_desc_locas(v, locs),
            None => locs.verify_exists(self.key.as_str(), &self.key),
        }
        match self.block.get_field("desc") {
            Some(v) => verify_desc_locas(v, locs),
            None => locs.verify_exists(&(self.key.to_string() + "_desc"), &self.key),
        }
        match self.block.get_field("selection_tooltip") {
            Some(v) => verify_desc_locas(v, locs),
            None => locs.verify_exists(&(self.key.to_string() + "_tooltip"), &self.key),
        }
        match self.block.get_field("confirm_text") {
            Some(v) => verify_desc_locas(v, locs),
            None => locs.verify_exists(&(self.key.to_string() + "_confirm"), &self.key),
        }
    }

    pub fn check_have_files(&self, fileset: &Fileset) {
        if let Some(picture) = self.block.get_field_value("picture") {
            fileset.verify_exists(picture);
        }
        if let Some(extra_picture) = self.block.get_field_value("extra_picture") {
            fileset.verify_exists(extra_picture);
        }
        // confirm_click_sound in vanilla kind of looks like a filename but it isn't.
        // TODO: check widget
    }

    fn validate(&self) {
        let mut vd = Validator::new(&self.block);

        vd.req_field_value("picture");
        vd.opt_field_value("extra_picture");
        vd.opt_field_bool("major");
        vd.opt_field_integer("sort_order");
        vd.opt_field_bool("is_invisible");
        vd.opt_field_bool("ai_goal");
        vd.opt_field_integer("ai_check_interval");
        if let Some(ai_goal) = self.block.get_field_value("ai_goal") {
            if ai_goal.is("yes") {
                vd.advice_field("ai_check_interval", "not needed if ai_goal = yes");
            }
        }
        vd.opt_field_block("cooldown");
        vd.opt_field_value("confirm_click_sound");
        vd.opt_field("selection_tooltip");
        vd.opt_field("title");
        vd.opt_field("desc");
        vd.opt_field("confirm_text");
        vd.opt_field_block("is_shown");
        vd.opt_field_block("is_valid_showing_failures_only");
        vd.opt_field_block("is_valid");
        // cost can have multiple definitions and they will be combined
        // however, two costs of the same type are not summed
        vd.opt_field_validated_blocks("cost", validate_cost);
        vd.opt_field_validated_blocks("minimum_cost", validate_cost);
        vd.opt_field_block("effect");
        vd.opt_field_block("ai_potential");
        vd.opt_field_block("ai_will_do");
        vd.opt_field_block("should_create_alert");
        vd.opt_field("widget");
        vd.warn_remaining();

        check_cost(&self.block.get_field_blocks("cost"));
        check_cost(&self.block.get_field_blocks("minimum_cost"));
    }
}

fn validate_cost(block: &Block) {
    let mut vd = Validator::new(block);
    vd.opt_field("gold");
    vd.opt_field("prestige");
    vd.opt_field("piety");
    vd.warn_remaining();
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
