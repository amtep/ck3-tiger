//! Types and functions related to `.gui` file processing

pub use self::block::{GuiBlock, GuiBlockFrom};
pub use self::builtins::BuiltinWidget;
pub use self::categories::GuiCategories;
pub use self::properties::{GuiValidation, PropertyContainer, WidgetProperty};
#[cfg(not(feature = "imperator"))]
pub use self::validate::validate_datatype_field;

mod block;
mod builtins;
mod categories;
mod properties;
mod validate;
