use fnv::FnvHashMap;
use std::ffi::{OsStr, OsString};
use std::fs::read_to_string;
use std::path::{Component, Path};

use crate::errors::{advice_info, error_info, warn, warn_info, ErrorKey};
use crate::everything::{FileEntry, FileHandler, FileKind};
use crate::localization::parse::parse_loca;
use crate::scope::Token;

mod parse;

#[derive(Clone, Debug, Default)]
pub struct Localization {
    warned_dirs: Vec<String>,
    locas: FnvHashMap<String, LocaEntry>,
}

// LAST UPDATED VERSION 1.6.2.2
const KNOWN_LANGUAGES: [&str; 7] = [
    "english",
    "spanish",
    "french",
    "german",
    "russian",
    "korean",
    "simp_chinese",
];

#[derive(Clone, Debug)]
pub struct LocaEntry {
    key: Token,
    value: LocaValue,
}

#[derive(Clone, Debug)]
pub enum LocaValue {
    Concat(Vec<LocaValue>),
    Text(Token),
    Error,
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
    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        let depth = entry.path().components().count();
        assert!(depth >= 2);
        assert!(entry.path().starts_with("localization"));
        if entry.kind() != FileKind::ModFile
            || entry.filename().to_string_lossy().ends_with(".info")
        {
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
        } else if lang != "replace" && !KNOWN_LANGUAGES.contains(&&*lang) {
            if self.warned_dirs.iter().any(|d| *d == *lang) {
                warn_info(
                    &Token::from(entry),
                    ErrorKey::Filename,
                    "unknown subdirectory in localization",
                    &format!(
                        "Valid subdirectories are {} and replace",
                        KNOWN_LANGUAGES.join(", ")
                    ),
                );
            }
            self.warned_dirs.push(lang.to_string());
            warned = true;
        }

        if let Some(filelang) = get_file_lang(entry.filename()) {
            if filelang != lang && lang != "replace" && !warned {
                advice_info(&Token::from(entry), ErrorKey::Filename, "localization file with wrong name or in wrong directory", "A localization file should be in a subdirectory corresponding to its language.");
            }
            let replace = entry
                .path()
                .components()
                .any(|c| c == Component::Normal(&OsString::from("replace")));
            match read_to_string(fullpath) {
                Ok(content) => {
                    for loca in parse_loca(entry.path(), entry.kind(), &content) {
                        if self.locas.contains_key(loca.key.as_str()) && !replace {
                            warn(&loca.key, ErrorKey::Localization, "This localization key redefines an existing key, but is not in a replace/ subdirectory.");
                        }
                        self.locas.insert(loca.key.as_str().to_string(), loca);
                    }
                }
                Err(e) => eprintln!("{:#}", e),
            }
        } else {
            error_info(&Token::from(entry), ErrorKey::Filename, "could not determine language from filename", &format!("Localization filenames should end in _l_language.yml, where language is one of {}", KNOWN_LANGUAGES.join(", ")));
        }
    }
}
