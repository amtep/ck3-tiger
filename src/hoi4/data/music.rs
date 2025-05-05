use std::path::PathBuf;

use crate::block::{Block, Field};
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{report, warn, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;
use crate::variables::Variables;

#[derive(Clone, Debug, Default)]
pub struct Hoi4Musics {
    musics: TigerHashMap<&'static str, Music>,
}

impl Hoi4Musics {
    pub fn load_item(&mut self, key: Token, block: Block, station: Option<Token>) {
        if let Some(other) = self.musics.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "music");
            }
        }
        self.musics.insert(key.as_str(), Music { station, key, block });
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.musics.values() {
            registry.scan(&item.block);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        let dlc_music = crate::hoi4::tables::misc::DLC_MUSIC;
        self.musics.contains_key(key) || dlc_music.contains(&key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.musics.values().map(|item| &item.key)
    }

    pub fn verify_exists_implied(&self, key: &str, item: &Token, max_sev: Severity) {
        if !self.exists(key) {
            let msg = if key == item.as_str() {
                "music not defined in music/ or dlc/*/music/".to_string()
            } else {
                format!("music {key} not defined in music/ or dlc/*/music/")
            };
            let info = "this could be due to a missing DLC";
            report(ErrorKey::MissingSound, Item::Sound.severity().at_most(max_sev))
                .msg(msg)
                .info(info)
                .loc(item)
                .push();
        }
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.musics.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Hoi4Musics {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("music")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read_no_bom(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        let mut station = None;

        for item in block.drain() {
            if let Some(Field(key, _, bv)) = item.expect_into_field() {
                if key.is("music_station") {
                    station = bv.expect_into_value();
                } else if key.is("music") {
                    if let Some(block) = bv.expect_into_block() {
                        self.load_item(key, block, station.clone());
                    }
                } else {
                    let msg = "unexpected key";
                    let info = "expected only `music_station` or `music`";
                    warn(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Music {
    station: Option<Token>,
    key: Token,
    block: Block,
}

impl Music {
    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Country, &self.key);

        if let Some(station) = &self.station {
            let loca = format!("{station}_TITLE");
            data.verify_exists_implied(Item::Localization, &loca, station);
        } else {
            let msg = "music item with no preceding music_station";
            warn(ErrorKey::Validation).msg(msg).loc(&self.key).push();
        }

        vd.field_item("song", Item::MusicAsset);
        vd.field_validated_block_sc("chance", &mut sc, validate_modifiers_with_base);
    }
}
