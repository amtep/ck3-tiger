use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use home::home_dir;

#[cfg(windows)]
use winreg::enums::HKEY_LOCAL_MACHINE;
#[cfg(windows)]
use winreg::RegKey;

// How to find steamapps dir on different systems
const STEAM_LINUX: &str = ".local/share/Steam/steamapps";
const STEAM_LINUX_PROTON: &str = ".steam/steam/steamapps";
const STEAM_MAC: &str = "Library/Application Support/Steam/steamapps";
#[cfg(windows)]
const STEAM_WINDOWS_KEY: &str = r"SOFTWARE\Wow6432Node\Valve\Steam";

/// Tries to locate the game files.
/// If there are several locations where the files may reside, it will try them one by one.
/// `steam_app_id` is the (normally numeric) id of the game on Steam.
/// You can find it by going to the game's store page in your web browser. The id will be after `/app/` in the url.
/// `game_dir` is the path to the game's files under the steam path for the app. Usually `steamapps/common/...`.
pub fn find_game_directory_steam(steam_app_id: &str, game_dir: &Path) -> Option<PathBuf> {
    if let Some(home) = home_dir() {
        let on_linux = find_game_dir_in_steam_dir(home.join(STEAM_LINUX), steam_app_id, game_dir);
        if on_linux.is_some() {
            return on_linux;
        }
        let on_linux_proton =
            find_game_dir_in_steam_dir(home.join(STEAM_LINUX_PROTON), steam_app_id, game_dir);
        if on_linux_proton.is_some() {
            return on_linux_proton;
        }
        let on_mac = find_game_dir_in_steam_dir(home.join(STEAM_MAC), steam_app_id, game_dir);
        if on_mac.is_some() {
            return on_mac;
        }
    }
    #[cfg(windows)]
    {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(STEAM_WINDOWS_KEY)
            .ok()?;
        let on_windows: String = key.get_value("InstallPath").ok()?;
        let on_windows = PathBuf::from(on_windows).join("steamapps");
        return find_game_dir_in_steam_dir(on_windows, steam_app_id, game_dir);
    }
    None
}

/// Tries to find the game's directory inside a steamapps/ directory.
/// Returns None if the steamapps/ directory doesn't exist, or isn't really a steamapps directory,
/// or doesn't contain the game.
fn find_game_dir_in_steam_dir(
    steam_dir: PathBuf,
    app_id: &str,
    game_dir: &Path,
) -> Option<PathBuf> {
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
                found_path = Some(PathBuf::from(value))
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
