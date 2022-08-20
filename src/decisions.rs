use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, will_log, LogPauseRaii};
use crate::everything::{FileEntry, FileHandler, FileKind};
use crate::pdxfile::PdxFile;
use crate::scope::{Comparator, Scope, ScopeOrValue, Token};

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: FnvHashMap<String, Decision>,

    // These decisions are known to exist, so don't warn abour them not being found,
    // but they had errors on validation.
    error_decisions: FnvHashMap<String, Token>,
}

impl Decisions {
    pub fn load_decision(&mut self, key: Token, scope: &Scope, values: Vec<(Token, Token)>) {
        if let Some(other) = self.decisions.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                if will_log(&key, ErrorKey::Duplicate) {
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
        }
        self.decisions
            .insert(key.to_string(), Decision::new(key, scope.clone(), values));
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
                        self.load_decision(key.clone(), s, decision_values.clone())
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
pub struct Decision {
    key: Token,
    scope: Scope,
    values: Vec<(Token, Token)>,
}

impl Decision {
    pub fn new(key: Token, scope: Scope, values: Vec<(Token, Token)>) -> Self {
        Decision { key, scope, values }
    }
}
