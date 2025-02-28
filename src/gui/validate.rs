use crate::block::{BV, Block};
use crate::context::ScopeContext;
use crate::data::localization::LocaValue;
use crate::datatype::{Datatype, validate_datatypes};
use crate::everything::Everything;
#[cfg(feature = "ck3")]
use crate::game::Game;
use crate::game::GameFlags;
use crate::gui::properties::{ALIGN, BLENDMODES};
use crate::gui::{GuiCategories, GuiValidation, PropertyContainer, WidgetProperty};
use crate::helpers::stringify_choices;
use crate::item::Item;
use crate::parse::localization::ValueParser;
use crate::report::{ErrorKey, Severity, err, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validator::Validator;

pub fn validate_property(
    property: WidgetProperty,
    container: Option<PropertyContainer>,
    key: &Token,
    bv: &BV,
    data: &Everything,
) {
    let game = GameFlags::game();
    let gameflags = property.to_game_flags();
    if !gameflags.contains(game) {
        if property == WidgetProperty::tooltip_enabled {
            let msg = "tooltip_enabled has been renamed to tooltip_visible";
            err(ErrorKey::Removed).msg(msg).loc(key).push();
        } else {
            let msg = format!("{key} is only for {gameflags}");
            err(ErrorKey::WrongGame).weak().msg(msg).loc(key).push();
        }
        return;
    }
    if let Some(container) = container {
        let allowed_properties = match container {
            PropertyContainer::BuiltinWidget(builtin) => {
                GuiCategories::widget_as_container(builtin)
            }
            PropertyContainer::ComplexProperty(prop) | PropertyContainer::WidgetProperty(prop) => {
                GuiCategories::property_as_container(prop)
            }
        };
        let allowed_containers = GuiCategories::property_in_container(property);
        if !allowed_containers.intersects(allowed_properties) {
            let msg = format!("property {property} is not allowed in {container}");
            err(ErrorKey::Gui).weak().msg(msg).loc(key).push();
        }
    }
    match GuiValidation::from_property(property) {
        GuiValidation::UncheckedValue | GuiValidation::Format => {
            // TODO: validate Format as a format string
            _ = bv.expect_value();
        }
        GuiValidation::DatatypeExpr | GuiValidation::Datamodel => {
            validate_datatype_field(Datatype::Unknown, key, bv, data, false);
        }
        GuiValidation::Datacontext => {
            validate_datatype_field(Datatype::Unknown, key, bv, data, true);
        }
        GuiValidation::Boolean => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::bool, key, bv, data, false);
                } else if !value.lowercase_is("yes") && !value.lowercase_is("no") {
                    // TODO: decide based on the field name whether to upgrade to error?
                    warn(ErrorKey::Validation).msg("expected yes or no").loc(value).push();
                }
            }
        }
        GuiValidation::Yes => {
            if let Some(value) = bv.expect_value() {
                if !value.is("yes") {
                    warn(ErrorKey::Validation).msg("expected only yes").loc(value).push();
                }
            }
        }
        GuiValidation::Align => {
            if let Some(value) = bv.expect_value() {
                for part in value.split('|') {
                    if !ALIGN.contains(&part.as_str()) {
                        let msg = format!("unknown {key} {part}");
                        let info = format!("known {key}s are {}", stringify_choices(ALIGN));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(part).push();
                    }
                }
            }
        }
        GuiValidation::Integer => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::int32, key, bv, data, false);
                } else {
                    value.expect_integer();
                }
            }
        }
        GuiValidation::UnsignedInteger => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::uint32, key, bv, data, false);
                } else if let Some(i) = value.expect_integer() {
                    if i < 0 {
                        let msg = format!("{key} needs an unsigned integer");
                        warn(ErrorKey::Range).msg(msg).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::Number => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::float, key, bv, data, false);
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::NumberOrInt32 => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need a way to express it can be int32 or float
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::NumberF => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need a way to express it can be int32 or float
                    validate_datatype_field(Datatype::float, key, bv, data, false);
                } else if let Some(value) = value.strip_suffix("f") {
                    // TODO: this f is used in vanilla; check it really works.
                    value.expect_number();
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::NumberOrPercent => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need a way to express it can be int32 or float
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else if let Some(value) = value.strip_suffix("%") {
                    value.expect_number();
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::TwoNumberOrPercent => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector2f, key, bv, data, false);
            }
            BV::Block(block) => {
                for value in block.iter_values_warn() {
                    if let Some(value) = value.strip_suffix("%") {
                        value.expect_number();
                    } else {
                        value.expect_number();
                    }
                }
            }
        },
        GuiValidation::CVector2f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector2f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_numbers_exactly(2);
            }
        },
        GuiValidation::CVector2i => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector2i, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_integers_exactly(2);
            }
        },
        GuiValidation::CVector3f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector3f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_numbers_exactly(3);
            }
        },
        GuiValidation::CVector4f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector4f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_numbers_exactly(4);
            }
        },
        GuiValidation::Color => match bv {
            BV::Value(_) => {
                // TODO: can be CVector4f or CString
                validate_datatype_field(Datatype::Unknown, key, bv, data, false);
            }
            BV::Block(block) => {
                validate_gui_color(block, data);
            }
        },
        GuiValidation::CString => {
            validate_datatype_field(Datatype::CString, key, bv, data, false);
        }
        GuiValidation::Item(itype) => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need some way of specifying "stringable" datatypes
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else {
                    data.verify_exists(itype, value);
                }
            }
        }
        GuiValidation::ItemOrBlank(itype) => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need some way of specifying "stringable" datatypes
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else if !value.is("") {
                    data.verify_exists(itype, value);
                }
            }
        }
        GuiValidation::Blendmode => {
            if let Some(value) = bv.expect_value() {
                let value_lc = value.as_str().to_ascii_lowercase();
                if !BLENDMODES.contains(&&*value_lc) {
                    let msg = "unknown blendmode";
                    let info = format!("expected one of {}", stringify_choices(BLENDMODES));
                    warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                }
            }
        }
        GuiValidation::MouseButton(choices) => {
            if let Some(value) = bv.expect_value() {
                // TODO: datatype is only really used by button_ignore.
                // Is it valid for the others?
                if value.starts_with("[") {
                    // TODO: need some way of specifying "stringable" datatypes
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else {
                    let value_lc = value.as_str().to_ascii_lowercase();
                    if !choices.contains(&&*value_lc) {
                        let msg = "unknown mouse button";
                        let info = format!("expected one of {}", stringify_choices(choices));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::MouseButtonSet(choices) => {
            if let Some(value) = bv.expect_value() {
                for part in value.split('|') {
                    let part_lc = part.as_str().to_ascii_lowercase();
                    if !choices.contains(&&*part_lc) {
                        let msg = "unknown mouse button";
                        let info = format!("expected one of {}", stringify_choices(choices));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::Choice(choices) => {
            if let Some(value) = bv.expect_value() {
                let value_lc = value.as_str().to_ascii_lowercase();
                if !choices.contains(&&*value_lc) {
                    let msg = "unknown value";
                    let info = format!("expected one of {}", stringify_choices(choices));
                    warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                }
            }
        }
        GuiValidation::ChoiceSet(choices) => {
            if let Some(value) = bv.expect_value() {
                for part in value.split('|') {
                    let part_lc = part.as_str().to_ascii_lowercase();
                    if !choices.contains(&&*part_lc) {
                        let msg = "unknown value";
                        let info = format!("expected one of {}", stringify_choices(choices));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::Widget => {
            match bv {
                BV::Value(value) => {
                    data.verify_exists(Item::GuiTemplate, value);
                    // Templates are validated separately, and this Widget field adds no context to that.
                    // TODO: verify that this is a template containing one widget.
                }
                BV::Block(_block) => {
                    // TODO: verify that this block contains only one widget (though it may also
                    // have a recursive = yes field).
                    // TODO: if this block is not recursive, validate it now.
                    // TODO: perform blockoverrides on this block.
                }
            }
        }
        GuiValidation::ComplexProperty => {
            // Complex properties have their own item type, where they get a GuiBlock input
            // rather than a BV.
            unreachable!();
        }
        GuiValidation::FormatOverride => {
            if let Some(block) = bv.expect_block() {
                let mut count = 0;
                for value in block.iter_values_warn() {
                    count += 1;
                    data.verify_exists(Item::TextFormat, value);
                    if count == 3 {
                        let msg = "expected exactly 2 text formats";
                        warn(ErrorKey::Validation).msg(msg).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::RawText | GuiValidation::Text => {
            if let Some(text) = bv.expect_value() {
                let value = ValueParser::new(vec![text]).parse();
                validate_gui_loca(key, value, data);
                if !text.starts_with("[") && !text.as_str().contains(' ') {
                    // even raw text can still be a localization key sometimes
                    data.mark_used(Item::Localization, text.as_str());
                }
            }
        }
    }
}

pub fn validate_datatype_field(
    dtype: Datatype,
    key: &Token,
    bv: &BV,
    data: &Everything,
    allow_promote: bool,
) {
    if let Some(value) = bv.expect_value() {
        if value.starts_with("[") {
            let loca_value = ValueParser::new(vec![value]).parse();
            let mut sc = ScopeContext::new(Scopes::None, key);
            match loca_value {
                // TODO: validate format
                LocaValue::Code(chain, format) => {
                    validate_datatypes(
                        &chain,
                        data,
                        &mut sc,
                        dtype,
                        "",
                        format.as_ref(),
                        allow_promote,
                    );
                }
                LocaValue::Error => (),
                _ => {
                    let msg = "expected whole field to be a [ ] expression";
                    warn(ErrorKey::Validation).msg(msg).loc(value).push();
                }
            }
        } else {
            let msg = "expected a [ ] expression here";
            warn(ErrorKey::Validation).msg(msg).loc(value).push();
        }
    }
}

fn validate_gui_loca(key: &Token, loca_value: LocaValue, data: &Everything) {
    match loca_value {
        LocaValue::Concat(v) => {
            for loca_value in v {
                validate_gui_loca(key, loca_value, data);
            }
        }
        LocaValue::Code(chain, format) => {
            // |E is the formatting used for game concepts in ck3
            #[cfg(feature = "ck3")]
            if Game::is_ck3() {
                if let Some(ref format) = format {
                    if format.as_str().contains('E') || format.as_str().contains('e') {
                        if let Some(concept) = chain.as_gameconcept() {
                            data.verify_exists(Item::GameConcept, concept);
                            return;
                        }
                    }
                }
            }

            let mut sc = ScopeContext::new(Scopes::None, key);
            validate_datatypes(
                &chain,
                data,
                &mut sc,
                Datatype::Unknown,
                "",
                format.as_ref(),
                false,
            );
        }
        LocaValue::Icon(token) => {
            data.verify_exists(Item::TextIcon, &token);
        }
        _ => (),
    }
}

fn validate_gui_color(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;
    for value in vd.values() {
        count += 1;
        // TODO: verify whether gui really does support precise numbers.
        // They're used in a few places by vanilla but that doesn't mean it works...
        // TODO: check ranges
        value.expect_precise_number();
    }
    if count != 4 {
        warn(ErrorKey::Colors).msg("expected 4 color values").loc(block).push();
    }
}
