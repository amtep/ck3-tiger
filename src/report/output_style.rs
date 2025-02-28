use std::collections::HashMap;

use ansiterm::Colour::{Black, Blue, Cyan, Green, Purple, Red, White, Yellow};
use ansiterm::Style;

use crate::report::Severity;

/// For looking up the style to use for the various parts of the output.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Styled {
    #[default]
    Default,
    Tag(Severity, IsTag),
    /// The actual error message, telling the user what is wrong.
    ErrorMessage,
    /// Introduces additional info on a report.
    InfoTag,
    /// The actual info message. Optionally attached to a report.
    Info,
    /// Filename, line number, column number.
    Location,
    /// The caret, pointing at the exact location of the error.
    /// TODO: Maybe this should depend on error level.
    Caret,
    /// Text from the source file.
    /// By default, this should probably be unstyled,
    /// but we might want to add the option to configure it.
    SourceText,
    /// We could potentially give emphasis to certain parts of an error message.
    /// For example; in the following message, the word limit could perhaps be italicized.
    /// "required field `limit` missing"
    Emphasis,
}

/// Whether the style applies to the `ErrorLevel` tag itself or the `ErrorKey` that follows it.
pub type IsTag = bool;

#[derive(Debug)]
pub struct OutputStyle {
    map: HashMap<Styled, Style>,
}

impl Default for OutputStyle {
    /// Constructs an instance of `OutputStyles` that uses default, hard-coded color values.
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(Styled::Default, Style::new());

        map.insert(Styled::InfoTag, Style::new().bold());
        map.insert(Styled::Info, Style::new());
        map.insert(Styled::ErrorMessage, Style::new().bold());
        map.insert(Styled::Location, Blue.bold());
        map.insert(Styled::Caret, Style::new().bold());
        map.insert(Styled::SourceText, Style::new());
        map.insert(Styled::Emphasis, Style::new().italic());

        map.insert(Styled::Tag(Severity::Fatal, true), White.bold());
        map.insert(Styled::Tag(Severity::Fatal, false), White.bold());
        map.insert(Styled::Tag(Severity::Error, true), Red.bold());
        map.insert(Styled::Tag(Severity::Error, false), Red.bold());
        map.insert(Styled::Tag(Severity::Warning, true), Yellow.bold());
        map.insert(Styled::Tag(Severity::Warning, false), Yellow.normal());
        map.insert(Styled::Tag(Severity::Untidy, true), Cyan.bold());
        map.insert(Styled::Tag(Severity::Untidy, false), Cyan.normal());
        map.insert(Styled::Tag(Severity::Tips, true), Green.bold());
        map.insert(Styled::Tag(Severity::Tips, false), Green.normal());

        OutputStyle { map }
    }
}

impl OutputStyle {
    /// Construct a version of the `OutputStyles` that always returns the default, no-colour style.
    /// Use this to effectively disable any ANSI characters in the output.
    pub fn no_color() -> Self {
        let mut map = HashMap::new();
        map.insert(Styled::Default, Style::new());
        OutputStyle { map }
    }
    pub fn style(&self, output: Styled) -> &Style {
        self.map
            .get(&output)
            .or_else(|| self.map.get(&Styled::Default))
            .expect("Failed to retrieve output style.")
    }
    /// Allows overriding a color for a given `ErrorLevel`.
    pub fn set(&mut self, severity: Severity, color_str: &str) {
        if let Some(color) = match color_str.to_ascii_lowercase().as_str() {
            "black" => Some(Black),
            "red" => Some(Red),
            "green" => Some(Green),
            "yellow" => Some(Yellow),
            "blue" => Some(Blue),
            "purple" => Some(Purple),
            "cyan" => Some(Cyan),
            "white" => Some(White),
            _ => None,
        } {
            self.map.insert(Styled::Tag(severity, true), color.bold());
            self.map.insert(Styled::Tag(severity, false), color.normal());
        } else {
            eprintln!(
                "Tried to set ErrorLevel::{severity} to color {color_str}, but that color was not recognised! Defaulting to regular color instead.\nSupported colors are Black, Red, Green, Yellow, Blue, Purple, Cyan, White."
            );
        }
    }
}
