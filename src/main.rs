use anyhow::{bail, Result};
use clap::Parser;
use home::home_dir;
use std::fs::read_to_string;
use std::path::PathBuf;

#[cfg(windows)]
use winreg::enums::HKEY_LOCAL_MACHINE;
#[cfg(windows)]
use winreg::RegKey;

use ck3_tiger::errorkey::ErrorKey;
use ck3_tiger::errors::{
    ignore_key, minimum_level, set_mod_root, set_vanilla_root, show_vanilla, ErrorLevel,
};
use ck3_tiger::everything::Everything;
use ck3_tiger::modfile::ModFile;

/// Steam's code for Crusader Kings 3
const CK3_APP_ID: &str = "1158310";

// How to find steamapps dir on different systems
const STEAM_LINUX: &str = ".local/share/Steam/steamapps";
const STEAM_MAC: &str = "Library/Application Support/Steam/steamapps";
#[cfg(windows)]
const STEAM_WINDOWS_KEY: &str = r"SOFTWARE\Wow6432Node\Valve\Steam";

/// CK3 directory under steam library dir
const CK3_GAME_DIR: &str = "steamapps/common/Crusader Kings III/game";

/// A file that should be present if this is a CK3 game directory
const CK3_SIGNATURE_FILE: &str = "events/witch_events.txt";

#[derive(Parser)]
struct Cli {
    /// Path to .mod file of mod to check.
    modpath: PathBuf,
    /// Path to CK3 game directory.
    #[clap(long)]
    ck3: Option<PathBuf>,
    /// Show errors in the base CK3 script code as well
    #[clap(long)]
    show_vanilla: bool,
    /// Show advice in addition to warnings and errors
    #[clap(long)]
    advice: bool,
    /// Warn about items that are defined but unused
    #[clap(long)]
    unused: bool,
    /// Do checks specific to the Princes of Darkness mod
    #[clap(long)]
    pod: bool,
}

fn find_steamapps_directory() -> Option<PathBuf> {
    if let Some(home) = home_dir() {
        let on_linux = home.join(STEAM_LINUX);
        if on_linux.is_dir() {
            return Some(on_linux);
        }
        let on_mac = home.join(STEAM_MAC);
        if on_mac.is_dir() {
            return Some(on_mac);
        }
    }
    #[cfg(windows)]
    {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(STEAM_WINDOWS_KEY)
            .ok()?;
        let on_windows: String = key.get_value("InstallPath").ok()?;
        let on_windows = PathBuf::from(on_windows).join("steamapps");
        if on_windows.is_dir() {
            return Some(on_windows);
        }
    }
    None
}

fn find_ck3_directory() -> Option<PathBuf> {
    let steamapps_dir = find_steamapps_directory()?;

    let vdf = steamapps_dir.join("libraryfolders.vdf");
    // Rudimentary libraryfolders.vdf parsing.
    // We're looking for a subsection with a "path" setting that has
    // our app (CK3) listed in its "apps" list.
    let mut found_path = None;
    for line in read_to_string(vdf).ok()?.lines() {
        let fields = line.split_ascii_whitespace().collect::<Vec<&str>>();
        if fields.len() == 2 {
            let key = fields[0].trim_matches('"');
            let value = fields[1].trim_matches('"');
            if key == "path" {
                found_path = Some(PathBuf::from(value))
            } else if key == CK3_APP_ID && found_path.is_some() {
                let ck3_path = found_path.unwrap().join(CK3_GAME_DIR);
                if ck3_path.is_dir() {
                    return Some(ck3_path);
                }
                return None;
            }
        }
    }
    None
}

fn main() -> Result<()> {
    let mut args = Cli::parse();

    // LAST UPDATED VERSION 1.9.2.1
    eprintln!("This validator was made for Crusader Kings version 1.9.2.1 (Lance).");
    eprintln!("If you are using a newer version of Crusader Kings, it may be inaccurate.");
    eprintln!("!! Currently it's inaccurate anyway because it's in beta state.");

    if args.ck3.is_none() {
        args.ck3 = find_ck3_directory();
    }
    if let Some(ref mut ck3) = args.ck3 {
        eprintln!("Using CK3 game directory: {}", ck3.display());
        let mut sig = ck3.clone();
        sig.push(CK3_SIGNATURE_FILE);
        if !sig.is_file() {
            eprintln!("That does not look like a CK3 game directory.");
            ck3.push("game");
            eprintln!("Trying: {}", ck3.display());
            sig = ck3.clone();
            sig.push(CK3_SIGNATURE_FILE);
            if sig.is_file() {
                eprintln!("Ok.");
            } else {
                bail!("Cannot find CK3 game directory. Please supply it as the --ck3 option.");
            }
        }
    } else {
        bail!("Cannot find CK3 game directory. Please supply it as the --ck3 option.");
    }

    set_vanilla_root(args.ck3.as_ref().unwrap().clone());

    if args.show_vanilla {
        eprintln!("Showing warnings for base game files too. There will be many false positives in those.");
        show_vanilla(true);
    }

    if !args.advice {
        minimum_level(ErrorLevel::Info);
    }

    if args.unused {
        eprintln!("Showing warnings for unused localization. There will be many false positives.");
    } else {
        ignore_key(ErrorKey::UnusedLocalization);
    }

    if args.modpath.is_dir() {
        args.modpath.push("descriptor.mod");
    }
    let modfile = ModFile::read(&args.modpath)?;
    let modpath = modfile.modpath();
    if !modpath.exists() {
        eprintln!("Looking for mod in {}", modpath.display());
        bail!("Cannot find mod directory. Please make sure the .mod file is correct.");
    }
    eprintln!("Using mod directory: {}", modpath.display());
    set_mod_root(modpath.clone());

    let mut everything = Everything::new(&args.ck3.unwrap(), &modpath, modfile.replace_paths())?;
    everything.load_all();
    everything.validate_all();
    everything.check_rivers();
    if args.pod {
        everything.check_pod();
    }

    Ok(())
}
