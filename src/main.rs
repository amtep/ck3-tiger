use anyhow::{bail, Result};
use clap::Parser;
use home::home_dir;
use std::path::PathBuf;

use ck3_mod_validator::everything::{Everything, FileKind};
use ck3_mod_validator::modfile::ModFile;

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

fn get_file_lang(filename: &str) -> Option<&'static str> {
    for lang in KNOWN_LANGUAGES {
        if filename.ends_with(&format!("l_{}.yml", lang)) {
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

    let everything = Everything::new(args.ck3.unwrap(), modpath)?;

    let mut warned_dirs = Vec::new();
    for entry in everything.get_files_under("localization") {
        if entry.kind() != FileKind::ModFile {
            continue;
        }
        if entry.path().len() == 2 && !entry.filename().ends_with(".info") {
            eprintln!();
            eprintln!("{}: file in wrong location", entry);
            eprintln!(
                "Localization files should be in subdirectories according to their language."
            );
            continue;
        }
        let lang: &str = &entry.path()[1];
        if lang != "replace" && !KNOWN_LANGUAGES.contains(&lang) {
            if !warned_dirs.contains(&lang) {
                eprintln!();
                eprintln!("{}: unknown subdirectory in localization", entry);
                eprintln!(
                    "Valid subdirectories are {} and replace",
                    KNOWN_LANGUAGES.join(", ")
                );
            }
            warned_dirs.push(lang);
            continue;
        }
        if let Some(filelang) = get_file_lang(entry.filename()) {
            if filelang != lang && lang != "replace" {
                eprintln!();
                eprintln!(
                    "{}: localization file with wrong name or in wrong directory",
                    entry
                );
                eprintln!("A localization file should be in a subdirectory corresponding to its language.");
            }
        } else {
            eprintln!();
            eprintln!("{}: could not determine language from filename", entry);
            eprintln!(
                "Localization filenames should end in _l_language.yml, where language is one of {}",
                KNOWN_LANGUAGES.join(", ")
            );
        }
    }

    Ok(())
}
