//! Validate `.yml` localization files

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs::read_to_string;
#[cfg(any(feature = "ck3", feature = "vic3"))]
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

#[cfg(any(feature = "ck3", feature = "vic3"))]
use murmur3::murmur3_32;
use rayon::scope;

use crate::block::Block;
#[cfg(feature = "ck3")]
use crate::ck3::tables::localization::{BUILTIN_MACROS_CK3, COMPLEX_TOOLTIPS_CK3};
use crate::context::ScopeContext;
use crate::datatype::{CodeChain, Datatype, validate_datatypes};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::game::Game;
use crate::helpers::{TigerHashMap, dup_error, stringify_list};
#[cfg(feature = "imperator")]
use crate::imperator::tables::localization::BUILTIN_MACROS_IMPERATOR;
use crate::item::Item;
use crate::macros::{MACRO_MAP, MacroMapIndex};
use crate::parse::ParserMemory;
use crate::parse::localization::{ValueParser, parse_loca};
use crate::report::{
    ErrorKey, Severity, err, report, warn, warn_abbreviated, warn_header, will_maybe_log,
};
use crate::scopes::Scopes;
use crate::token::Token;
#[cfg(feature = "vic3")]
use crate::vic3::tables::localization::BUILTIN_MACROS_VIC3;

/// Database of all loaded localization keys and their values, for all supported languages.
#[derive(Debug)]
pub struct Localization {
    /// Which languages to check, according to the config file.
    check_langs: Vec<&'static str>,
    /// Which languages also actually exist in the mod.
    /// This is used to not warn about missing loca when a mod doesn't have the language at all.
    /// (This saves them the effort of configuring `check_langs`).
    mod_langs: Vec<&'static str>,
    /// Database of all localizations, indexed first by language and then by localization key.
    locas: TigerHashMap<&'static str, TigerHashMap<String, LocaEntry>>,
}

/// List of languages that are supported by the game engine.
// LAST UPDATED CK3 VERSION 1.14.0.2
// LAST UPDATED VIC3 VERSION 1.7.6
pub const KNOWN_LANGUAGES: &[&str] = &[
    "english",
    "spanish",
    "french",
    "german",
    "russian",
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    "korean",
    "simp_chinese",
    #[cfg(feature = "vic3")]
    "braz_por",
    #[cfg(feature = "vic3")]
    "japanese",
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    "polish",
    #[cfg(feature = "vic3")]
    "turkish",
];

/// List of known built-in keys used between `$...$` in any localization.
/// This list is used to avoid reporting false positives.
// TODO: maybe make the list more specific about which keys can contain which builtins
fn is_builtin_macro<S: Borrow<str>>(s: S) -> bool {
    let s = s.borrow();
    match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => BUILTIN_MACROS_CK3.contains(&s),
        #[cfg(feature = "vic3")]
        Game::Vic3 => BUILTIN_MACROS_VIC3.contains(&s),
        #[cfg(feature = "imperator")]
        Game::Imperator => BUILTIN_MACROS_IMPERATOR.contains(&s),
    }
}

/// One parsed key: value line from the localization values.
#[derive(Debug)]
pub struct LocaEntry {
    key: Token,
    value: LocaValue,
    /// The original unparsed value, with enclosing `"` stripped.
    /// This is used for macro replacement.
    orig: Option<Token>,
    /// Whether this entry has been "used" (looked up) by anything in the mod
    used: AtomicBool,
    /// Whether this entry has been validated with a `ScopeContext`
    validated: AtomicBool,
}

impl PartialEq for LocaEntry {
    fn eq(&self, other: &LocaEntry) -> bool {
        self.key.loc == other.key.loc
    }
}

impl Eq for LocaEntry {}

impl PartialOrd for LocaEntry {
    fn partial_cmp(&self, other: &LocaEntry) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LocaEntry {
    fn cmp(&self, other: &LocaEntry) -> Ordering {
        self.key.loc.cmp(&other.key.loc)
    }
}

impl LocaEntry {
    pub fn new(key: Token, value: LocaValue, orig: Option<Token>) -> Self {
        Self { key, value, orig, used: AtomicBool::new(false), validated: AtomicBool::new(false) }
    }

    // returns false to abort expansion in case of an error
    fn expand_macros<'a>(
        &'a self,
        vec: &mut Vec<Token>,
        from: &'a TigerHashMap<String, LocaEntry>,
        count: &mut usize,
        sc: &mut ScopeContext,
        link: Option<MacroMapIndex>,
    ) -> bool {
        // Are we (probably) stuck in a macro loop?
        if *count > 250 {
            return false;
        }
        *count += 1;

        if let LocaValue::Macro(v) = &self.value {
            for macrovalue in v {
                match macrovalue {
                    MacroValue::Text(token) => vec.push(token.clone().linked(link)),
                    MacroValue::Keyword(keyword) => {
                        if let Some(entry) = from.get(keyword.as_str()) {
                            entry.used.store(true, Relaxed);
                            entry.validated.store(true, Relaxed);
                            if !entry.expand_macros(
                                vec,
                                from,
                                count,
                                sc,
                                Some(MACRO_MAP.get_or_insert_loc(keyword.loc)),
                            ) {
                                return false;
                            }
                        } else if is_builtin_macro(keyword) {
                            // we can't know what value it really has, so replace it with itself to
                            // at least get comprehensible error messages
                            vec.push(keyword.clone().linked(link));
                        } else if let Some(scopes) = sc.is_name_defined(keyword.as_str()) {
                            if scopes.contains(Scopes::Value) {
                                // same as above... we can't know what value it really has
                                vec.push(keyword.clone().linked(link));
                            } else {
                                let msg = &format!(
                                    "The substitution parameter ${keyword}$ is not defined anywhere as a key."
                                );
                                warn(ErrorKey::Localization).msg(msg).loc(keyword).push();
                            }
                        } else {
                            let msg = &format!(
                                "The substitution parameter ${keyword}$ is not defined anywhere as a key."
                            );
                            warn(ErrorKey::Localization).msg(msg).loc(keyword).push();
                        }
                    }
                }
            }
            true
        } else if let Some(orig) = &self.orig {
            vec.push(orig.clone().linked(link));
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
    #[allow(dead_code)] // the Token is only used for ck3
    Text(Token),
    Markup,
    MarkupEnd,
    Tooltip(Token),
    // Tag, key, value. Tag can influence how tooltip is looked up. If tag is `GAME_TRAIT`,
    // tooltip is a trait name and value is a character id. Any of the tokens may be a datatype
    // expression, which is passed through unparsed here.
    // The value is not stored in the enum because we don't validate it.
    // TODO: instead of Token here, maybe need Box<LocaValue> or a Vec<LocaValue>, or maybe a type
    // that's specifically "Token or CodeChain"
    ComplexTooltip(Token, Token),
    // The optional token is the formatting
    Code(CodeChain, Option<Token>),
    Icon(Token),
    #[default]
    Error,
}

#[derive(Clone, Debug)]
pub enum MacroValue {
    Text(Token),
    // The formatting is not stored in the enum because it's not validated.
    Keyword(Token),
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
        for lang in &self.mod_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(key) {
                return false;
            }
        }
        true
    }

    // Undocumented; the hash algorithm was revealed by inspecting error.log and reverse
    // engineering of CK3 binary through magic numbers. CK3 and VIC3 are supported.
    #[cfg(any(feature = "ck3", feature = "vic3"))]
    fn all_collision_keys(&self, lang: &str) -> TigerHashMap<u32, Vec<&LocaEntry>> {
        let mut result: TigerHashMap<u32, Vec<&LocaEntry>> = TigerHashMap::default();
        for loca in self.locas[lang].values() {
            result
                .entry(murmur3_32(&mut Cursor::new(loca.key.as_str()), 0).unwrap())
                .or_default()
                .push(loca);
        }
        result.retain(|_, locas| locas.len() > 1);
        result
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.mod_langs
            .iter()
            .filter_map(|lang| self.locas.get(lang))
            .flat_map(|hash| hash.values().map(|item| &item.key))
    }

    pub fn verify_exists_implied(&self, key: &str, token: &Token, max_sev: Severity) {
        if key.is_empty() {
            return;
        }
        self.mark_used(key);
        let mut langs_missing: Vec<&str> = Vec::new();
        for lang in &self.mod_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(key) {
                langs_missing.push(lang);
            }
        }
        if !langs_missing.is_empty() {
            let msg = format!("missing {} localization key {key}", stringify_list(&langs_missing));
            // TODO: get confidence level from caller
            report(ErrorKey::MissingLocalization, Item::Localization.severity().at_most(max_sev))
                .msg(msg)
                .loc(token)
                .push();
        }
    }

    #[cfg(feature = "ck3")]
    pub fn verify_name_exists(&self, name: &Token, max_sev: Severity) {
        if name.as_str().is_empty() {
            report(ErrorKey::MissingLocalization, Severity::Warning.at_most(max_sev))
                .msg("empty name")
                .loc(name)
                .push();
            return;
        }
        self.mark_used(name.as_str());
        let mut langs_missing: Vec<&str> = Vec::new();
        for lang in &self.mod_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(name.as_str()) {
                langs_missing.push(lang);
            }
        }
        if !langs_missing.is_empty() {
            // It's merely untidy if the name is only missing in latin-script languages and the
            // name doesn't have indicators that it really needs to be localized (such as underscores
            // or extra uppercase letters). In all other cases it's a warning.
            //
            // TODO: this logic assumes the input name is in English and it doesn't consider for example
            // a Russian mod that only supports Russian localization and has names in Cyrillic.
            let sev = if only_latin_script(&langs_missing)
                && !name.as_str().contains('_')
                && normal_capitalization_for_name(name.as_str())
            {
                Severity::Untidy
            } else {
                Severity::Warning
            };

            let msg =
                format!("missing {} localization for name {name}", stringify_list(&langs_missing));
            report(ErrorKey::MissingLocalization, sev.at_most(max_sev))
                .strong()
                .msg(msg)
                .loc(name)
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
        if lang.is_empty() {
            self.verify_exists_implied(key, token, Severity::Warning);
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
        for lang in &self.mod_langs {
            if let Some(entry) = self.locas.get(lang).and_then(|hash| hash.get(key)) {
                entry.used.store(true, Relaxed);
            }
        }
    }

    // Does every `[concept|E]` reference have a defined game concept?
    // Does every other `[code]` block have valid promotes and functions?
    // Does every $key$ in a macro have a corresponding loca key or named scope?
    fn check_loca_code(
        value: &LocaValue,
        data: &Everything,
        sc: &mut ScopeContext,
        lang: &'static str,
    ) {
        match value {
            LocaValue::Concat(v) => {
                for value in v {
                    Self::check_loca_code(value, data, sc, lang);
                }
            }
            // TODO: validate the formatting codes
            LocaValue::Code(chain, format) => {
                // |E is the formatting used for game concepts in ck3
                #[cfg(feature = "ck3")]
                if Game::is_ck3() {
                    if let Some(format) = format {
                        if format.as_str().contains('E') || format.as_str().contains('e') {
                            if let Some(name) = chain.as_gameconcept() {
                                if !is_builtin_macro(name) {
                                    data.verify_exists(Item::GameConcept, name);
                                }
                                return;
                            }
                        }
                    }
                }

                // TODO: datatype is not really Unknown here, it should be a CString or CFixedPoint or some kind of number.
                // But we can't express that yet.
                validate_datatypes(
                    chain,
                    data,
                    sc,
                    Datatype::Unknown,
                    lang,
                    format.as_ref(),
                    false,
                );
            }
            LocaValue::Tooltip(token) => {
                // TODO: should this be validated with validate_localization_sc ? (remember to avoid infinite loops)
                if !(Game::is_vic3() && token.is("BREAKDOWN_TAG")) {
                    data.localization.verify_exists_lang(token, lang);
                }
            }
            #[allow(unused_variables)] // tag only used by ck3
            LocaValue::ComplexTooltip(tag, token) => {
                // TODO: if any of the three are datatype expressions, validate them.
                #[cfg(feature = "ck3")]
                if Game::is_ck3() && !token.starts_with("[") && !is_builtin_macro(token) {
                    match COMPLEX_TOOLTIPS_CK3.get(&*tag.as_str().to_lowercase()).copied() {
                        None => {
                            // TODO: should this be validated with validate_localization_sc ? (remember to avoid infinite loops)
                            data.localization.verify_exists_lang(token, lang);
                        }
                        Some(None) => (), // token is a runtime id
                        Some(Some(itype)) => data.verify_exists(itype, token),
                    }
                }
                #[cfg(feature = "vic3")]
                if Game::is_vic3() && !token.starts_with("[") && !is_builtin_macro(token) {
                    data.localization.verify_exists_lang(token, lang);
                }
                // TODO: - imperator -
            }
            LocaValue::Icon(token) => {
                if !is_builtin_macro(token) && !token.is("ICONKEY_icon") && !token.is("KEY_icon") {
                    data.verify_exists(Item::TextIcon, token);
                }
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
                                warn(ErrorKey::Validation)
                                    .msg(format!("localization is missing {find}"))
                                    .loc(key)
                                    .loc_msg(&entry.key, "here")
                                    .push();
                            }
                        }
                        let find = format!("${prefix}{}$", n + 1);
                        let find2 = format!("${prefix}{}|", n + 1);
                        if orig.as_str().contains(&find) && !orig.as_str().contains(&find2) {
                            warn(ErrorKey::Validation)
                                .msg("localization has too many options")
                                .loc(key)
                                .loc_msg(&entry.key, "here")
                                .push();
                        }
                    } else if n > 0 {
                        let msg = format!("localization is missing ${prefix}1$");
                        warn(ErrorKey::Validation)
                            .msg(msg)
                            .loc(key)
                            .loc_msg(&entry.key, "here")
                            .push();
                    }
                }
            }
        }
    }

    fn validate_loca(
        entry: &LocaEntry,
        from: &TigerHashMap<String, LocaEntry>,
        data: &Everything,
        sc: &mut ScopeContext,
        lang: &'static str,
    ) {
        if matches!(entry.value, LocaValue::Macro(_)) {
            let mut new_line = Vec::new();
            let mut count = 0;
            if entry.expand_macros(&mut new_line, from, &mut count, sc, None) {
                // re-parse after macro expansion
                let new_line_as_ref = new_line.iter().collect();
                let value = ValueParser::new(new_line_as_ref).parse();
                Self::check_loca_code(&value, data, sc, lang);
            }
        } else {
            Self::check_loca_code(&entry.value, data, sc, lang);
        }
    }

    pub fn validate_use(&self, key: &str, data: &Everything, sc: &mut ScopeContext) {
        for lang in &self.mod_langs {
            if let Some(hash) = self.locas.get(lang) {
                if let Some(entry) = hash.get(key) {
                    entry.used.store(true, Relaxed);
                    entry.validated.store(true, Relaxed);
                    Self::validate_loca(entry, hash, data, sc, lang);
                }
            }
        }
    }

    #[cfg(any(feature = "ck3", feature = "vic3"))]
    fn check_collisions(&self, lang: &str) {
        for (k, v) in self.all_collision_keys(lang) {
            let mut rep = report(ErrorKey::LocalizationKeyCollision, Severity::Error)
                .strong()
                .msg(format!(
                    "localization keys '{}' have same MURMUR3A hash '0x{k:08X}'",
                    stringify_list(&v.iter().map(|loca| loca.key.as_str()).collect::<Vec<&str>>())
                ))
                .info("localization keys hash collision will cause some of them fail to load")
                .loc(&v[0].key);
            for loc in v.iter().skip(1) {
                rep = rep.loc_msg(&loc.key, "here");
            }
            rep.push();
        }
    }

    // This is in pass2 to make sure all `validated` entries have been marked.
    pub fn validate_pass2(&self, data: &Everything) {
        scope(|s| {
            for (lang, hash) in &self.locas {
                // Check localization key collisions
                #[cfg(any(feature = "ck3", feature = "vic3"))]
                s.spawn(|_| self.check_collisions(lang));

                // Collect and sort the entries before looping, to create more stable output
                let mut unvalidated_entries: Vec<&LocaEntry> =
                    hash.values().filter(|e| !e.validated.load(Relaxed)).collect();
                unvalidated_entries.sort_unstable();
                for entry in unvalidated_entries {
                    // Technically we can now store true in entry.validated,
                    // but the value is not needed anymore after this.
                    s.spawn(|_| {
                        let mut sc = ScopeContext::new_unrooted(Scopes::all(), &entry.key);
                        sc.set_strict_scopes(false);
                        Self::validate_loca(entry, hash, data, &mut sc, lang);
                    });
                }
            }
        });
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
                for entry in hash.values() {
                    if !entry.used.load(Relaxed) {
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
                for key in data.database.iter_keys(Item::PerkTree) {
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
                    err(ErrorKey::PrincesOfDarkness).msg(msg).info(info).loc(key).push();
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
        _parser: &ParserMemory,
    ) -> Option<(&'static str, Vec<LocaEntry>)> {
        let depth = entry.path().components().count();
        assert!(depth >= 2);
        assert!(entry.path().starts_with("localization"));
        if !entry.filename().to_string_lossy().ends_with(".yml") {
            return None;
        }

        // unwrap is safe here because we're only handed files under localization/
        // to_string_lossy is ok because we compare lang against a set of known strings.
        let lang = entry.path().components().nth(1).unwrap().as_os_str().to_string_lossy();

        // special case for this file
        if lang == "languages.yml" {
            return None;
        }

        if let Some(filelang) = get_file_lang(entry.filename()) {
            if !self.check_langs.contains(&filelang) {
                return None;
            }
            // Localization files don't have to be in a subdirectory corresponding to their language.
            // However, if there's one in a subdirectory for a *different* language than the one in its name,
            // then something is probably wrong.
            if filelang != lang && KNOWN_LANGUAGES.contains(&&*lang) {
                let msg = "localization file with wrong name or in wrong directory";
                let info = "A localization file should be in a subdirectory corresponding to its language.";
                warn(ErrorKey::Filename).msg(msg).info(info).loc(entry).push();
            }
            match read_to_string(entry.fullpath()) {
                Ok(content) => {
                    return Some((filelang, parse_loca(entry, content, filelang).collect()));
                }
                Err(e) => {
                    let msg = "could not read file";
                    let info = &format!("{e:#}");
                    err(ErrorKey::ReadError).msg(msg).info(info).loc(entry).push();
                }
            }
        } else if entry.kind() >= FileKind::Vanilla {
            // Check for `FileKind::Vanilla` because Jomini and Clausewitz support more languages
            let msg = "could not determine language from filename";
            let info = format!(
                "Localization filenames should end in _l_language.yml, where language is one of {}",
                KNOWN_LANGUAGES.join(", ")
            );
            err(ErrorKey::Filename).msg(msg).info(info).loc(entry).push();
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
}

impl Default for Localization {
    fn default() -> Self {
        Localization {
            check_langs: Vec::from(KNOWN_LANGUAGES),
            locas: TigerHashMap::default(),
            mod_langs: Vec::default(),
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

/// These are the languages in which it's reasonable to present an ascii name unchanged.
#[cfg(feature = "ck3")]
const LATIN_SCRIPT_LANGS: &[&str] =
    &["english", "french", "german", "spanish", "braz_por", "polish", "turkish"];

/// Return true iff `langs` only contains languages in which it's reasonable to present an ascii
/// name unchanged.
#[cfg(feature = "ck3")]
fn only_latin_script(langs: &[&str]) -> bool {
    langs.iter().all(|lang| LATIN_SCRIPT_LANGS.contains(lang))
}

/// Check that the string only has capital letters at the start or after a space or hyphen
#[cfg(feature = "ck3")]
fn normal_capitalization_for_name(name: &str) -> bool {
    let mut expect_cap = true;
    for ch in name.chars() {
        if ch.is_uppercase() && !expect_cap {
            return false;
        }
        expect_cap = ch == ' ' || ch == '-';
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_latin_script() {
        let mut langs = vec!["english", "french", "german"];
        assert!(only_latin_script(&langs));
        langs.push("korean");
        assert!(!only_latin_script(&langs));
        langs.clear();
        assert!(only_latin_script(&langs));
    }

    #[test]
    fn test_normal_capitalization_for_name() {
        assert!(normal_capitalization_for_name("George"));
        assert!(normal_capitalization_for_name("george"));
        assert!(!normal_capitalization_for_name("BjOrn"));
        assert!(normal_capitalization_for_name("Jean-Claude"));
        assert!(normal_capitalization_for_name("Abu-l-Fadl al-Malik"));
        assert!(normal_capitalization_for_name("Abu Abdallah Muhammad"));
        assert!(!normal_capitalization_for_name("AbuAbdallahMuhammad"));
    }
}
