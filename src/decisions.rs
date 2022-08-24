use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, will_log, LogPauseRaii};
use crate::everything::FileHandler;
use crate::fileset::{FileEntry, FileKind, Fileset};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::scope::validator::Validator;
use crate::scope::{Comparator, Scope, ScopeOrValue, Token};
use crate::validate::{Validate, ValidationError};

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: FnvHashMap<String, DecisionEntry>,

    // These decisions are known to exist, so don't warn abour them not being found,
    // but they had errors on validation.
    error_decisions: FnvHashMap<String, Token>,
}

impl Decisions {
    pub fn load_decision(&mut self, key: Token, scope: &Scope, values: Vec<(Token, Token)>) {
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
            DecisionEntry::new(key, scope.clone(), values),
        );
    }
}

impl Decisions {
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

    fn config(&mut self, _config: &Scope) {}

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

        let scope = match PdxFile::read(entry.path(), entry.kind(), fullpath) {
            Ok(scope) => scope,
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

        for (k, cmp, v) in scope.iter_items() {
            if let Some(key) = k {
                if !matches!(*cmp, Comparator::Eq) {
                    error(
                        key,
                        ErrorKey::Validation,
                        &format!("expected `{} =`, found `{}`", key, cmp),
                    );
                }
                match v {
                    ScopeOrValue::Token(t) => {
                        if key.as_str().starts_with('@') {
                            decision_values.push((key.clone(), t.clone()));
                        } else {
                            error(
                                key,
                                ErrorKey::Validation,
                                "unknown setting in decision file",
                            );
                        }
                    }
                    ScopeOrValue::Scope(s) => {
                        self.load_decision(key.clone(), s, decision_values.clone());
                    }
                }
            } else {
                match v {
                    ScopeOrValue::Token(t) => error_info(
                        t,
                        ErrorKey::Validation,
                        "unexpected token",
                        "Did you forget an = ?",
                    ),
                    ScopeOrValue::Scope(s) => error_info(
                        s,
                        ErrorKey::Validation,
                        "unexpected block",
                        "Did you forget an = ?",
                    ),
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
    pub fn new(key: Token, scope: Scope, values: Vec<(Token, Token)>) -> Self {
        let decision = Decision::from_scope(scope, key.as_str()).ok();
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
            Some(ScopeOrValue::Scope(_)) => (),
            Some(ScopeOrValue::Token(t)) => locs.verify_have_key(t.as_str(), t, "decision title"),
            None => locs.verify_have_key(self.key.as_str(), &self.key, "decision title"),
        }
        match decision.desc.as_ref() {
            Some(ScopeOrValue::Scope(_)) => (),
            Some(ScopeOrValue::Token(t)) => {
                locs.verify_have_key(t.as_str(), t, "decision description");
            }
            None => locs.verify_have_key(
                &(self.key.to_string() + "_desc"),
                &self.key,
                "decision description",
            ),
        }
        match decision.tooltip.as_ref() {
            Some(ScopeOrValue::Scope(_)) => (),
            Some(ScopeOrValue::Token(t)) => {
                locs.verify_have_key(t.as_str(), t, "decision tooltip");
            }
            None => locs.verify_have_key(
                &(self.key.to_string() + "_tooltip"),
                &self.key,
                "decision tooltip",
            ),
        }
        match decision.confirm.as_ref() {
            Some(ScopeOrValue::Scope(_)) => (),
            Some(ScopeOrValue::Token(t)) => locs.verify_have_key(t.as_str(), &t, "decision button"),
            None => locs.verify_have_key(
                &(self.key.to_string() + "_confirm"),
                &self.key,
                "decision button",
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
    cooldown: Option<Scope>,
    confirm_click_sound: Option<Token>,
    tooltip: Option<ScopeOrValue>,
    title: Option<ScopeOrValue>,
    desc: Option<ScopeOrValue>,
    confirm: Option<ScopeOrValue>,
    is_shown: Option<Scope>,
    is_valid_showing_failures_only: Option<Scope>,
    is_valid: Option<Scope>,
    cost: Vec<DecisionCost>,
    minimum_cost: Vec<DecisionCost>,
    effect: Option<Scope>,
    ai_potential: Option<Scope>,
    ai_will_do: Option<Scope>,
    should_create_alert: Option<Scope>,
    widget: Option<ScopeOrValue>,
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
    fn from_scope(scope: Scope, id: &str) -> Result<Self, ValidationError> {
        let mut vd = Validator::new(&scope, id);

        let picture = vd.require_unique_field_value("picture");
        let extra_picture = vd.allow_unique_field_value("extra_picture");
        let major = vd.allow_unique_field_boolean("major").unwrap_or(false);
        let sort_order = vd.allow_unique_field_integer("sort_order");
        let is_invisible = vd
            .allow_unique_field_boolean("is_invisible")
            .unwrap_or(false);
        let ai_goal = vd.allow_unique_field_boolean("ai_goal").unwrap_or(false);
        let ai_check_interval = vd.allow_unique_field_integer("ai_check_interval");
        let cooldown = vd.allow_unique_field_scope("cooldown");
        let confirm_click_sound = vd.allow_unique_field_value("confirm_click_sound");
        let tooltip = vd.allow_unique_field("selection_tooltip");
        let title = vd.allow_unique_field("title");
        let desc = vd.allow_unique_field("desc");
        let confirm = vd.allow_unique_field("confirm_text");
        let is_shown = vd.allow_unique_field_scope("is_shown");
        let is_valid_showing_failures_only =
            vd.allow_unique_field_scope("is_valid_showing_failures_only");
        let is_valid = vd.allow_unique_field_scope("is_valid");
        // cost can have multiple definitions and they will be combined
        // TODO: figure out if cost { gold = 500 } cost { gold = 500} will result in gold = 1000
        let cost = vd.allow_field_validated_scopes("cost");
        let minimum_cost = vd.allow_field_validated_scopes("minimum_cost");
        let effect = vd.allow_unique_field_scope("effect");
        let ai_potential = vd.allow_unique_field_scope("ai_potential");
        let ai_will_do = vd.allow_unique_field_scope("ai_will_do");
        let should_create_alert = vd.allow_unique_field_scope("should_create_alert");
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
    gold: Option<ScopeOrValue>,
    prestige: Option<ScopeOrValue>,
    piety: Option<ScopeOrValue>,
}

impl Validate for DecisionCost {
    fn from_scope(scope: Scope, id: &str) -> Result<Self, ValidationError> {
        let mut vd = Validator::new(&scope, id);

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
