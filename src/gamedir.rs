//! Helper functions for finding the base and mod directories of the game being validated.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use home::home_dir;

#[cfg(windows)]
use winreg::enums::HKEY_LOCAL_MACHINE;
#[cfg(windows)]
use winreg::RegKey;

// How to find steamapps dir on different systems
const STEAM_LINUX: &str = ".local/share/Steam";
const STEAM_LINUX_PROTON: &str = ".steam/steam";
const STEAM_MAC: &str = "Library/Application Support/Steam";
const STEAM_WINDOWS: &str = "C:/Program Files (x86)/Steam";
#[cfg(windows)]
const STEAM_WINDOWS_KEY: &str = r"SOFTWARE\Wow6432Node\Valve\Steam";

// How to find the paradox local files dir on different systems
const PDX_LINUX: &str = ".local/share/Paradox Interactive";
const PDX_MAC: &str = "Library/Application Support/Paradox Interactive"; // TODO this is a guess
const PDX_WINDOWS: &str = "Documents/Paradox Interactive";

/// Tries to locate the game files.
/// If there are several locations where the files may reside, it will try them one by one.
/// `steam_app_id` is the (normally numeric) id of the game on Steam.
/// You can find it by going to the game's store page in your web browser. The id will be after `/app/` in the url.
/// `game_dir` is the path to the game's files under the steam path for the app. Usually `steamapps/common/...`.
pub fn find_game_directory_steam(steam_app_id: &str, game_dir: &Path) -> Option<PathBuf> {
    if let Some(home) = home_dir() {
        for try_dir in &[STEAM_LINUX, STEAM_LINUX_PROTON, STEAM_MAC] {
            let steam_dir = find_game_dir_in_steam_dir(
                &home.join(try_dir).join("steamapps"),
                steam_app_id,
                game_dir,
            );
            if steam_dir.is_some() {
                return steam_dir;
            }
            // Try the default directory too
            let steam_dir = &home.join(try_dir).join(game_dir);
            if steam_dir.is_dir() {
                return Some(steam_dir.clone());
            }
        }
    }

    let on_windows = find_game_dir_in_steam_dir(
        &PathBuf::from(STEAM_WINDOWS).join("steamapps"),
        steam_app_id,
        game_dir,
    );
    if on_windows.is_some() {
        return on_windows;
    }
    let on_windows = PathBuf::from(STEAM_WINDOWS).join(game_dir);
    if on_windows.is_dir() {
        return Some(on_windows);
    }

    // If the game is not in the default dirs, go via the registry to find Steam and then find the game
    #[cfg(windows)]
    {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(STEAM_WINDOWS_KEY).ok()?;
        let on_windows: String = key.get_value("InstallPath").ok()?;
        let on_windows = PathBuf::from(on_windows).join("steamapps");
        return find_game_dir_in_steam_dir(&on_windows, steam_app_id, game_dir);
    }
    None
}

/// Tries to find the game's directory inside a steamapps/ directory.
/// Returns None if the steamapps/ directory doesn't exist, or isn't really a steamapps directory,
/// or doesn't contain the game.
fn find_game_dir_in_steam_dir(steam_dir: &Path, app_id: &str, game_dir: &Path) -> Option<PathBuf> {
    if !steam_dir.is_dir() {
        return None;
    }
    let vdf = steam_dir.join("libraryfolders.vdf");
    // Rudimentary libraryfolders.vdf parsing.
    // We're looking for a subsection with a "path" setting that has
    // our app listed in its "apps" list.
    let mut found_path = None;
    for line in read_to_string(vdf).ok()?.lines() {
        let fields = line.split_ascii_whitespace().collect::<Vec<&str>>();
        if fields.len() == 2 {
            let key = fields[0].trim_matches('"');
            let value = fields[1].trim_matches('"');
            if key == "path" {
                found_path = Some(PathBuf::from(value));
            } else if key == app_id && found_path.is_some() {
                let game_path = found_path.unwrap().join(game_dir);
                if game_path.is_dir() {
                    return Some(game_path);
                }
                return None;
            }
        }
    }
    None
}

pub fn find_paradox_directory(dir_under: &Path) -> Option<PathBuf> {
    if let Some(home) = home_dir() {
        let on_linux = home.join(PDX_LINUX).join(dir_under);
        if on_linux.is_dir() {
            return Some(on_linux);
        }
        let on_mac = home.join(PDX_MAC).join(dir_under);
        if on_mac.is_dir() {
            return Some(on_mac);
        }
        let on_windows = home.join(PDX_WINDOWS).join(dir_under);
        if on_windows.is_dir() {
            return Some(on_windows);
        }
    }
    None
}
