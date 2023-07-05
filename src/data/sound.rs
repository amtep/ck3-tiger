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
use crate::report::{error, warn_info, ErrorKey};
use crate::token::{Loc, Token};

#[derive(Clone, Debug, Default)]
pub struct Sounds {
    sounds: FnvHashMap<String, Sound>,
}

impl Sounds {
    pub fn load_item(&mut self, key: Token, guid: Token) {
        if let Some(other) = self.sounds.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "sound");
            }
        }
        self.sounds.insert(key.to_string(), Sound::new(key, guid));
    }

    pub fn exists(&self, key: &str) -> bool {
        self.sounds.contains_key(key)
    }

    pub fn verify_exists_implied(&self, key: &str, item: &Token, data: &Everything) {
        if let Some(file) = key.strip_prefix("file:/") {
            data.verify_exists_implied(Item::File, file, item);
        } else if !self.sounds.contains_key(key) {
            let msg = if key == item.as_str() {
                "sound not defined in sounds/GUIDs.txt".to_string()
            } else {
                format!("sound {key} not defined in sounds/GUIDs.txt")
            };
            let info = "this could be due to a missing DLC";
            warn_info(item, ErrorKey::MissingSound, &msg, info);
        }
    }

    pub fn validate(&self, _data: &Everything) {}
}

impl FileHandler for Sounds {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("sound")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if entry.path() != PathBuf::from("sound/GUIDs.txt") {
            return;
        }
        let content = match read_guids(fullpath) {
            Ok(content) => content,
            Err(e) => {
                let msg = format!("could not read file: {e:#}");
                error(entry, ErrorKey::ReadError, &msg);
                return;
            }
        };

        let mut linenr = 1;
        for line in content.lines() {
            let mut loc = Loc::for_entry(entry);
            loc.line = linenr;
            loc.column = 1;
            let token = Token::new(line.to_string(), loc);
            if let Some((guid, sound)) = token.split_once(' ') {
                self.load_item(sound, guid);
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
    guid: Token,
}

impl Sound {
    pub fn new(key: Token, guid: Token) -> Self {
        Sound { key, guid }
    }
}

fn read_guids(fullpath: &Path) -> Result<String> {
    let bytes = read(fullpath)?;
    WINDOWS_1252
        .decode(&bytes, DecoderTrap::Strict)
        .map_err(anyhow::Error::msg)
}
