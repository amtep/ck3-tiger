use anyhow::{bail, Result};
use clap::Parser;
use home::home_dir;
use std::ffi::OsStr;
use std::path::PathBuf;
use walkdir::WalkDir;

use ck3_mod_validator::{Everything, ModFile};

const CK3_RECOGNITION_FILE: &str = "events/councillor_task_events/steward_task_events.txt";

const KNOWN_LANGUAGES: [&str; 7] = [
    "english",
    "spanish",
    "french",
    "german",
    "russian",
    "korean",
    "simp_chinese",
];

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

fn get_file_lang(filename: &OsStr) -> Option<&str> {
    let filename = filename.to_str()?;
    for lang in KNOWN_LANGUAGES {
        if filename.ends_with(&format!("_l_{}.yml", lang)) {
            return Some(lang);
        }
    }
    None
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

    for entry in WalkDir::new(modpath.join("localization")) {
        match entry {
            Ok(entry) => {
                if entry.depth() == 0 {
                    continue;
                }
                let inner_path = entry.path().strip_prefix(&modpath)?;
                if entry.depth() == 1 && !entry.file_type().is_dir() {
                    eprintln!("found file in wrong location: {}", inner_path.display());
                    eprintln!("localization files should be in subdirectories according to their language.");
                }
                let lang = inner_path
                    .components()
                    .nth(1)
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy();
                if !KNOWN_LANGUAGES.contains(&&*lang) && lang != "replace" {
                    // check depth, to warn only once
                    if entry.depth() == 1 {
                        eprintln!(
                            "Unknown subdirectory in localization: {}",
                            inner_path.display()
                        );
                        eprintln!(
                            "Valid subdirectories are {} and replace",
                            KNOWN_LANGUAGES.join(", ")
                        );
                    }
                    continue;
                }
                if !entry.file_type().is_file() {
                    continue;
                }
                let filename = entry.file_name();
                let filelang = get_file_lang(filename);
                if filelang.is_none() {
                    eprintln!(
                        "could not determine language from filename: {}",
                        inner_path.display()
                    );
                    eprintln!(
                        "localization filenames should end in _l_language.yml, where language is"
                    );
                    eprintln!("one of {}", KNOWN_LANGUAGES.join(", "));
                    continue;
                }
                if filelang.unwrap() != lang {
                    eprintln!("localization file with wrong name or in wrong directory:");
                    eprintln!("  {}", inner_path.display());
                    eprintln!("a localization file should be in a subdirectory corresponding to its language.");
                }
            }
            Err(e) => eprintln!("{:#}", e),
        }
    }

    let everything = Everything::new(args.ck3.unwrap(), modpath);

    Ok(())
}
