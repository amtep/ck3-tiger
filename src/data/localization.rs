use fnv::{FnvHashMap, FnvHashSet};
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::data::localization::parse::{parse_loca, ValueParser};
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, error_info, warn_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::token::Token;

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
    orig: Option<Token>, // original unparsed value, with enclosing " stripped
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

impl Localization {
    pub fn exists(&self, key: &str) -> bool {
        for lang in &self.check_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(key) {
                return false;
            }
        }
        true
    }

    pub fn verify_exists(&self, token: &Token) {
        self.verify_exists_implied(token.as_str(), token);
    }

    pub fn verify_exists_implied(&self, key: &str, token: &Token) {
        if key.is_empty() {
            return;
        }
        for lang in &self.check_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(key) {
                error(
                    token,
                    ErrorKey::MissingLocalization,
                    &format!("missing {} localization key {}", lang, key),
                );
            }
        }
    }

    fn check_game_concepts(&self, value: &LocaValue, data: &Everything) {
        match value {
            LocaValue::Concat(v) => {
                for value in v {
                    self.check_game_concepts(value, data);
                }
            }
            // TODO: The is_lowercase here is a heuristic. We should be checking that it's
            // not a known function like [GetGameVersionInfo]
            LocaValue::Code(chain, _)
                if chain.codes.len() == 1
                    && chain.codes[0].arguments.is_empty()
                    && chain.codes[0].name.as_str().chars().all(char::is_lowercase) =>
            {
                data.verify_exists(Item::GameConcept, &chain.codes[0].name);
            }
            _ => (),
        }
    }

    pub fn validate(&self, data: &Everything) {
        // Does every `[concept]` reference have a defined game concept?
        for hash in self.locas.values() {
            for entry in hash.values() {
                self.check_game_concepts(&entry.value, data);
            }
        }
    }
}

impl FileHandler for Localization {
    fn config(&mut self, config: &Block) {
        let mut langs: Vec<&str> = Vec::new();

        if let Some(block) = config.get_field_block("languages") {
            // TODO: warn if there are unknown languages in check or skip?
            let check = block.get_field_values("check");
            let skip = block.get_field_values("skip");
            for lang in &KNOWN_LANGUAGES {
                if check.iter().any(|t| t.is(*lang))
                    || (check.is_empty() && skip.iter().all(|t| !t.is(*lang)))
                {
                    langs.push(lang);
                }
            }
            self.check_langs = langs;
        }
    }

    fn subpath(&self) -> PathBuf {
        PathBuf::from("localization")
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
                entry,
                ErrorKey::Filename,
                "file in wrong location",
                "Localization files should be in subdirectories according to their language.",
            );
            warned = true;
        } else if !KNOWN_LANGUAGES.contains(&&*lang) {
            if self.warned_dirs.iter().any(|d| *d == *lang) {
                warn_info(
                    entry,
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
                advice_info(entry, ErrorKey::Filename, "localization file with wrong name or in wrong directory", "A localization file should be in a subdirectory corresponding to its language.");
            }
            match read_to_string(fullpath) {
                Ok(content) => {
                    for loca in parse_loca(entry.path(), entry.kind(), &content) {
                        let hash = self.locas.entry(filelang).or_default();
                        if let Some(other) = hash.get(loca.key.as_str()) {
                            if other.key.loc.kind == entry.kind() {
                                dup_error(&loca.key, &other.key, "localization");
                            }
                        }
                        hash.insert(loca.key.to_string(), loca);
                    }
                }
                Err(e) => eprintln!("{:#}", e),
            }
        } else {
            error_info(
               entry,
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
        // TODO: scan the character interactions, which can also define macros
        let mut builtins = FnvHashSet::default();
        builtins.extend(&BUILTIN_MACROS);
        for lang in self.locas.values() {
            for entry in lang.values() {
                if entry.key.loc.kind != FileKind::Vanilla {
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
                if let LocaValue::Macro(ref v) = entry.value {
                    for macrovalue in v {
                        if let MacroValue::Keyword(k, _) = macrovalue {
                            if !lang.contains_key(k.as_str()) && !builtins.contains(k.as_str()) {
                                // TODO: display these errors in a sensible order, like by filename
                                error(k, ErrorKey::Localization, &format!("The substitution parameter ${}$ is not defined anywhere as a key.", k.as_str()));
                            }
                        }
                    }
                }
            }
        }

        // Now expand all the macro values we can, and re-parse them after expansion
        for lang in self.locas.values_mut() {
            let orig_lang = lang.clone();
            for mut entry in lang.values_mut() {
                let mut recursion_count = 0u8;
                'outer: while let LocaValue::Macro(ref v) = entry.value {
                    let mut new_line: Vec<&Token> = Vec::new();
                    for macrovalue in v {
                        match macrovalue {
                            MacroValue::Text(token) => new_line.push(token),
                            MacroValue::Keyword(k, _) => {
                                if let Some(entry) = orig_lang.get(k.as_str()) {
                                    if let Some(orig) = &entry.orig {
                                        new_line.push(orig);
                                    } else {
                                        break 'outer;
                                    }
                                } else {
                                    break 'outer;
                                }
                            }
                        }
                    }
                    let mut value = ValueParser::new(new_line).parse_value();
                    entry.value = if value.len() == 1 {
                        std::mem::take(&mut value[0])
                    } else {
                        LocaValue::Concat(value)
                    };
                    recursion_count += 1;
                    if recursion_count >= 250 {
                        break;
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
