//! The main() function for the executable.

use std::env::{current_exe, set_current_dir};

use anyhow::Result;
use iced::{Application, Settings};

use tiger_gui::TigerApp;

fn main() -> Result<()> {
    // Set the current directory to the executable's directory, to make sure the default paths
    // "./ck3-tiger" etc will work if the tiger executables are in the same directory.
    set_current_dir(current_exe()?.canonicalize()?.parent().expect("executable is a file"))?;
    TigerApp::run(Settings::default())?;
    Ok(())
}
