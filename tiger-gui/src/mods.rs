use std::fs::read_dir;
use std::path::Path;

use anyhow::{bail, Result};
use enum_map::EnumMap;
use iced::widget::{column, Column, Container, Rule, Scrollable, Text};
use strum::IntoEnumIterator;

use crate::game::Game;
use crate::message::Message;

pub(crate) struct Mods {
    /// Mods to choose from when running Tiger, in the order they should be displayed to the user.
    /// Invariant: Every game has an entry in the map.
    game_mods: EnumMap<Game, Vec<Mod>>,
}

impl Default for Mods {
    fn default() -> Self {
        Self { game_mods: Game::iter().map(|game| (game, Vec::new())).collect() }
    }
}

impl Mods {
    /// Scan the mods that are installed locally in each game's mod directory (so not counting the
    /// ones installed from the workshop).
    // TODO: load mods from persistent settings first, and then add newly scanned mods and skip
    // scanned mods that were removed by the user.
    pub(crate) fn load() -> Self {
        let mut mods = Self::default();

        for game in Game::iter() {
            mods.game_mods[game] = Mod::enumerate_local(game);
        }

        mods
    }

    pub(crate) fn view(&self) -> Column<Message> {
        let mut game_sections = Column::new().spacing(10);
        for game in Game::iter() {
            game_sections =
                game_sections.push(column![Text::new(game.fullname()), Rule::horizontal(1)]);
        }

        column![Text::new("Mods"), Container::new(Scrollable::new(game_sections)),]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Mod {}

impl Mod {
    /// List the locally installed mods for the given game.
    /// Ignore any errors encountered in the process; just return the mods that were readable.
    fn enumerate_local(game: Game) -> Vec<Self> {
        let mut mods = Vec::new();
        if let Some(mod_dir) = game.find_mod_dir() {
            if let Ok(entries) = read_dir(mod_dir) {
                for entry in entries.flatten() {
                    match game {
                        Game::Ck3 | Game::Imperator => {
                            if !entry.file_name().to_string_lossy().ends_with(".mod")
                                && !entry.file_name().to_string_lossy().starts_with("ugc_")
                            {
                                if let Ok(the_mod) = Self::from_descriptor(&entry.path()) {
                                    mods.push(the_mod);
                                }
                            }
                        }
                        Game::Vic3 => {
                            if let Ok(the_mod) = Self::from_metadata(
                                &entry.path(),
                                &entry.path().join(".metadata/metadata.json"),
                            ) {
                                mods.push(the_mod);
                            }
                        }
                    }
                }
            }
        }
        mods.sort();
        mods
    }

    /// Construct a `Mod` from reading a `.mod` file.
    // TODO: implement
    fn from_descriptor(_descriptor: &Path) -> Result<Self> {
        bail!("not implemented yet");
    }

    /// Construct a `Mod` from reading a `metadata.json` file.
    fn from_metadata(_dir: &Path, _metadata: &Path) -> Result<Self> {
        bail!("not implemented yet");
    }
}
