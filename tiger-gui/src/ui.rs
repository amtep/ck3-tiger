use iced::widget::{row, vertical_rule};
use iced::{Element, Length, Sandbox};

use crate::message::Message;
use crate::mods::Mods;
use crate::reports::Reports;

pub struct TigerApp {
    mods: Mods,
    reports: Reports,
}

impl Sandbox for TigerApp {
    type Message = Message;

    fn new() -> Self {
        Self { mods: Mods::default(), reports: Reports::new() }
    }

    fn title(&self) -> String {
        "Tiger".to_owned()
    }

    fn update(&mut self, _message: Self::Message) {}

    fn view(&self) -> Element<'_, Self::Message> {
        row![
            self.mods.view().width(Length::FillPortion(1)).padding(10),
            vertical_rule(4),
            self.reports.view().width(Length::FillPortion(2)).padding(10)
        ]
        .into()
    }
}
