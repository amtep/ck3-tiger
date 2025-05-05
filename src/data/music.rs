use std::path::PathBuf;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::game::{Game, GameFlags};
use crate::helpers::{dup_error, TigerHashMap};
use crate::item::{Item, ItemLoader};
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{report, warn, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_optional_duration_int;
use crate::validator::Validator;
use crate::variables::Variables;

#[derive(Clone, Debug, Default)]
pub struct Musics {
    musics: TigerHashMap<&'static str, Music>,
}

impl Musics {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.musics.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind {
                dup_error(&key, &other.key, "music");
            }
        }
        self.musics.insert(key.as_str(), Music { key, block });
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.musics.values() {
            registry.scan(&item.block);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        let dlc_music = match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::tables::misc::DLC_MUSIC,
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::tables::misc::DLC_MUSIC,
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::tables::misc::DLC_MUSIC,
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => crate::hoi4::tables::misc::DLC_MUSIC,
        };
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

impl FileHandler<Block> for Musics {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("music")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if entry.path().parent().unwrap().ends_with("music_player_categories") {
            return None;
        }
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Music {
    key: Token,
    block: Block,
}

impl Music {
    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        let mut sc = ScopeContext::new(Scopes::Character, &self.key);

        vd.field_localization("name", &mut sc);
        vd.field_item("music", Item::Sound);
        vd.field_item("group", Item::Music); // Take settings from this item
        vd.field_integer("pause_factor");

        vd.field_bool("mood");
        vd.field_bool("is_prioritized_mood");
        vd.field_bool("can_be_interrupted");

        validate_optional_duration_int(&mut vd);
        vd.field_integer("calls");

        vd.field_bool("trigger_prio_override");
        vd.field_validated_block("is_valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_list_numeric_exactly("subsequent_playback_chance", 3);
    }
}

#[derive(Clone, Debug)]
pub struct MusicPlayerCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::MusicPlayerCategory, MusicPlayerCategory::add)
}

impl MusicPlayerCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("category") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::MusicPlayerCategory, id.clone(), block, Box::new(Self {}));
            } else {
                let msg = "category without id";
                warn(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else {
            let msg = format!("unknown key {key} in music categories");
            warn(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for MusicPlayerCategory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);

        vd.field_value("id"); // used in ::add
        vd.field_item("name", Item::Localization);
        vd.field_list_items("tracks", Item::Music);
    }
}
