//! An enum listing the supported games that tiger can check.

use enum_map::Enum;
use strum_macros::EnumIter;

/// An enum listing the supported games that tiger can check.
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Enum, EnumIter)]
pub(crate) enum Game {
    Ck3,
    Vic3,
    Imperator,
}

impl Game {
    pub(crate) fn fullname(self) -> &'static str {
        match self {
            Game::Ck3 => "Crusader Kings 3",
            Game::Vic3 => "Victoria 3",
            Game::Imperator => "Imperator: Rome",
        }
    }
}
