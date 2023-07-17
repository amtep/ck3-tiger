use std::fs::read;
use std::path::{Path, PathBuf};

use anyhow::Result;
use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};
use fnv::FnvHashMap;

use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::report::{error, report, ErrorKey, Severity};
use crate::token::{Loc, Token};

#[derive(Clone, Debug, Default)]
pub struct Sounds {
    sounds: FnvHashMap<String, Sound>,
}

impl Sounds {
    pub fn load_item(&mut self, key: Token) {
        if let Some(other) = self.sounds.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "sound");
            }
        }
        self.sounds.insert(key.to_string(), Sound::new(key));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.sounds.contains_key(key)
    }

    pub fn verify_exists_implied(
        &self,
        key: &str,
        item: &Token,
        data: &Everything,
        max_sev: Severity,
    ) {
        if let Some(file) = key.strip_prefix("file:/") {
            data.verify_exists_implied(Item::File, file, item);
        } else if !self.sounds.contains_key(key) {
            let msg = if key == item.as_str() {
                "sound not defined in sounds/GUIDs.txt".to_string()
            } else {
                format!("sound {key} not defined in sounds/GUIDs.txt")
            };
            let info = "this could be due to a missing DLC";
            let sev = Item::Sound.severity().at_most(max_sev);
            report(ErrorKey::MissingSound, sev).msg(msg).info(info).loc(item).push();
        }
    }

    #[allow(clippy::unused_self)] // want to have a normal .validate call in Everything
    pub fn validate(&self, _data: &Everything) {}
}

impl FileHandler<String> for Sounds {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("sound")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<String> {
        if entry.path() != PathBuf::from("sound/GUIDs.txt") {
            return None;
        }
        match read_guids(fullpath) {
            Ok(content) => Some(content),
            Err(e) => {
                let msg = format!("could not read file: {e:#}");
                error(entry, ErrorKey::ReadError, &msg);
                None
            }
        }
    }

    fn handle_file(&mut self, entry: &FileEntry, content: String) {
        let mut linenr = 1;
        for line in content.lines() {
            let mut loc = Loc::for_entry(entry);
            loc.line = linenr;
            loc.column = 1;
            let token = Token::new(line, loc);
            if let Some((_guid, sound)) = token.split_once(' ') {
                self.load_item(sound);
            } else {
                error(token, ErrorKey::ParseError, "could not parse sound guid");
            }
            linenr += 1;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sound {
    key: Token,
}

impl Sound {
    pub fn new(key: Token) -> Self {
        Sound { key }
    }
}

fn read_guids(fullpath: &Path) -> Result<String> {
    let bytes = read(fullpath)?;
    WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).map_err(anyhow::Error::msg)
}
