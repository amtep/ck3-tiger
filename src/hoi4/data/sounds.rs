use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::pdxfile::PdxEncoding;
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::util::SmartJoin;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SoundCategory {}
#[derive(Clone, Debug)]
pub struct SoundCompressor {}
#[derive(Clone, Debug)]
pub struct Sound {}
#[derive(Clone, Debug)]
pub struct SoundEffect {}
#[derive(Clone, Debug)]
pub struct SoundFalloff {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Hoi4, Item::SoundEffect, PdxEncoding::Utf8NoBom, ".asset", LoadAsFile::No, Recursive::Yes, SoundEffect::add)
}

impl SoundEffect {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("category") {
            db.add_anonymous(key, block, Box::new(SoundCategory {}));
        } else if key.is("master_compressor") {
            db.add_anonymous(key, block, Box::new(SoundCompressor {}));
        } else if key.is("music_compressor") {
            db.add_anonymous(key, block, Box::new(SoundCompressor {}));
        } else if key.is("sound") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::Sound, name.clone(), block, Box::new(Sound {}));
            } else {
                let msg = "missing `name` field";
                warn(ErrorKey::FieldMissing).msg(msg).loc(&key).push();
            }
        } else if key.is("soundeffect") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::SoundEffect, name.clone(), block, Box::new(SoundEffect {}));
            } else {
                let msg = "missing `name` field";
                warn(ErrorKey::FieldMissing).msg(msg).loc(&key).push();
            }
        } else if key.is("falloff") {
            if let Some(name) = block.get_field_value("name") {
                db.add(Item::SoundFalloff, name.clone(), block, Box::new(SoundFalloff {}));
            } else {
                let msg = "missing `name` field";
                warn(ErrorKey::FieldMissing).msg(msg).loc(&key).push();
            }
        } else {
            let msg = "unexpected key";
            warn(ErrorKey::UnknownField).msg(msg).loc(&key).push();
        }
    }
}

impl DbKind for SoundCategory {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_list_items("soundeffects", Item::SoundEffect);
        vd.field_validated_block("compressor", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_bool("enabled");
            validate_compressor(vd);
        });
    }
}

impl DbKind for SoundCompressor {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let vd = Validator::new(block, data);
        validate_compressor(vd);
    }
}

fn validate_compressor(mut vd: Validator) {
    vd.field_numeric("pregain");
    vd.field_numeric("postgain");
    vd.field_numeric("ratio");
    vd.field_numeric("threshold");
    vd.field_numeric("attacktime");
    vd.field_numeric("releasetime");
}

impl DbKind for Sound {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_value("name");
        vd.req_field("file");
        if let Some(value) = vd.field_value("file") {
            let pathname = value.loc.pathname().smart_join_parent(value.as_str());
            data.verify_exists_implied(Item::File, &pathname.to_string_lossy(), value);
        }
        vd.field_bool("always_load");
        vd.field_numeric("volume");
    }
}

impl DbKind for SoundEffect {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_value("name");
        vd.field_item("falloff", Item::SoundFalloff);
        vd.field_validated_block("sounds", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_item("sound", Item::Sound);
            vd.multi_field_validated_block("weighted_sound", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_item("sound", Item::Sound);
                vd.field_integer("weight");
            });
        });
        vd.field_numeric("volume");
        vd.field_numeric("fade_in");
        vd.field_numeric("fade_out");
        vd.field_integer("max_audible");
        vd.field_integer("polyphony");
        vd.field_bool("is3d");
        vd.field_bool("loop");
        vd.field_choice("max_audible_behaviour", &["fail"]); // TODO: other options
        vd.field_bool("random_sound_when_looping");
        vd.field_bool("looping_playbackrate_random_offset");
        vd.field_bool("looping_delay_random_offset");
        vd.field_bool("prevent_random_repetition");
        vd.field_list_numeric_exactly("delay_random_offset", 2);
        vd.field_list_numeric_exactly("volume_random_offset", 2);
        vd.field_list_numeric_exactly("playbackrate_random_offset", 2);
    }
}

impl DbKind for SoundFalloff {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_value("name");
        vd.field_numeric("min_distance");
        vd.field_numeric("max_distance");
        vd.field_numeric("height_scale");
        vd.field_choice("type", &["linear"]); // TODO: other choices
    }
}
