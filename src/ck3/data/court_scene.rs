use std::cmp::max;

use crate::block::validator::Validator;
use crate::block::Block;
use crate::ck3::data::scripted_animations::validate_scripted_animation;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{old_warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;

#[derive(Clone, Debug)]
pub struct CourtSceneGroup {}

impl CourtSceneGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtSceneGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtSceneGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("order_type", &["random", "ascending", "descending"]);
        vd.field_choice("position_type", &["dynamic", "static"]);
        vd.field_choice("access_type", &["random", "top"]);
        vd.field_value("value"); // TODO
    }
}

#[derive(Clone, Debug)]
pub struct CourtSceneRole {}

impl CourtSceneRole {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtSceneRole, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtSceneRole {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("ruler", Scopes::Character, key);

        vd.field_validated_sc("scripted_animation", &mut sc, validate_scripted_animation);

        vd.field_validated_key_block("effect", |key, block, data| {
            sc.define_list("characters", Scopes::Character, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_bool("is_low_priority");
        vd.field_item("group", Item::CourtSceneGroup);
    }
}

#[derive(Clone, Debug)]
pub struct CourtSceneCulture {}

impl CourtSceneCulture {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CourtSceneCulture, key, block, Box::new(Self {}));
    }
}

impl DbKind for CourtSceneCulture {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.field_validated_block("trigger", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct CourtSceneSetting {}

impl CourtSceneSetting {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        // TODO: validate grandeur_levels.txt as well
        if !key.is("grandeur_levels") {
            db.add(Item::CourtSceneSetting, key, block, Box::new(Self {}));
        }
    }
}

impl DbKind for CourtSceneSetting {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name"); // TODO
        vd.field_item("culture", Item::CourtSceneCulture);
        vd.field_integer("visual_culture_level");
        vd.field_item("cubemap", Item::File);
        vd.field_item("environment", Item::File);
        vd.field_precise_numeric("audio_culture");

        let mut cameras = Vec::new();
        vd.field_validated_block("camera", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                validate_camera(block, data, &mut cameras);
            }
            if cameras.is_empty() {
                old_warn(block, ErrorKey::Validation, "need at least one camera");
            }
        });

        vd.field_integer_range("default_camera", 0, max(cameras.len() as i64 - 1, 0));
        vd.field_precise_numeric("shadows_fade");
        vd.field_precise_numeric("shadows_strength");

        vd.field_validated_block("lights", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                validate_light(block, data);
            }
        });

        vd.field_validated_block("characters", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                validate_character(block, data, &cameras);
            }
        });

        vd.field_validated_block("assets", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                validate_asset(block, data);
            }
        });

        vd.field_validated_block("artifacts", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                validate_artifact(block, data);
            }
        });

        vd.field_validated_block("support_type", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|_, value| {
                // TODO: what is the key?
                data.verify_exists(Item::Entity, value);
            });
        });
    }
}

fn validate_camera(block: &Block, data: &Everything, cameras: &mut Vec<String>) {
    let mut vd = Validator::new(block, data);
    vd.req_field("description");
    if let Some(token) = vd.field_value("description") {
        cameras.push(token.to_string());
    }
    vd.field_precise_numeric("fov");
    vd.field_list_precise_numeric_exactly("position", 3);
    vd.field_precise_numeric("pitch");
    vd.field_precise_numeric("yaw");
    vd.field_list_precise_numeric_exactly("camera_near_far", 2);
    vd.field_bool("is_camera_used_for_screenshots");
    vd.field_item("royal_court_camera_name_key", Item::Localization);
}

fn validate_light(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_value("description");
    vd.field_block("light"); // TODO
    vd.field_block("shadow_camera"); // TODO
}

fn validate_character(block: &Block, data: &Everything, cameras: &[String]) {
    let mut vd = Validator::new(block, data);
    vd.field_list_precise_numeric_exactly("position", 3);
    vd.field_list_precise_numeric_exactly("rotation", 3);
    vd.field_precise_numeric("direction");
    vd.field_value("locator"); // TODO
    vd.field_value("description");
    if let Some(token) = vd.field_value("camera") {
        if !cameras.contains(&token.to_string()) {
            old_warn(token, ErrorKey::MissingItem, "unknown camera");
        }
    }
    vd.field_list_items("roles", Item::CourtSceneRole);
}

fn validate_asset(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_list_precise_numeric_exactly("position", 3);
    vd.field_list_precise_numeric_exactly("rotation", 3);
    vd.field_precise_numeric("direction");
    vd.field_precise_numeric("scale");
    vd.field_value("description");
    vd.field_item("asset", Item::Asset);
    vd.field_value("roles"); // TODO
    vd.field_item("tag", Item::CourtSceneRole);
}

fn validate_artifact(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_list_precise_numeric_exactly("position", 3);
    vd.field_list_precise_numeric_exactly("rotation", 3);
    vd.field_precise_numeric("direction");
    vd.field_value("locator"); // TODO
    vd.field_item("slot", Item::ArtifactSlot);
}
