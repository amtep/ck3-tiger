use std::process;

use iced::widget::{column, container, horizontal_rule, text, Column};

use crate::message::Message;
use crate::mods::Mod;

#[derive(Debug, Default)]
pub(crate) enum Reports {
    #[default]
    Empty,
    Scanning(Mod),
    Results(Mod, Results),
    Failed(Mod, String),
}

impl Reports {
    pub(crate) fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::ShowResults(a_mod) => {
                *self = Reports::Scanning(a_mod.clone());
                iced::Command::perform(
                    scan(a_mod.game.executable().to_owned(), a_mod),
                    Message::ModScanned,
                )
            }
            Message::ModScanned((a_mod, results)) => {
                // Only handle results for the mod that's currently being scanned.
                // Discard any others.
                if let Reports::Scanning(current_mod) = self {
                    if &a_mod == current_mod {
                        *self = match results {
                            Ok(results) => Reports::Results(a_mod, results),
                            Err(stderr) => Reports::Failed(a_mod, stderr),
                        }
                    }
                }
                iced::Command::none()
            }
        }
    }

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
            Reports::Results(a_mod, results) => column![
                text(format!("Reports for {a_mod}")),
                horizontal_rule(1),
                container(text(results.stderr.trim())).padding(10),
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

#[derive(Debug, Clone)]
pub struct Results {
    stderr: String,
}

/// Run tiger on a mod and return the parsed results.
/// On failure, return tiger's stderr output or a message that tiger couldn't be run.
async fn scan(tiger: String, the_mod: Mod) -> (Mod, Result<Results, String>) {
    match process::Command::new(&tiger).arg("--json").arg(&the_mod.locator).output() {
        Err(e) => (the_mod, Err(format!("Could not run {tiger}: {e}"))),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if output.status.success() {
                (the_mod, Ok(Results { stderr }))
            } else {
                (the_mod, Err(stderr))
            }
        }
    }
}
