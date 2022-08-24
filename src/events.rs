use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::{Block, BlockOrValue, Comparator, Token};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, warn_info, LogPauseRaii};
use crate::everything::FileHandler;
use crate::fileset::{FileEntry, FileKind};
use crate::pdxfile::PdxFile;

#[derive(Clone, Debug, Default)]
pub struct Events {
    events: FnvHashMap<String, Event>,
    scripted_triggers: FnvHashMap<String, ScriptedTrigger>,
    scripted_effects: FnvHashMap<String, ScriptedEffect>,

    // These events are known to exist, so don't warn abour them not being found,
    // but they had errors on validation.
    error_events: FnvHashMap<String, Token>,
}

impl Events {
    pub fn load_event(&mut self, _key: Token, _block: &Block) {}
    pub fn load_scripted_trigger(&mut self, _key: Token, _block: &Block) {}
    pub fn load_scripted_effect(&mut self, _key: Token, _block: &Block) {}
}

impl FileHandler for Events {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("events")
    }

    fn config(&mut self, _config: &Block) {}

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        #[derive(Copy, Clone)]
        enum Expecting {
            Event,
            ScriptedTrigger,
            ScriptedEffect,
        }

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

        let mut namespaces = Vec::new();
        let mut expecting = Expecting::Event;

        for (k, cmp, v) in block.iter_items() {
            if let Some(key) = k {
                if !matches!(*cmp, Comparator::Eq) {
                    error(
                        key,
                        ErrorKey::Validation,
                        &format!("expected `{} =`, found `{}`", key, cmp),
                    );
                }
                if key.as_str() == "namespace" {
                    match v {
                        BlockOrValue::Token(t) => namespaces.push(t.as_str()),
                        BlockOrValue::Block(s) => error(
                            s,
                            ErrorKey::EventNamespace,
                            "expected namespace to have a simple string value",
                        ),
                    }
                } else {
                    match v {
                        BlockOrValue::Token(_) => error(
                            key,
                            ErrorKey::Validation,
                            "unknown setting in event files, expected only `namespace`",
                        ),
                        BlockOrValue::Block(s) => match expecting {
                            Expecting::ScriptedTrigger => {
                                self.load_scripted_trigger(key.clone(), s);
                                expecting = Expecting::Event;
                            }
                            Expecting::ScriptedEffect => {
                                self.load_scripted_effect(key.clone(), s);
                                expecting = Expecting::Event;
                            }
                            Expecting::Event => {
                                let mut namespace_ok = false;

                                if namespaces.is_empty() {
                                    error(
                                        key,
                                        ErrorKey::EventNamespace,
                                        "Event files must start with a namespace declaration",
                                    );
                                } else if let Some((key_a, key_b)) = key.as_str().split_once('.') {
                                    if key_b.chars().all(|c| c.is_ascii_digit()) {
                                        if namespaces.contains(&key_a) {
                                            namespace_ok = true;
                                        } else {
                                            warn_info(key, ErrorKey::EventNamespace, "Event name should start with namespace", "If the event doesn't match its namespace, the game can't properly find the event when triggering it.");
                                        }
                                    } else {
                                        warn_info(key, ErrorKey::EventNamespace, "Event names should be in the form NAMESPACE.NUMBER", "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of digits.");
                                    }
                                } else {
                                    warn_info(key, ErrorKey::EventNamespace, "Event names should be in the form NAMESPACE.NUMBER", "where NAMESPACE is the namespace declared at the top of the file, and NUMBER is a series of digits.");
                                }

                                if namespace_ok {
                                    self.load_event(key.clone(), s);
                                } else {
                                    self.error_events.insert(key.to_string(), key.clone());
                                }
                            }
                        },
                    }
                }
            } else {
                match v {
                    BlockOrValue::Token(t) => {
                        if matches!(expecting, Expecting::Event) && t.as_str() == "scripted_trigger"
                        {
                            expecting = Expecting::ScriptedTrigger;
                        } else if matches!(expecting, Expecting::Event)
                            && t.as_str() == "scripted_effect"
                        {
                            expecting = Expecting::ScriptedEffect;
                        } else {
                            error_info(
                                t,
                                ErrorKey::Validation,
                                "unexpected token",
                                "Did you forget an = ?",
                            );
                        }
                    }
                    BlockOrValue::Block(s) => error_info(
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
pub struct Event {}

#[derive(Clone, Debug)]
pub struct ScriptedTrigger {}

#[derive(Clone, Debug)]
pub struct ScriptedEffect {}
