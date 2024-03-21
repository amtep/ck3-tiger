use std::path::PathBuf;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::report::{report, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_optional_duration_int;
use crate::validator::Validator;

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

    pub fn exists(&self, key: &str) -> bool {
        self.musics.contains_key(key) || DLC_MUSIC.contains(&key)
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
        PathBuf::from("")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !(entry.path().starts_with("music")
            || entry.path().starts_with("dlc") && entry.path().parent().unwrap().ends_with("music"))
        {
            return None;
        }
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry)
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

/// A list of music provided by DLCs, for people who don't have them
/// LAST UPDATED VERSION 1.12.1
const DLC_MUSIC: &[&str] = &[
    // FP1
    "mx_raid",
    "mx_drakkar",
    "mx_scandinavia",
    "mx_thefeast",
    // EP1
    "middleeasterncourt_cue",
    "europeancourt_cue",
    "indiancourt_cue",
    "mediterraneancourt_cue",
    "mep1_mood_01",
    "mep1_mood_02",
    "mep1_mood_03",
    "mep1_mood_04",
    "group_roco",
    // FP2
    "mx_IberiaWar",
    "mx_Struggle_ending_compromise",
    "mx_Struggle_ending_conciliation",
    "mx_Struggle_ending_hostility",
    "mx_Struggle_Opening",
    "mx_iberian_moodTrack1",
    "mx_iberian_moodTrack2",
    "mx_iberian_moodTrack3",
    "group_foi",
    // BP1
    "mx_BP1Mood_Generic",
    "mx_BP1Mood_Western",
    "mx_BP1Mood_MiddleEastern",
    "group_bp1",
    // EP2
    "tournamentwest_cue",
    "tournamentmena_cue",
    "tournamentindia_cue",
    "tournamentend_cue",
    "tourwest_cue",
    "tourmena_cue",
    "tourindia_cue",
    "tourend_cue",
    "weddingwest_cue",
    "weddingmena_cue",
    "weddingindia_cue",
    "weddingend_cue",
    "grandfeast_cue",
    "murderfeast_event_cue",
    "murderfest_cue",
    "india_arrival_neutral_cue",
    "india_arrival_suspicious_cue",
    "india_arrival_welcome_cue",
    "mena_arrival_neutral_cue",
    "mena_arrival_suspicious_cue",
    "mena_arrival_welcome_cue",
    "west_arrival_neutral_cue",
    "west_arrival_suspicious_cue",
    "west_arrival_welcome_cue",
    "mep2_mood_01",
    "mep2_mood_02",
    "mep2_mood_03",
    "mep2_mood_04",
    "group_ep2_cuetrack",
    "group_ep2_moodtrack",
    "mx_cue_tournament_win",
    "mx_cue_tournament_lose",
    "mx_cue_tournament_brawl",
    "mx_cue_tournament_horse",
    "mx_cue_tournament_mind",
    "mx_cue_armorer",
    "mx_cue_visitor_camp",
    "mx_cue_farrier",
    "mx_cue_fletcher",
    "mx_cue_tourney_grounds",
    "mx_cue_settlement",
    "mx_cue_tailor",
    "mx_cue_tavern",
    "mx_cue_temple",
    "mx_cue_weaponsmith",
    // BP2
    "mbp2_mood_01",
    "mbp2_mood_02",
    "mbp2_mood_03",
    "mbp2_mood_04",
    "group_bp2_moodtrack",
    // FP3
    "strugglestart_cue",
    "struggleend_cue",
    "strugglewar_cue",
    "mfp3_mood_01",
    "mfp3_mood_02",
    "mfp3_mood_03",
    "mfp3_mood_04",
    "mfp3_mood_05",
    "group_fp3_cuetrack",
    "group_fp3_moodtrack",
    // CE1 (documented as fp4)
    "apocalyptic_plague",
    "black_death",
    "legend_begins",
    "mfp4_mood_epidemics_01",
    "mfp4_mood_epidemics_02",
    "mfp4_mood_legends_01",
    "mfp4_mood_legends_02",
    "group_fp4_cuetrack",
    "group_fp4_moodtrack",
];
