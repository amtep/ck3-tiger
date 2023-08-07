//! Types and functions related to `.gui` file processing

pub use self::block::{GuiBlock, GuiBlockFrom};
pub use self::builtins::BuiltinWidget;
pub use self::properties::{GuiValidation, WidgetProperty};

mod block;
mod builtins;
mod properties;
mod validate;
