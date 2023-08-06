//! Validator for triggered description blocks that compose a description from multiple
//! localization keys.
//!
//! Such desc blocks are accepted in many places in the game script.
//!
//! The main entry point is [`validate_desc`].

use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{old_warn, warn_info, ErrorKey};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

/// Internal function to recurse over the complex description block logic.
///
/// `caller` is the name of the key that opened this block.
/// `block` is the block or sub-block being validated.
/// `sc` is the scope in which to evaluate any triggers found.
/// `f` is the closure to run over any strings found.
fn validate_desc_map_block(
    caller: &str,
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    f: &impl Fn(&Token, &Everything, &mut ScopeContext),
) {
    let mut vd = Validator::new(block, data);
    let mut seen_desc = false;
    let mut seen_unconditional_desc = false;
    vd.unknown_fields(|key, bv| {
        if key.is("desc") || key.is("first_valid") || key.is("random_valid") {
            if seen_desc && caller == "triggered_desc" {
                let msg = "multiple descs in one triggered_desc";
                let info = "only the last one will be shown";
                warn_info(key, ErrorKey::DuplicateField, msg, info);
            }
            if seen_unconditional_desc && caller == "first_valid" {
                let msg = "multiple unconditional desc in one first_valid";
                let info = "only the first one will be shown";
                warn_info(key, ErrorKey::DuplicateField, msg, info);
            }
            if key.is("desc") {
                match bv {
                    BV::Value(token) => {
                        if !token.as_str().contains(' ') {
                            f(token, data, sc);
                        }
                    }
                    BV::Block(block) => {
                        validate_desc_map_block(key.as_str(), block, data, sc, f);
                    }
                }
                // first_valid and random_valid are not unconditional because all their triggers might fail
                seen_unconditional_desc = true;
            } else if let Some(block) = bv.expect_block() {
                validate_desc_map_block(key.as_str(), block, data, sc, f);
            }
            seen_desc = true;
        } else if key.is("triggered_desc") {
            if let Some(block) = bv.expect_block() {
                if seen_desc && caller == "triggered_desc" {
                    let msg = "multiple descs in one triggered_desc";
                    let info = "only the last one will be shown";
                    warn_info(key, ErrorKey::DuplicateField, msg, info);
                }
                validate_desc_map_block(key.as_str(), block, data, sc, f);
                seen_desc = true;
            }
        } else if key.is("trigger") {
            if let Some(block) = bv.expect_block() {
                if caller != "triggered_desc" {
                    let msg = "`trigger` is only for `triggered_desc";
                    old_warn(key, ErrorKey::Validation, msg);
                }
                validate_trigger(block, data, sc, Tooltipped::No);
            }
        } else {
            old_warn(key, ErrorKey::UnknownField, "unexpected key in description");
        }
    });
}

/// Like [`validate_desc`], but allows the caller to decide what to do with the strings found in
/// the description. This is useful for example for description blocks that resolve an icon name
/// rather than a description.
pub fn validate_desc_map(
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    f: impl Fn(&Token, &Everything, &mut ScopeContext),
) {
    match bv {
        BV::Value(t) => {
            if !t.as_str().contains(' ') {
                f(t, data, sc);
            }
        }
        BV::Block(b) => {
            validate_desc_map_block("", b, data, sc, &f);
        }
    }
}

/// Validate a complex description, which may be a simple localization key or a block containing
/// items like `triggered_desc` or `first_valid`.
pub fn validate_desc(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    validate_desc_map(bv, data, sc, |token, data, sc| {
        data.verify_exists(Item::Localization, token);
        data.validate_localization_sc(token.as_str(), sc);
    });
}
