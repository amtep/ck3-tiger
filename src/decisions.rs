use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BlockOrValue, DefinitionItem, Token};
use crate::desc::verify_desc_locas;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, will_log, LogPauseRaii};
use crate::everything::FileHandler;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::validate::{Validate, ValidationError};

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: FnvHashMap<String, DecisionEntry>,

    // These decisions are known to exist, so don't warn abour them not being found,
    // but they had errors on validation.
    error_decisions: FnvHashMap<String, Token>,
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
        self.decisions.insert(
            key.to_string(),
            DecisionEntry::new(key, block.clone(), values),
        );
    }

    pub fn check_have_localizations(&self, locs: &Localization) {
        for decision in self.decisions.values() {
            decision.check_have_localizations(locs);
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

    fn config(&mut self, _config: &Block) {}

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

    fn finalize(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct DecisionEntry {
    key: Token,
    values: Vec<(Token, Token)>,

    decision: Option<Decision>,
}

impl DecisionEntry {
    pub fn new(key: Token, block: Block, values: Vec<(Token, Token)>) -> Self {
        let decision = Decision::from_block(block, key.as_str()).ok();
        if let Some(decision) = &decision {
            decision.check();
        }
        DecisionEntry {
            key,
            values,
            decision,
        }
    }

    pub fn check_have_localizations(&self, locs: &Localization) {
        if self.decision.is_none() {
            return;
        }
        let decision = self.decision.as_ref().unwrap();

        // TODO: if these fields have complex descriptions, parse them to figure out
        // which loca entries must exist.

        match decision.title.as_ref() {
            Some(v) => verify_desc_locas(v, locs, "decision title"),
            None => locs.verify_have_key(self.key.as_str(), &self.key, "decision title"),
        }
        match decision.desc.as_ref() {
            Some(v) => verify_desc_locas(v, locs, "decision description"),
            None => locs.verify_have_key(
                &(self.key.to_string() + "_desc"),
                &self.key,
                "decision description",
            ),
        }
        match decision.tooltip.as_ref() {
            Some(v) => verify_desc_locas(v, locs, "decision tooltip"),
            None => locs.verify_have_key(
                &(self.key.to_string() + "_tooltip"),
                &self.key,
                "decision tooltip",
            ),
        }
        match decision.confirm.as_ref() {
            Some(v) => verify_desc_locas(v, locs, "decision confirm text"),
            None => locs.verify_have_key(
                &(self.key.to_string() + "_confirm"),
                &self.key,
                "decision confirm text",
            ),
        }
    }

    pub fn check_have_files(&self, fileset: &Fileset) {
        if self.decision.is_none() {
            return;
        }
        let decision = self.decision.as_ref().unwrap();

        fileset.verify_have_file(&decision.picture);
        if let Some(extra_picture) = &decision.extra_picture {
            fileset.verify_have_file(extra_picture);
        }
        // confirm_click_sound in vanilla kind of looks like a filename but it isn't.
        // TODO: check widget
    }
}

#[derive(Clone, Debug)]
pub struct Decision {
    picture: Token,
    extra_picture: Option<Token>,
    major: bool, // default no
    sort_order: Option<i64>,
    is_invisible: bool, // default no
    ai_goal: bool,      // default no
    ai_check_interval: Option<i64>,
    cooldown: Option<Block>,
    confirm_click_sound: Option<Token>,
    tooltip: Option<BlockOrValue>,
    title: Option<BlockOrValue>,
    desc: Option<BlockOrValue>,
    confirm: Option<BlockOrValue>,
    is_shown: Option<Block>,
    is_valid_showing_failures_only: Option<Block>,
    is_valid: Option<Block>,
    cost: Vec<DecisionCost>,
    minimum_cost: Vec<DecisionCost>,
    effect: Option<Block>,
    ai_potential: Option<Block>,
    ai_will_do: Option<Block>,
    should_create_alert: Option<Block>,
    widget: Option<BlockOrValue>,
}

impl Decision {
    pub fn check(&self) {
        let mut seen_gold = false;
        let mut seen_prestige = false;
        let mut seen_piety = false;
        if self.cost.len() > 1 {
            for cost in &self.cost {
                if let Some(gold) = &cost.gold {
                    if seen_gold {
                        warn(
                            gold,
                            ErrorKey::Conflict,
                            "This value of the gold cost overrides the previously set cost.",
                        );
                    }
                    seen_gold = true;
                }
                if let Some(prestige) = &cost.prestige {
                    if seen_prestige {
                        warn(
                            prestige,
                            ErrorKey::Conflict,
                            "This value of the prestige cost overrides the previously set cost.",
                        );
                    }
                    seen_prestige = true;
                }
                if let Some(piety) = &cost.piety {
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
}

impl Validate for Decision {
    fn from_block(block: Block, id: &str) -> Result<Self, ValidationError> {
        let mut vd = Validator::new(&block, id);

        let picture = vd.require_unique_field_value("picture");
        let extra_picture = vd.allow_unique_field_value("extra_picture");
        let major = vd.allow_unique_field_boolean("major").unwrap_or(false);
        let sort_order = vd.allow_unique_field_integer("sort_order");
        let is_invisible = vd
            .allow_unique_field_boolean("is_invisible")
            .unwrap_or(false);
        let ai_goal = vd.allow_unique_field_boolean("ai_goal").unwrap_or(false);
        let ai_check_interval = vd.allow_unique_field_integer("ai_check_interval");
        let cooldown = vd.allow_unique_field_block("cooldown");
        let confirm_click_sound = vd.allow_unique_field_value("confirm_click_sound");
        let tooltip = vd.allow_unique_field("selection_tooltip");
        let title = vd.allow_unique_field("title");
        let desc = vd.allow_unique_field("desc");
        let confirm = vd.allow_unique_field("confirm_text");
        let is_shown = vd.allow_unique_field_block("is_shown");
        let is_valid_showing_failures_only =
            vd.allow_unique_field_block("is_valid_showing_failures_only");
        let is_valid = vd.allow_unique_field_block("is_valid");
        // cost can have multiple definitions and they will be combined
        // TODO: figure out if cost { gold = 500 } cost { gold = 500} will result in gold = 1000
        let cost = vd.allow_field_validated_blocks("cost");
        let minimum_cost = vd.allow_field_validated_blocks("minimum_cost");
        let effect = vd.allow_unique_field_block("effect");
        let ai_potential = vd.allow_unique_field_block("ai_potential");
        let ai_will_do = vd.allow_unique_field_block("ai_will_do");
        let should_create_alert = vd.allow_unique_field_block("should_create_alert");
        let widget = vd.allow_unique_field("widget");
        vd.warn_unused_entries();

        if let Some(err) = vd.err {
            return Err(err);
        }

        let decision = Decision {
            picture: picture?,
            extra_picture,
            major,
            sort_order,
            is_invisible,
            ai_goal,
            ai_check_interval,
            cooldown,
            confirm_click_sound,
            tooltip,
            title,
            desc,
            confirm,
            is_shown,
            is_valid_showing_failures_only,
            is_valid,
            cost: cost?,
            minimum_cost: minimum_cost?,
            effect,
            ai_potential,
            ai_will_do,
            should_create_alert,
            widget,
        };

        Ok(decision)
    }
}

#[derive(Clone, Debug)]
pub struct DecisionCost {
    gold: Option<BlockOrValue>,
    prestige: Option<BlockOrValue>,
    piety: Option<BlockOrValue>,
}

impl Validate for DecisionCost {
    fn from_block(block: Block, id: &str) -> Result<Self, ValidationError> {
        let mut vd = Validator::new(&block, id);

        let gold = vd.allow_unique_field("gold");
        let prestige = vd.allow_unique_field("prestige");
        let piety = vd.allow_unique_field("piety");
        vd.warn_unused_entries();

        if let Some(err) = vd.err {
            return Err(err);
        }

        Ok(DecisionCost {
            gold,
            prestige,
            piety,
        })
    }
}
