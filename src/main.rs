use anyhow::{bail, Result};
use clap::Parser;
use home::home_dir;
use std::path::PathBuf;

use ck3_mod_validator::everything::Everything;
use ck3_mod_validator::modfile::ModFile;

const CK3_RECOGNITION_FILE: &str = "events/councillor_task_events/steward_task_events.txt";

#[derive(Parser)]
struct Cli {
    /// Path to .mod file of mod to check.
    modpath: PathBuf,
    /// Path to CK3 game directory.
    #[clap(long)]
    ck3: Option<PathBuf>,
}

fn try_ck3_directory(dir: PathBuf) -> Option<PathBuf> {
    let mut recognize = dir.clone();
    recognize.push(CK3_RECOGNITION_FILE);
    if recognize.exists() {
        Some(dir)
    } else {
        None
    }
}

fn find_ck3_directory() -> Option<PathBuf> {
    // Linux default
    home_dir()
        .and_then(|mut home| {
            home.push(".local/share/Steam/steamapps/common/Crusader Kings III/game");
            try_ck3_directory(home)
        })
        .or_else(|| {
            // Windows default
            let mut path = PathBuf::new();
            path.push("C:/Program Files (x86)/Steam/steamapps/common/Crusader Kings III/game");
            try_ck3_directory(path)
        })
        .or_else(|| {
            // Mac default
            home_dir().and_then(|mut home| {
                home.push(
                    "Library/Application Support/Steam/steamapps/common/Crusader Kings III/game",
                );
                try_ck3_directory(home)
            })
        })
}

fn main() -> Result<()> {
    let mut args = Cli::parse();

    if args.ck3.is_none() {
        args.ck3 = find_ck3_directory();
    }
    if args.ck3.is_none() {
        bail!("Cannot find CK3 game directory. Please supply it as the --ck3 option.");
    }

    let modfile = ModFile::read(&args.modpath)?;
    let modpath = modfile.modpath();
    if !modpath.exists() {
        eprintln!("Looking for mod in {}", modpath.display());
        bail!("Cannot find mod directory. Please make sure the .mod file is correct.");
    }

    let mut everything = Everything::new(args.ck3.unwrap(), modpath)?;
    everything.load_localizations();

    Ok(())
}
