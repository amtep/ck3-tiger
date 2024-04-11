use enum_map::EnumMap;
use iced::widget::{column, Column, Container, Rule, Scrollable, Text};
use strum::IntoEnumIterator;

use crate::game::Game;
use crate::message::Message;

#[derive(Default)]
pub(crate) struct Mods {
    game_mods: EnumMap<Game, Vec<Mod>>,
}

impl Mods {
    pub(crate) fn view(&self) -> Column<Message> {
        let mut game_sections = Column::new().spacing(10);
        for game in Game::iter() {
            game_sections =
                game_sections.push(column![Text::new(game.fullname()), Rule::horizontal(1)]);
        }

        column![Text::new("Mods"), Container::new(Scrollable::new(game_sections)),]
    }
}

struct Mod {}
