use fnv::{FnvHashMap, FnvHashSet};
use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::datatype::{validate_datatypes, CodeChain, Datatype};
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, error_info, warn, warn2, warn_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::helpers::{dup_error, stringify_list};
use crate::item::Item;
use crate::parse::localization::{parse_loca, ValueParser};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct Localization {
    check_langs: Vec<&'static str>,
    warned_dirs: Vec<String>,
    locas: FnvHashMap<&'static str, FnvHashMap<String, LocaEntry>>,
    mod_langs: Vec<&'static str>,
    used_locas: RefCell<FnvHashSet<String>>,
}

// LAST UPDATED VERSION 1.9.2
pub const KNOWN_LANGUAGES: [&str; 7] = [
    "english",
    "spanish",
    "french",
    "german",
    "russian",
    "korean",
    "simp_chinese",
];

// LAST UPDATED VERSION 1.9.2
// Most are deduced from the vanilla localization files, but the known ones are
// hardcoded here.
pub const BUILTIN_MACROS: &[&str] = &[
    "ACTION",
    "ACTUAL_NEGATION",
    "ADJUSTMENTS",
    "BASE_NAME",
    "BATTLENAME",
    "BUDGET_CATEGORY",
    "BUDGET_GOLD",
    "BUDGET_MAXIMUM",
    "BUILDING_NAME",
    "CAP",
    "CASUALTIES",
    "CAUSE",
    "CHAR01",
    "CHAR02",
    "COMPANIONS",
    "COMPARATOR",
    "CONTROLLER",
    "DAY",
    "DLC_NAME",
    "DURATION_MIN",
    "DURATION_MAX",
    "ERRORS",
    "ERROR_ACTION",
    "EVENT",
    "EVENT_TITLE",
    "EXPENSE_DESC",
    "FERVOR",
    "FIRST",
    "INCOME_DESC",
    "INTERACTION",
    "MAX_LEVIES",
    "MAX_MEN_AT_ARMS",
    "MAX_NEGATION",
    "MAX_SUPPLY",
    "MEN_AT_ARMS",
    "MISSING_HOLDING",
    "MOD",
    "MONTH",
    "MONTH_SHORT",
    "MORE_RELATIONS",
    "MULT",
    "NUM",
    "PERSONALITY",
    "PING",
    "PREVIOUS_NAME",
    "ON_ACCEPT",
    "ON_DECLINE",
    "ON_SEND",
    "OTHER_TRAIT",
    "REGIMENTS",
    "REINFORCEMENTS",
    "RELATION01",
    "RELATION02",
    "SECOND",
    "TIER_KEY",
    "TRAIT",
    "TRAIT_AGE",
    "TRAIT_SEX",
    "TRIGGER_AND",
    "TRIGGER_OR",
    "VALUE",
    "WHAT",
    "WHO",
    "WINLOSE",
];

#[derive(Clone, Debug)]
pub struct LocaEntry {
    key: Token,
    value: LocaValue,
    orig: Option<Token>, // original unparsed value, with enclosing " stripped
}

impl LocaEntry {
    pub fn new(key: Token, value: LocaValue, orig: Option<Token>) -> Self {
        Self { key, value, orig }
    }

    // returns false to abort expansion in case of an error
    fn expand_macros<'a, 'b>(
        &'a self,
        vec: &mut Vec<&'a Token>,
        from: &'a FnvHashMap<String, LocaEntry>,
        count: &mut usize,
        used: &'b mut FnvHashSet<String>,
    ) -> bool {
        // Are we (probably) stuck in a macro loop?
        if *count > 250 {
            return false;
        }
        *count += 1;

        if let LocaValue::Macro(v) = &self.value {
            for macrovalue in v {
                match macrovalue {
                    MacroValue::Text(ref token) => vec.push(token),
                    MacroValue::Keyword(k, _) => {
                        used.insert(k.to_string());
                        if let Some(entry) = from.get(k.as_str()) {
                            if !entry.expand_macros(vec, from, count, used) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
            }
            true
        } else if let Some(orig) = &self.orig {
            vec.push(orig);
            true
        } else {
            false
        }
    }
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
    Tooltip(Token),
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

fn get_file_lang(filename: &OsStr) -> Option<&'static str> {
    // Deliberate discrepancy here between the check and the error msg below.
    // `l_{}.yml` works, but `_l_{}.yml` is still recommended.
    //
    // Using to_string_lossy is ok here because non-unicode sequences will
    // never match the suffix anyway.
    let filename = filename.to_string_lossy();
    KNOWN_LANGUAGES
        .into_iter()
        .find(|&lang| filename.ends_with(&format!("l_{lang}.yml")))
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
        self.mark_used(key);
        let mut langs: Vec<&str> = Vec::new();
        for lang in &self.mod_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(key) {
                langs.push(lang);
            }
        }
        if !langs.is_empty() {
            let msg = format!("missing {} localization key {key}", stringify_list(&langs));
            error(token, ErrorKey::MissingLocalization, &msg);
        }
    }

    pub fn exists_lang(&self, key: &str, lang: &'static str) -> bool {
        if lang.is_empty() {
            return self.exists(key);
        }
        let hash = self.locas.get(lang);
        if hash.is_none() || !hash.unwrap().contains_key(key) {
            return false;
        }
        true
    }

    pub fn verify_exists_lang(&self, token: &Token, lang: &'static str) {
        self.verify_exists_implied_lang(token.as_str(), token, lang);
    }

    pub fn verify_exists_implied_lang(&self, key: &str, token: &Token, lang: &'static str) {
        if key.is_empty() {
            return;
        }
        self.mark_used(key);
        if !self.exists_lang(key, lang) {
            let msg = format!("missing {lang} localization key {key}");
            error(token, ErrorKey::MissingLocalization, &msg);
        }
    }

    pub fn mark_used(&self, key: &str) {
        self.used_locas.borrow_mut().insert(key.to_string());
    }

    fn check_loca_code(value: &LocaValue, data: &Everything, lang: &'static str) {
        match value {
            LocaValue::Concat(v) => {
                for value in v {
                    Self::check_loca_code(value, data, lang);
                }
            }
            // A reference to a game concept
            LocaValue::Code(chain, Some(fmt))
                if fmt.as_str().contains('E') || fmt.as_str().contains('e') =>
            {
                if let Some(name) = chain.as_gameconcept() {
                    data.verify_exists(Item::GameConcept, name);
                } else {
                    let msg = format!("cannot figure out game concept for this |{fmt}");
                    warn(fmt, ErrorKey::ParseError, &msg);
                }
            }
            // Some other code
            // TODO: check the formatting codes
            LocaValue::Code(chain, _) => {
                validate_datatypes(chain, data, Datatype::Unknown, lang, false);
            }
            LocaValue::Tooltip(token) => {
                data.localization.verify_exists_lang(token, lang);
            }
            _ => (),
        }
    }

    pub fn verify_key_has_options(&self, key: &Token, n: i64) {
        for lang in &self.mod_langs {
            if let Some(hash) = self.locas.get(lang) {
                if let Some(entry) = hash.get(key.as_str()) {
                    if let Some(ref orig) = entry.orig {
                        for i in 1..=n {
                            let find = format!("$OPTION_{i}$");
                            if !orig.as_str().contains(&find) {
                                warn2(
                                    key,
                                    ErrorKey::Validation,
                                    &format!("localization is missing {find}"),
                                    &entry.key,
                                    "here",
                                );
                            }
                        }
                        let find = format!("$OPTION_{}$", n + 1);
                        if orig.as_str().contains(&find) {
                            warn2(
                                key,
                                ErrorKey::Validation,
                                "localization has too many options",
                                &entry.key,
                                "here",
                            );
                        }
                    } else if n > 0 {
                        warn2(
                            key,
                            ErrorKey::Validation,
                            "localization is missing $OPTION_1$",
                            &entry.key,
                            "here",
                        );
                    }
                }
            }
        }
    }

    pub fn validate(&self, data: &Everything) {
        // Does every `[concept|E]` reference have a defined game concept?
        // Does every other `[code]` block have valid promotes and functions?
        for (lang, hash) in &self.locas {
            for entry in hash.values() {
                Self::check_loca_code(&entry.value, data, lang);
            }
        }
    }

    pub fn check_unused(&self, _data: &Everything) {
        // Mark all the loading tips as used
        let mut tip = 0;
        loop {
            let loca = format!("LOADING_TIP_{tip}");
            if self.exists(&loca) {
                self.mark_used(&loca);
            } else {
                break;
            }
            tip += 1;
        }

        for lang in &self.mod_langs {
            if let Some(hash) = self.locas.get(lang) {
                let mut vec = Vec::new();
                for (key, entry) in hash.iter() {
                    if !self.used_locas.borrow().contains(key) {
                        vec.push(entry);
                    }
                }
                vec.sort_unstable_by_key(|entry| &entry.key.loc);
                for entry in vec {
                    let msg = "localization not used anywhere";
                    warn(&entry.key, ErrorKey::UnusedLocalization, &msg);
                }
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
                if check.iter().any(|t| t.is(lang))
                    || (check.is_empty() && skip.iter().all(|t| !t.is(lang)))
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
        } else if !KNOWN_LANGUAGES.contains(&&*lang) && lang != "replace" {
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

        if entry.kind() == FileKind::Mod && !self.mod_langs.contains(&&*lang) {
            for known in KNOWN_LANGUAGES {
                if known == lang {
                    self.mod_langs.push(known);
                }
            }
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
                    for loca in parse_loca(entry, &content, filelang) {
                        let hash = self.locas.entry(filelang).or_default();
                        if let Some(other) = hash.get(loca.key.as_str()) {
                            if other.key.loc.kind == entry.kind() && other.orig != loca.orig {
                                dup_error(&loca.key, &other.key, "localization");
                            }
                        }
                        hash.insert(loca.key.to_string(), loca);
                    }
                }
                Err(e) => eprintln!("{e:#}"),
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
        builtins.extend(BUILTIN_MACROS);
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
                if matches!(entry.value, LocaValue::Macro(_)) {
                    let mut count = 0;
                    let mut new_line: Vec<&Token> = Vec::new();
                    if entry.expand_macros(
                        &mut new_line,
                        &orig_lang,
                        &mut count,
                        &mut self.used_locas.borrow_mut(),
                    ) {
                        let mut value = ValueParser::new(new_line).parse_value();
                        entry.value = if value.len() == 1 {
                            std::mem::take(&mut value[0])
                        } else {
                            LocaValue::Concat(value)
                        };
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
            mod_langs: Vec::default(),
            used_locas: RefCell::new(FnvHashSet::default()),
        }
    }
}
