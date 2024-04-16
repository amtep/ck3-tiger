use iced::widget::{column, horizontal_rule, text, Column};

use crate::message::Message;
use crate::mods::Mod;

#[derive(Default)]
pub(crate) enum Reports {
    #[default]
    Empty,
    Scanning(Mod),
    Results(Mod, String, Results),
    Failed(Mod, String),
}

impl Reports {
    pub(crate) fn view(&self) -> Column<Message> {
        match self {
            Reports::Empty => column![
                text("Reports"),
                horizontal_rule(1),
                text("Choose or add a mod on the left to begin")
            ],
            Reports::Scanning(a_mod) => column![
                text(format!("Reports for {a_mod}")),
                horizontal_rule(1),
                text("Scanning mod...")
            ],
            Reports::Results(a_mod, stderr, _results) => column![
                text(format!("Reports for {a_mod}")),
                horizontal_rule(1),
                text(stderr),
                horizontal_rule(1),
                text("TODO results")
            ],
            Reports::Failed(a_mod, stderr) => column![
                text(format!("Reports for {a_mod} (SCAN FAILED)")),
                horizontal_rule(1),
                text(stderr)
            ],
        }
    }
}

pub(crate) struct Results {}
