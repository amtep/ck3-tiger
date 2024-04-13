//! The main() function for the executable.

use anyhow::Result;
use iced::{Application, Settings};

use tiger_gui::TigerApp;

fn main() -> Result<()> {
    TigerApp::run(Settings::default())?;
    Ok(())
}
