use iced::widget::{column, Column, Text};

use crate::message::Message;

pub(crate) struct Reports {}

impl Reports {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn view(&self) -> Column<Message> {
        column![Text::new("Reports"), Text::new("no reports yet")]
    }
}
