use crate::block::Block;
use crate::ck3::validate::validate_camera_color;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::pdxfile::PdxEncoding;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MapEnvironment {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::MapEnvironment, PdxEncoding::Utf8OptionalBom, ".txt", true, MapEnvironment::add)
}

impl MapEnvironment {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MapEnvironment, key, block, Box::new(Self {}));
    }
}

impl DbKind for MapEnvironment {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("ambient_pos_x", validate_camera_color);
        vd.field_validated_block("ambient_neg_x", validate_camera_color);
        vd.field_validated_block("ambient_pos_y", validate_camera_color);
        vd.field_validated_block("ambient_neg_y", validate_camera_color);
        vd.field_validated_block("ambient_pos_z", validate_camera_color);
        vd.field_validated_block("ambient_neg_z", validate_camera_color);

        vd.field_validated_block("shadow_ambient_pos_x", validate_camera_color);
        vd.field_validated_block("shadow_ambient_neg_x", validate_camera_color);
        vd.field_validated_block("shadow_ambient_pos_y", validate_camera_color);
        vd.field_validated_block("shadow_ambient_neg_y", validate_camera_color);
        vd.field_validated_block("shadow_ambient_pos_z", validate_camera_color);
        vd.field_validated_block("shadow_ambient_neg_z", validate_camera_color);

        vd.field_validated_block("sun_color", validate_camera_color);
        vd.field_precise_numeric("sun_intensity");
        vd.field_list_precise_numeric_exactly("sun_direction", 3);
        vd.field_list_precise_numeric_exactly("shadow_direction_offset", 3);
        vd.field_precise_numeric("cubemap_intensity");
        vd.field_item("cubemap", Item::File);

        vd.field_validated_block("fog_color", validate_camera_color);
        vd.field_precise_numeric("fog_begin");
        vd.field_precise_numeric("fog_end");
        vd.field_precise_numeric("fog_max");

        vd.field_list_precise_numeric_exactly("water_sun_direction_offset", 3);

        vd.field_precise_numeric("hue_offset");
        vd.field_precise_numeric("saturation_scale");
        vd.field_precise_numeric("value_scale");
        vd.field_list_precise_numeric_exactly("colorbalance", 3);
        vd.field_validated_block("levels_min", validate_camera_color);
        vd.field_validated_block("levels_max", validate_camera_color);

        vd.field_precise_numeric("bloom_width");
        vd.field_precise_numeric("bloom_scale");
        vd.field_precise_numeric("bright_threshold");

        vd.field_precise_numeric("hdr_min_adjustment");
        vd.field_precise_numeric("hdr_max_adjustment");
        vd.field_precise_numeric("hdr_adjustment_speed");
        vd.field_precise_numeric("tonemap_middlegrey");
        vd.field_precise_numeric("tonemap_whiteluminance");

        vd.field_value("exposure_function"); // TODO
        vd.field_precise_numeric("exposure");

        vd.field_choice(
            "tonemap_function",
            &["Uncharted", "Reinhard", "ReinhardModified", "Filmic", ""],
        );
        vd.field_validated_block("tonemap_curve", validate_curve);

        vd.field_validated_block("depthoffield", validate_dof);
    }
}

fn validate_curve(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_precise_numeric("shoulder_strength");
    vd.field_precise_numeric("linear_strength");
    vd.field_precise_numeric("linear_angle");
    vd.field_precise_numeric("toe_strength");
    vd.field_precise_numeric("toe_numerator");
    vd.field_precise_numeric("toe_denominator");
    vd.field_precise_numeric("linear_white");
}

fn validate_dof(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_bool("enabled");
    vd.field_precise_numeric("dof_samplecount");
    vd.field_precise_numeric("dof_baseradius");
    vd.field_precise_numeric("dof_blurblendmin");
    vd.field_precise_numeric("dof_blurblendmax");
    vd.field_precise_numeric("dof_blurmin");
    vd.field_precise_numeric("dof_blurmax");
    vd.field_precise_numeric("dof_blurscale");
    vd.field_precise_numeric("dof_blurexponent");
    vd.field_precise_numeric("dof_heightmin");
    vd.field_precise_numeric("dof_heightmax");
}
