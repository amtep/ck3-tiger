use crate::block::validator::Validator;
/// A module for validation functions that are useful for more than one data module.
use crate::block::Block;
use crate::errorkey::ErrorKey;
use crate::errors::warn;

pub fn validate_theme_background(block: &Block) {
    let mut vd = Validator::new(block);

    vd.opt_field_block("trigger");
    // TODO: verify the background is defined
    vd.opt_field_value("event_background");
    // TODO: check if `reference` actually works or is a mistake in vanilla
    vd.opt_field_value("reference");
    vd.warn_remaining();
}

pub fn validate_theme_icon(block: &Block) {
    let mut vd = Validator::new(block);

    vd.opt_field_block("trigger");
    // TODO: verify the file exists
    vd.opt_field_value("reference"); // file
    vd.warn_remaining();
}

pub fn validate_theme_sound(block: &Block) {
    let mut vd = Validator::new(block);

    vd.opt_field_block("trigger");
    vd.opt_field_value("reference"); // event:/ resource reference
    vd.warn_remaining();
}

pub fn validate_cooldown(block: &Block) {
    let mut vd = Validator::new(block);

    let mut count = 0;
    count += isize::from(vd.opt_field_integer("years"));
    count += isize::from(vd.opt_field_integer("months"));
    count += isize::from(vd.opt_field_integer("days"));
    if count != 1 {
        warn(
            block,
            ErrorKey::Validation,
            "cooldown must have one of `years`, `months`, or `days`",
        );
    }
    vd.warn_remaining();
}
