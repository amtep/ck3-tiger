//! An enum listing the supported games that tiger can check.

use std::path::PathBuf;

use enum_map::Enum;
use home::home_dir;
use strum_macros::EnumIter;

/// An enum listing the supported games that tiger can check.
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Enum, EnumIter)]
pub enum Game {
    Ck3,
    Vic3,
    Imperator,
}

impl Game {
    /// The human-readable name for this game.
    pub(crate) fn fullname(self) -> &'static str {
        match self {
            Game::Ck3 => "Crusader Kings 3",
            Game::Vic3 => "Victoria 3",
            Game::Imperator => "Imperator: Rome",
        }
    }

    /// The name of this game's directory under the Paradox folder.
    fn dir_name(self) -> &'static str {
        match self {
            Game::Ck3 => "Crusader Kings III",
            Game::Vic3 => "Victoria 3",
            Game::Imperator => "Imperator",
        }
    }

    /// The directory where the mods for this game are kept by Paradox.
    /// May return None if no such directory was found.
    pub(crate) fn find_mod_dir(self) -> Option<PathBuf> {
        if let Some(home) = home_dir() {
            // Try each of the standard folders for Linux, MacOs, and Windows
            for try_dir in [".local/share", "Library/Application Support", "Documents"] {
                let full_dir = home
                    .join(try_dir)
                    .join("Paradox Interactive")
                    .join(self.dir_name())
                    .join("mod");
                if full_dir.is_dir() {
                    return Some(full_dir);
                }
            }
        }
        None
    }
}
