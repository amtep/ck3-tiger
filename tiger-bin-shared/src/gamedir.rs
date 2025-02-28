//! Helper functions for finding the base and mod directories of the game being validated.

use std::path::{Path, PathBuf};
use std::env::home_dir;

use anyhow::{bail, Result};
use steamlocate::SteamDir;

// How to find the paradox local files dir on different systems
const PDX_LINUX: &str = ".local/share/Paradox Interactive";
const PDX_MAC: &str = "Library/Application Support/Paradox Interactive"; // TODO this is a guess
const PDX_WINDOWS: &str = "Documents/Paradox Interactive";

/// Tries to locate the game files.
/// `steam_app_id` is the numeric id of the game on Steam.
/// You can find it by going to the game's store page in your web browser. The id will be after `/app/` in the url.
pub fn find_game_directory_steam(steam_app_id: u32) -> Result<PathBuf> {
    let steamdir = SteamDir::locate()?;
    if let Some((app, library)) = steamdir.find_app(steam_app_id)? {
        Ok(library.resolve_app_dir(&app))
    } else {
        bail!("Game not found in Steam library")
    }
}

pub fn find_paradox_directory(dir_under: &Path) -> Option<PathBuf> {
    if let Some(home) = home_dir() {
        for try_dir in &[PDX_LINUX, PDX_MAC, PDX_WINDOWS] {
            let full_try_dir = home.join(try_dir).join(dir_under);
            if full_try_dir.is_dir() {
                return Some(fix_slashes_for_target_platform(full_try_dir));
            }
        }
    }
    None
}

/// Redo a path so that all the slashes lean the correct way for the target platform.
/// This is mostly for Windows users, to avoid showing them paths with a mix of slashes.
fn fix_slashes_for_target_platform<P: std::borrow::Borrow<Path>>(path: P) -> PathBuf {
    path.borrow().components().collect()
}
