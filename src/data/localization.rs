use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use fnv::{FnvHashMap, FnvHashSet};

use crate::block::Block;
use crate::datatype::{validate_datatypes, CodeChain, Datatype};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::helpers::{dup_error, stringify_list};
use crate::item::Item;
use crate::parse::localization::{parse_loca, ValueParser};
#[cfg(feature = "ck3")]
use crate::report::warn2;
use crate::report::{
    error, error_info, old_warn, report, warn, warn_abbreviated, warn_header, warn_info,
    will_maybe_log, ErrorKey, Severity,
};
use crate::token::Token;

#[derive(Debug)]
pub struct Localization {
    check_langs: Vec<&'static str>,
    locas: FnvHashMap<&'static str, FnvHashMap<String, LocaEntry>>,
    mod_langs: Vec<&'static str>,
    used_locas: RwLock<FnvHashSet<String>>,
}

// LAST UPDATED VERSION 1.9.2
pub const KNOWN_LANGUAGES: &[&str] = &[
    "english",
    "spanish",
    "french",
    "german",
    "russian",
    "korean",
    "simp_chinese",
    #[cfg(feature = "vic3")]
    "braz_por",
    #[cfg(feature = "vic3")]
    "japanese",
    #[cfg(feature = "vic3")]
    "polish",
    #[cfg(feature = "vic3")]
    "turkish",
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
    fn expand_macros<'a>(
        &'a self,
        vec: &mut Vec<&'a Token>,
        from: &'a FnvHashMap<String, LocaEntry>,
        count: &mut usize,
        used: &mut FnvHashSet<String>,
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
    // `l_{}` anywhere in the filename works, but `_l_{}.yml` is still recommended.
    //
    // Using to_string_lossy is ok here because non-unicode sequences will
    // never match the suffix anyway.
    let filename = filename.to_string_lossy();
    KNOWN_LANGUAGES.iter().find(|&lang| filename.contains(&format!("l_{lang}"))).copied()
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
        self.verify_exists_implied(token.as_str(), token, Severity::Error);
    }

    pub fn verify_exists_implied(&self, key: &str, token: &Token, max_sev: Severity) {
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
            // TODO: get confidence level from caller
            report(ErrorKey::MissingLocalization, Item::Localization.severity().at_most(max_sev))
                .msg(msg)
                .loc(token)
                .push();
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
            // TODO: get confidence level from caller
            warn(ErrorKey::MissingLocalization).msg(msg).loc(token).push();
        }
    }

    pub fn mark_used(&self, key: &str) {
        self.used_locas.write().unwrap().insert(key.to_string());
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
                    old_warn(fmt, ErrorKey::ParseError, &msg);
                }
            }
            // Some other code
            // TODO: validate the formatting codes
            LocaValue::Code(chain, _) => {
                // TODO: datatype is not really Unknown here, it should be a CString or CFixedPoint or some kind of number.
                // But we can't express that yet.
                validate_datatypes(chain, data, Datatype::Unknown, lang, false);
            }
            LocaValue::Tooltip(token) => {
                data.localization.verify_exists_lang(token, lang);
            }
            _ => (),
        }
    }

    #[cfg(feature = "ck3")]
    pub fn verify_key_has_options(&self, loca: &str, key: &Token, n: i64, prefix: &str) {
        for lang in &self.mod_langs {
            if let Some(hash) = self.locas.get(lang) {
                if let Some(entry) = hash.get(loca) {
                    if let Some(ref orig) = entry.orig {
                        for i in 1..=n {
                            let find = format!("${prefix}{i}$");
                            let find2 = format!("${prefix}{i}|");
                            if !orig.as_str().contains(&find) && !orig.as_str().contains(&find2) {
                                warn2(
                                    key,
                                    ErrorKey::Validation,
                                    &format!("localization is missing {find}"),
                                    &entry.key,
                                    "here",
                                );
                            }
                        }
                        let find = format!("${prefix}{}$", n + 1);
                        let find2 = format!("${prefix}{}|", n + 1);
                        if orig.as_str().contains(&find) && !orig.as_str().contains(&find2) {
                            warn2(
                                key,
                                ErrorKey::Validation,
                                "localization has too many options",
                                &entry.key,
                                "here",
                            );
                        }
                    } else if n > 0 {
                        let msg = format!("localization is missing ${prefix}1$");
                        warn2(key, ErrorKey::Validation, &msg, &entry.key, "here");
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

    pub fn mark_category_used(&self, prefix: &str) {
        let mut i = 0;
        loop {
            let loca = format!("{prefix}{i}");
            if self.exists(&loca) {
                self.mark_used(&loca);
            } else {
                break;
            }
            i += 1;
        }
    }

    pub fn check_unused(&self, _data: &Everything) {
        self.mark_category_used("LOADING_TIP_");
        self.mark_category_used("HYBRID_NAME_FORMAT_");
        self.mark_category_used("DIVERGE_NAME_FORMAT_");

        for lang in &self.mod_langs {
            if let Some(hash) = self.locas.get(lang) {
                let mut vec = Vec::new();
                for (key, entry) in hash.iter() {
                    if !self.used_locas.read().unwrap().contains(key) {
                        vec.push(entry);
                    }
                }
                vec.sort_unstable_by_key(|entry| &entry.key.loc);
                let mut printed_header = false;
                for entry in vec {
                    if !printed_header && will_maybe_log(&entry.key, ErrorKey::UnusedLocalization) {
                        warn_header(
                            ErrorKey::UnusedLocalization,
                            &format!("Unused localization - {lang}:\n"),
                        );
                        printed_header = true;
                    }
                    warn_abbreviated(&entry.key, ErrorKey::UnusedLocalization);
                }
                if printed_header {
                    warn_header(ErrorKey::UnusedLocalization, "\n");
                }
            }
        }
    }

    #[cfg(feature = "ck3")]
    pub fn check_pod_loca(&self, data: &Everything) {
        for lang in &self.mod_langs {
            if let Some(hash) = self.locas.get(lang) {
                for key in data.database.iter_itype_flags(Item::PerkTree) {
                    let loca = format!("{key}_name");
                    if let Some(entry) = hash.get(&loca) {
                        if let LocaValue::Text(token) = &entry.value {
                            if token.as_str().ends_with("_visible") {
                                data.verify_exists(Item::ScriptedGui, token);
                                data.verify_exists(Item::Localization, token);
                            }
                            continue;
                        }
                    }
                    let msg = format!("missing loca `{key}_name: \"{key}_visible\"`");
                    let info = "this is needed for the `window_character_lifestyle.gui` code";
                    error_info(key, ErrorKey::PrincesOfDarkness, &msg, info);
                }
            }
        }
    }
}

impl FileHandler<(&'static str, Vec<LocaEntry>)> for Localization {
    fn config(&mut self, config: &Block) {
        let mut langs: Vec<&str> = Vec::new();

        if let Some(block) = config.get_field_block("languages") {
            // TODO: warn if there are unknown languages in check or skip?
            let check = block.get_field_values("check");
            let skip = block.get_field_values("skip");
            for lang in KNOWN_LANGUAGES {
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

    fn load_file(
        &self,
        entry: &FileEntry,
        fullpath: &Path,
    ) -> Option<(&'static str, Vec<LocaEntry>)> {
        let depth = entry.path().components().count();
        assert!(depth >= 2);
        assert!(entry.path().starts_with("localization"));
        if entry.filename().to_string_lossy().ends_with(".info") {
            return None;
        }

        // unwrap is safe here because we're only handed files under localization/
        // to_string_lossy is ok because we compare lang against a set of known strings.
        let lang = entry.path().components().nth(1).unwrap().as_os_str().to_string_lossy();

        if let Some(filelang) = get_file_lang(entry.filename()) {
            if !self.check_langs.contains(&filelang) {
                return None;
            }
            // Localization files don't have to be in a subdirectory corresponding to their language.
            // However, if there's one in a subdirectory for a *different* language than the one in its name,
            // then something is probably wrong.
            if filelang != lang && KNOWN_LANGUAGES.contains(&&*lang) {
                warn_info(entry, ErrorKey::Filename, "localization file with wrong name or in wrong directory", "A localization file should be in a subdirectory corresponding to its language.");
            }
            match read_to_string(fullpath) {
                Ok(content) => {
                    return Some((filelang, parse_loca(entry, &content, filelang).collect()));
                }
                Err(e) => eprintln!("{e:#}"),
            }
        } else if entry.kind() >= FileKind::Vanilla {
            // Check for `FileKind::Vanilla` because Jomini and Clausewitz support more languages
            error_info(
                entry,
                ErrorKey::Filename,
                "could not determine language from filename",
                &format!("Localization filenames should end in _l_language.yml, where language is one of {}", KNOWN_LANGUAGES.join(", ")),
            );
        }
        None
    }

    fn handle_file(&mut self, entry: &FileEntry, loaded: (&'static str, Vec<LocaEntry>)) {
        let (filelang, mut vec) = loaded;
        if entry.kind() == FileKind::Mod && !self.mod_langs.contains(&filelang) {
            for &known in KNOWN_LANGUAGES {
                if known == filelang {
                    self.mod_langs.push(known);
                }
            }
        }

        for loca in vec.drain(..) {
            let hash = self.locas.entry(filelang).or_default();
            if !is_replace_path(entry.path()) {
                if let Some(other) = hash.get(loca.key.as_str()) {
                    // other.key and loca.key are in the other order than usual here,
                    // because in loca the older definition overrides the later one.
                    if other.key.loc.kind == entry.kind() && other.orig != loca.orig {
                        dup_error(&other.key, &loca.key, "localization");
                        continue;
                    }
                }
            }
            hash.insert(loca.key.to_string(), loca);
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
                if entry.key.loc.kind > FileKind::Vanilla {
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
                        &mut self.used_locas.write().unwrap(),
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
            locas: FnvHashMap::default(),
            mod_langs: Vec::default(),
            used_locas: RwLock::new(FnvHashSet::default()),
        }
    }
}

/// It's been tested that localization/replace/english and localization/english/replace both work
fn is_replace_path(path: &Path) -> bool {
    for element in path {
        if element.to_string_lossy() == "replace" {
            return true;
        }
    }
    false
}
