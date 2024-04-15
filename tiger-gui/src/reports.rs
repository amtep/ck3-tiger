use iced::widget::{column, text, Column};

use crate::message::Message;

pub(crate) struct Reports {}

impl Reports {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn view(&self) -> Column<Message> {
        column![text("Reports"), text("no reports yet")]
    }
}
