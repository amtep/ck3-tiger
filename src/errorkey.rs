use strum_macros::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumString, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum ErrorKey {
    Config,
    ReadError,
    ParseError,
    BracePlacement,
    Packaging,
    Validation,
    Filename,
    Encoding,
    Localization,
    Duplicate,
    NameConflict,
    EventNamespace,
    MissingLocalization,
    MissingFile,
    MissingItem,
    WrongGender,
    Conflict,
    ImageFormat,
    Unneeded,
    Scopes,
    Crash,
    Range,
    Tooltip,
    Tidying,
    Rivers,
    Modifiers,
    Macro,
    History,
    Logic,
    Bugs,

    PrincesOfDarkness,
}
