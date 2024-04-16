use iced::widget::{row, vertical_rule};
use iced::{executor, Application, Command, Element, Length, Theme};

use crate::message::Message;
use crate::mods::Mods;
use crate::reports::Reports;

pub struct TigerApp {
    mods: Mods,
    reports: Reports,
}

impl Application for TigerApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { mods: Mods::load(), reports: Reports::default() }, Command::none())
    }

    fn title(&self) -> String {
        "Tiger".to_owned()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        row![
            self.mods.view().width(Length::FillPortion(1)).padding(10),
            vertical_rule(4),
            self.reports.view().width(Length::FillPortion(2)).padding(10)
        ]
        .into()
    }
}
