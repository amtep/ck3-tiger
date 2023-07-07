use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;
use crate::validate::validate_camera_color;

#[derive(Clone, Debug)]
pub struct Environment {}

impl Environment {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Environment, key, block, Box::new(Self {}));
    }
}

impl DbKind for Environment {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("cubemap", Item::File);
        vd.field_precise_numeric("cubemap_intensity");

        vd.field_validated("lights", |bv, data| {
            match bv {
                BV::Value(token) => data.verify_exists(Item::Environment, token),
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);
                    for (_, block) in vd.integer_blocks() {
                        let mut vd = Validator::new(block, data);
                        vd.field_choice(
                            "type",
                            &["spot_light", "point_light", "directional_light"],
                        );
                        vd.field_bool("affected_by_shadow");
                        vd.field_validated_block("color", validate_camera_color);

                        vd.field_list_precise_numeric_exactly("position", 3);
                        if block.field_value_is("type", "spot_light")
                            || block.field_value_is("type", "directional_light")
                        {
                            vd.field_list_precise_numeric_exactly("look_at", 3);
                            vd.field_value("look_at_node"); // TODO
                        } else {
                            vd.ban_field("look_at", || "spot_light or directional_light");
                            vd.ban_field("look_at_node", || "spot_light or directional_light");
                        }

                        if block.field_value_is("type", "spot_light")
                            || block.field_value_is("type", "point_light")
                        {
                            vd.field_precise_numeric("radius");
                            vd.field_precise_numeric("falloff");
                        } else {
                            vd.ban_field("radius", || "spot_light or point_light");
                            vd.ban_field("falloff", || "spot_light or point_light");
                        }
                        if block.field_value_is("type", "point_light") {
                            vd.field_precise_numeric("outer_cone_angle");
                            vd.field_precise_numeric("inner_cone_angle");
                        } else {
                            // These fields are very often present anyway, so instead of lots of warnings,
                            // just advice about them.
                            vd.advice_field(
                                "outer_cone_angle",
                                "outer_cone_angle is only for point_light",
                            );
                            vd.advice_field(
                                "inner_cone_angle",
                                "inner_cone_angle is only for point_light",
                            );
                        }
                    }
                }
            }
        });

        vd.field_validated("shadow_camera", |bv, data| {
            match bv {
                BV::Value(token) => data.verify_exists(Item::Environment, token),
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);

                    vd.field_list_precise_numeric_exactly("position", 3);
                    vd.field_list_precise_numeric_exactly("look_at", 3);
                    vd.field_value("look_at_node"); // TODO
                    vd.field_precise_numeric("fov");
                    vd.field_list_integers_exactly("camera_near_far", 2);
                }
            }
        });
    }
}
