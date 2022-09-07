use strum_macros::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumString)]
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

    PrincesOfDarkness,
}
