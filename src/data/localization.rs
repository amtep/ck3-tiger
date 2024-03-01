//! Validate `.yml` localization files

use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use fnv::{FnvHashMap, FnvHashSet};
use rayon::scope;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::datatype::{validate_datatypes, CodeChain, Datatype};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
#[cfg(any(feature = "ck3", feature = "vic3"))]
use crate::game::Game;
use crate::helpers::{dup_error, stringify_list};
use crate::item::Item;
use crate::macros::{MacroMapIndex, MACRO_MAP};
use crate::parse::localization::{parse_loca, ValueParser};
use crate::report::{
    err, report, warn, warn_abbreviated, warn_header, will_maybe_log, ErrorKey, Severity,
};
use crate::scopes::Scopes;
use crate::token::Token;

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
    locas: FnvHashMap<&'static str, FnvHashMap<String, LocaEntry>>,
    /// Which localization keys have been "used" (looked up) by the rest of the mod.
    /// This is used to print out the unused ones if requested.
    keys_used: RwLock<FnvHashSet<String>>,
    /// Which localization keys have been validated via [`Localization::validate_use`].
    /// `validate_use` takes a [`ScopeContext`], so this field is used to avoid re-validating those
    /// keys with less information during the general validation pass.
    keys_validated_with_sc: RwLock<FnvHashSet<String>>,
}

/// List of languages that are supported by the game engine.
// LAST UPDATED CK3 VERSION 1.12.1
// LAST UPDATED VIC3 VERSION 1.6.0
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
    #[cfg(feature = "vic3")]
    "polish",
    #[cfg(feature = "vic3")]
    "turkish",
];

/// List of known built-in keys used between `$...$` in any localization.
/// This list is used to avoid reporting false positives.
/// The [`Localization`] module also does a scan of vanilla localization values to see which
/// all-uppercase keys are used, and adds them to the list here.
// LAST UPDATED CK3 VERSION 1.9.2
// TODO: an updated version of this list would be very long and it's not clear what the benefit is,
// considering that there is also the runtime scan.
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

/// One parsed key: value line from the localization values.
#[derive(Clone, Debug)]
pub struct LocaEntry {
    key: Token,
    value: LocaValue,
    /// The original unparsed value, with enclosing `"` stripped.
    /// This is used for macro replacement.
    orig: Option<Token>,
}

impl LocaEntry {
    pub fn new(key: Token, value: LocaValue, orig: Option<Token>) -> Self {
        Self { key, value, orig }
    }

    // returns false to abort expansion in case of an error
    fn expand_macros<'a>(
        &'a self,
        vec: &mut Vec<Token>,
        from: &'a FnvHashMap<String, LocaEntry>,
        count: &mut usize,
        used: &mut FnvHashSet<String>,
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
                    MacroValue::Text(ref token) => vec.push(token.clone().linked(link)),
                    MacroValue::Keyword(k, _) => {
                        used.insert(k.to_string());
                        if let Some(entry) = from.get(k.as_str()) {
                            if !entry.expand_macros(
                                vec,
                                from,
                                count,
                                used,
                                Some(MACRO_MAP.get_or_insert_loc(k.loc)),
                            ) {
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
    Text(Token),
    Markup(Token),
    MarkupEnd(Token),
    Tooltip(Token),
    // Tag, key, value. Tag can influence how tooltip is looked up. If tag is `GAME_TRAIT`,
    // tooltip is a trait name and value is a character id. Any of the tokens may be a datatype
    // expression, which is passed through unparsed here.
    // TODO: instead of Token here, maybe need Box<LocaValue> or a Vec<LocaValue>, or maybe a type
    // that's specifically "Token or CodeChain"
    ComplexTooltip(Token, Token, Option<Token>),
    // The optional token is the formatting
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
        for lang in &self.mod_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(key) {
                return false;
            }
        }
        true
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.mod_langs
            .iter()
            .filter_map(|lang| self.locas.get(lang))
            .flat_map(|hash| hash.values().map(|item| &item.key))
    }

    #[cfg(not(feature = "imperator"))]
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
        let mut langs: Vec<&str> = Vec::new();
        for lang in &self.mod_langs {
            let hash = self.locas.get(lang);
            if hash.is_none() || !hash.unwrap().contains_key(name.as_str()) {
                langs.push(lang);
            }
        }
        if !langs.is_empty() {
            // It's merely untidy if the name is only missing in latin-script languages and the
            // name doesn't have indicators that it really needs to be localized (such as underscores
            // or extra uppercase letters). In all other cases it's a warning.
            //
            // TODO: this logic assumes the input name is in English and it doesn't consider for example
            // a Russian mod that only supports Russian localization and has names in Cyrillic.
            let sev = if only_latin_script(&langs)
                && !name.as_str().contains('_')
                && normal_capitalization_for_name(name.as_str())
            {
                Severity::Untidy
            } else {
                Severity::Warning
            };

            let msg = format!("missing {} localization for name {name}", stringify_list(&langs));
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
        self.keys_used.write().unwrap().insert(key.to_string());
    }

    // Does every `[concept|E]` reference have a defined game concept?
    // Does every other `[code]` block have valid promotes and functions?
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
                    if let Some(ref format) = format {
                        if format.as_str().contains('E') || format.as_str().contains('e') {
                            if let Some(name) = chain.as_gameconcept() {
                                data.verify_exists(Item::GameConcept, name);
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
                data.localization.verify_exists_lang(token, lang);
            }
            #[allow(unused_variables)] // tag only used by ck3
            LocaValue::ComplexTooltip(tag, token, _) => {
                // TODO: if any of the three are datatype expressions, validate them.
                #[cfg(feature = "ck3")]
                if Game::is_ck3() && !token.starts_with("[") && !token.starts_with("$") {
                    // The list of tag types can be found in ck3
                    // localization/english/tooltip_structs_l_english.yml
                    // LAST UPDATED CK3 VERSION 1.11.3
                    match &*tag.as_str().to_lowercase() {
                        "accolade"
                        | "activity"
                        | "army"
                        | "character"
                        | "char_context_tooltip"
                        | "scheme"
                        | "secret"
                        | "travel_plan" => (), // runtime id
                        "game_concept" => data.verify_exists(Item::GameConcept, token),
                        "culture" | "culture_innovation" | "culture_era" => {
                            data.verify_exists(Item::Culture, token);
                        }
                        "faith" => data.verify_exists(Item::Faith, token),
                        "religion" => data.verify_exists(Item::Religion, token),
                        "religion_family" => data.verify_exists(Item::ReligionFamily, token),
                        "game_trait" => data.verify_exists(Item::Trait, token),
                        "men_at_arms_type" => data.verify_exists(Item::MenAtArmsBase, token),
                        "specific_men_at_arms_type" => {
                            data.verify_exists(Item::MenAtArms, token);
                        }
                        "dynasty" => data.verify_exists(Item::Dynasty, token),
                        "dynasty_house" => data.verify_exists(Item::House, token),
                        "building" => data.verify_exists(Item::Building, token),
                        "faction" => data.verify_exists(Item::Faction, token),
                        "title" => data.verify_exists(Item::Title, token),
                        "government_type" => data.verify_exists(Item::GovernmentType, token),
                        // TODO: Verify scaled modifier has `scale`
                        "static_modifier" | "scaled_static_modifier" => {
                            data.verify_exists(Item::Modifier, token);
                        }
                        "law" => data.verify_exists(Item::Law, token),
                        "terrain" => data.verify_exists(Item::Terrain, token),
                        "game_faith_doctrine" => data.verify_exists(Item::Doctrine, token),
                        "lifestyle" => data.verify_exists(Item::Lifestyle, token),
                        "focus" => data.verify_exists(Item::Focus, token),
                        "perk" => data.verify_exists(Item::Perk, token),
                        "dynasty_perk" => data.verify_exists(Item::DynastyPerk, token),
                        // TODO "obligation_level", TODO: contract type?
                        "holding" => data.verify_exists(Item::HoldingType, token),
                        "secret_type" => data.verify_exists(Item::Secret, token),
                        "geographical_region" => data.verify_exists(Item::Region, token),
                        "culture_pillar" => data.verify_exists(Item::CulturePillar, token),
                        "culture_tradition" => {
                            data.verify_exists(Item::CultureTradition, token);
                        }
                        "inspiration" => data.verify_exists(Item::Inspiration, token),
                        "court_type" => data.verify_exists(Item::CourtType, token),
                        "artifact" => data.verify_exists(Item::ArtifactType, token),
                        "court_position_type" => data.verify_exists(Item::CourtPosition, token),
                        "scheme_type" => data.verify_exists(Item::Scheme, token),
                        // TODO "court_amenities_setting" => { data.verify_exists(Item::CourtAmenitiesSetting, token); }
                        "nickname" => data.verify_exists(Item::Nickname, token),
                        "struggle" => data.verify_exists(Item::Struggle, token),
                        "struggle_phase" => data.verify_exists(Item::StrugglePhase, token),
                        "activity_type" => data.verify_exists(Item::ActivityType, token),
                        "vassal_stance" => data.verify_exists(Item::VassalStance, token),
                        // TODO "ai_personality" => data.verify_exists(Item::AiPersonality, token),
                        "accolade_type" => data.verify_exists(Item::AccoladeType, token),
                        "travel_option" => data.verify_exists(Item::TravelOption, token),
                        "house_unity_stage" => data.verify_exists(Item::HouseUnityStage, token),
                        "decision" => data.verify_exists(Item::Decision, token),
                        "tax_slot_obligation" => data.verify_exists(Item::TaxSlotObligation, token),
                        _ => {
                            // TODO: should this be validated with validate_localization_sc ? (remember to avoid infinite loops)
                            data.localization.verify_exists_lang(token, lang);
                        }
                    }
                }
                #[cfg(feature = "vic3")]
                if Game::is_vic3() && !token.starts_with("[") && !token.starts_with("$") {
                    data.localization.verify_exists_lang(token, lang);
                }
                // TODO: - imperator -
            }
            LocaValue::Icon(token) => {
                data.verify_exists(Item::TextIcon, token);
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

    pub fn validate_use(&self, key: &str, data: &Everything, sc: &mut ScopeContext) {
        self.keys_validated_with_sc.write().unwrap().insert(key.to_string());
        for lang in &self.mod_langs {
            if let Some(hash) = self.locas.get(lang) {
                if let Some(entry) = hash.get(key) {
                    Self::check_loca_code(&entry.value, data, sc, lang);
                }
            }
        }
    }

    // This is in pass2 to make sure all `keys_validated_with_sc` have been marked.
    pub fn validate_pass2(&self, data: &Everything) {
        scope(|s| {
            // Hold the lock for the whole validation loop, to avoid the overhead of re-acquiring it
            let already_validated = self.keys_validated_with_sc.read().unwrap();
            for (lang, hash) in &self.locas {
                for entry in hash.values() {
                    if !already_validated.contains(entry.key.as_str()) {
                        s.spawn(|_| {
                            let mut sc = ScopeContext::new_unrooted(Scopes::all(), &entry.key);
                            sc.set_strict_scopes(false);
                            Self::check_loca_code(&entry.value, data, &mut sc, lang);
                        });
                    }
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
                for (key, entry) in hash {
                    if !self.keys_used.read().unwrap().contains(key) {
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

    fn load_file(&self, entry: &FileEntry) -> Option<(&'static str, Vec<LocaEntry>)> {
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
                    return Some((filelang, parse_loca(entry, &content, filelang).collect()));
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

    /// Do checks that can only be done after having all of the loca values
    fn finalize(&mut self) {
        // Check that every macro use refers to a defined key.
        // First build the list of builtin macros by just checking which ones vanilla uses.
        // TODO: scan the character interactions, which can also define macros
        let mut builtins = FnvHashSet::default();
        builtins.extend(BUILTIN_MACROS);
        for lang in self.locas.values() {
            for entry in lang.values() {
                if !entry.key.loc.kind.counts_as_vanilla() {
                    continue;
                }

                if let LocaValue::Macro(ref v) = entry.value {
                    for macrovalue in v {
                        if let MacroValue::Keyword(k, _) = macrovalue {
                            if k.as_str()
                                .chars()
                                .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
                                && !lang.contains_key(k.as_str())
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
                                let msg = &format!("The substitution parameter ${k}$ is not defined anywhere as a key.");
                                warn(ErrorKey::Localization).msg(msg).loc(k).push();
                            }
                        }
                    }
                }
            }
        }

        // Now expand all the macro values we can, and re-parse them after expansion
        for lang in self.locas.values_mut() {
            let orig_lang = lang.clone();
            for entry in lang.values_mut() {
                if matches!(entry.value, LocaValue::Macro(_)) {
                    let mut count = 0;
                    let mut new_line: Vec<Token> = Vec::new();
                    if entry.expand_macros(
                        &mut new_line,
                        &orig_lang,
                        &mut count,
                        &mut self.keys_used.write().unwrap(),
                        None,
                    ) {
                        let new_line_as_ref = new_line.iter().collect();
                        let mut value = ValueParser::new(new_line_as_ref).parse_value();
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
            keys_used: RwLock::new(FnvHashSet::default()),
            keys_validated_with_sc: RwLock::new(FnvHashSet::default()),
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
