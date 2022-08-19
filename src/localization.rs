use fnv::{FnvHashMap, FnvHashSet};
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::errors::{advice_info, error, error_info, warn, warn_info, ErrorKey};
use crate::everything::{FileEntry, FileHandler, FileKind};
use crate::localization::parse::parse_loca;
use crate::scope::{Scope, Token};

mod parse;

#[derive(Clone, Debug)]
pub struct Localization {
    check_langs: Vec<&'static str>,
    warned_dirs: Vec<String>,
    locas: FnvHashMap<&'static str, FnvHashMap<String, LocaEntry>>,
}

// LAST UPDATED VERSION 1.6.2.2
pub const KNOWN_LANGUAGES: [&str; 7] = [
    "english",
    "spanish",
    "french",
    "german",
    "russian",
    "korean",
    "simp_chinese",
];

// LAST UPDATED VERSION 1.6.2.2
// These are just the ones that can't be deduced from the vanilla localization files.
pub const BUILTIN_MACROS: [&str; 2] = ["TRIGGER_AND", "TRIGGER_OR"];

#[derive(Clone, Debug)]
pub struct LocaEntry {
    key: Token,
    value: LocaValue,
}

#[derive(Clone, Debug, Default)]
pub enum LocaValue {
    // If the LocaValue is a Macro type, then it should be re-parsed after the macro values
    // have been filled in. Some macro values are supplied at runtime and we'll have to guess
    // at those.
    Macro(Vec<MacroValue>),
    Concat(Vec<LocaValue>),
    Text(Token),
    Markup(Token),
    MarkupEnd(Token),
    // The optional token is the formatting
    // TODO: convert [topic|E] code to something else than Code
    Code(CodeChain, Option<Token>),
    Icon(Token),
    #[default]
    Error,
}

#[derive(Clone, Debug)]
pub enum MacroValue {
    Text(Token),
    // The optional token is the formatting
    Keyword(Token, Option<Token>),
}

#[derive(Clone, Debug)]
pub struct CodeChain {
    // "codes" is my name for the things separated by dots in gui functions.
    // They may be "scopes", "promotes", or "functions" according to the game.
    // I don't understand the difference well enough yet to parse them that way.
    codes: Vec<Code>,
}

// Most "codes" are just a name followed by another dot or by the end of the code section.
// Some have arguments, which can be single-quoted strings, or other code chains.
// There is apparently a limit of two arguments per call, but we parse more so we can
// warn about that.
#[derive(Clone, Debug)]
pub struct Code {
    name: Token,
    arguments: Vec<CodeArg>,
}

// Possibly the literals can themselves contain [ ] code blocks.
// I'll have to test that.
#[derive(Clone, Debug)]
pub enum CodeArg {
    Chain(CodeChain),
    Literal(Token),
}

fn get_file_lang(filename: &OsStr) -> Option<&'static str> {
    for lang in KNOWN_LANGUAGES {
        // Deliberate discrepancy here between the check and the error msg below.
        // `l_{}.yml` works, but `_l_{}.yml` is still recommended.
        //
        // Using to_string_lossy is ok here because non-unicode sequences will
        // never match the suffix anyway.
        if filename
            .to_string_lossy()
            .ends_with(&format!("l_{}.yml", lang))
        {
            return Some(lang);
        }
    }
    None
}

impl FileHandler for Localization {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("localization")
    }

    fn config(&mut self, config: &Scope) {
        let mut langs: Vec<&str> = Vec::new();

        if let Some(scope) = config.get_field_scope("languages") {
            let check = scope.get_field_values("check");
            let skip = scope.get_field_values("skip");
            for lang in &KNOWN_LANGUAGES {
                if check.iter().any(|t| t.as_str() == *lang)
                    || (check.is_empty() && skip.iter().all(|t| t.as_str() != *lang))
                {
                    langs.push(lang);
                }
            }
            self.check_langs = langs;
        }
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        let depth = entry.path().components().count();
        assert!(depth >= 2);
        assert!(entry.path().starts_with("localization"));
        if entry.filename().to_string_lossy().ends_with(".info") {
            return;
        }

        // unwrap is safe here because we're only handed files under localization/
        // to_string_lossy is ok because we compare lang against a set of known strings.
        let lang = entry
            .path()
            .components()
            .nth(1)
            .unwrap()
            .as_os_str()
            .to_string_lossy();
        let mut warned = false;

        if depth == 2 {
            advice_info(
                &Token::from(entry),
                ErrorKey::Filename,
                "file in wrong location",
                "Localization files should be in subdirectories according to their language.",
            );
            warned = true;
        } else if !KNOWN_LANGUAGES.contains(&&*lang) {
            if self.warned_dirs.iter().any(|d| *d == *lang) {
                warn_info(
                    &Token::from(entry),
                    ErrorKey::Filename,
                    "unknown subdirectory in localization",
                    &format!("Valid subdirectories are {}", KNOWN_LANGUAGES.join(", ")),
                );
            }
            self.warned_dirs.push(lang.to_string());
            warned = true;
        }

        if KNOWN_LANGUAGES.contains(&&*lang) && !self.check_langs.contains(&&*lang) {
            return;
        }

        if let Some(filelang) = get_file_lang(entry.filename()) {
            if !self.check_langs.contains(&filelang) {
                return;
            }
            if filelang != lang && !warned {
                advice_info(&Token::from(entry), ErrorKey::Filename, "localization file with wrong name or in wrong directory", "A localization file should be in a subdirectory corresponding to its language.");
            }
            match read_to_string(fullpath) {
                Ok(content) => {
                    for loca in parse_loca(entry.path(), entry.kind(), &content) {
                        let hash = self.locas.entry(filelang).or_default();
                        if hash.contains_key(loca.key.as_str())
                            && hash.get(loca.key.as_str()).unwrap().key.loc.kind == entry.kind()
                        {
                            // TODO: show where the other key is
                            warn(
                                &loca.key,
                                ErrorKey::LocalizationDup,
                                "This localization key redefines an existing key",
                            );
                            warn(
                                &hash.get(loca.key.as_str()).unwrap().key,
                                ErrorKey::LocalizationDup,
                                "-- the other key is here.",
                            );
                        }
                        hash.insert(loca.key.as_str().to_string(), loca);
                    }
                }
                Err(e) => eprintln!("{:#}", e),
            }
        } else {
            error_info(
               &Token::from(entry),
               ErrorKey::Filename,
               "could not determine language from filename",
               &format!("Localization filenames should end in _l_language.yml, where language is one of {}", KNOWN_LANGUAGES.join(", "))
            );
        }
    }

    /// Do checks that can only be done after having all of the loca values
    fn finalize(&mut self) {
        // Does every macro use refer to a defined key?
        // First build the list of builtin macros by just checking which ones vanilla uses.
        let mut builtins = FnvHashSet::default();
        for lang in self.locas.values() {
            for entry in lang.values() {
                if entry.key.loc.kind != FileKind::VanillaFile {
                    continue;
                }

                if let LocaValue::Macro(ref v) = entry.value {
                    for macrovalue in v {
                        if let MacroValue::Keyword(k, _) = macrovalue {
                            if k.as_str()
                                .chars()
                                .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
                            {
                                builtins.insert(k.as_str());
                            }
                        }
                    }
                }
            }
        }

        for lang in self.locas.values() {
            for entry in lang.values() {
                if entry.key.loc.kind == FileKind::VanillaFile {
                    // Only warn for mod files
                    continue;
                }

                if let LocaValue::Macro(ref v) = entry.value {
                    for macrovalue in v {
                        if let MacroValue::Keyword(k, _) = macrovalue {
                            if !lang.contains_key(k.as_str())
                                && !builtins.contains(k.as_str())
                                && !BUILTIN_MACROS.contains(&k.as_str())
                            {
                                // TODO: display these errors in a sensible order, like by filename
                                error(&k, ErrorKey::Localization, &format!("The substitution parameter ${}$ is not defined anywhere as a key.", k.as_str()));
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for Localization {
    fn default() -> Self {
        Localization {
            check_langs: Vec::from(KNOWN_LANGUAGES),
            warned_dirs: Vec::default(),
            locas: FnvHashMap::default(),
        }
    }
}
