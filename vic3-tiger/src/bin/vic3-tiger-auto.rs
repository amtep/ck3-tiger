use std::fs::{read_dir, DirEntry};
use std::mem::forget;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use console::Term;

use tiger_lib::{
    emit_reports, find_game_directory_steam, find_paradox_directory, set_output_file, Everything,
    Game, ModMetadata,
};

/// Steam's code for Victoria 3
const VIC3_APP_ID: &str = "529340";

/// VIC3 directory under steam library dir
const VIC3_DIR: &str = "steamapps/common/Victoria 3";

/// A file that should be present if this is the VIC3 directory
const VIC3_SIGNATURE_FILE: &str = "game/events/titanic_events.txt";

/// The directory under the Paradox Interactive directory for local files
const VIC3_PARADOX_DIR: &str = "Victoria 3";

fn main() {
    match inner_main() {
        Ok(_) => (),
        Err(e) => {
            eprintln!();
            eprintln!("ERROR: {e:#}");
            eprintln!("Please try the main vic3-tiger executable from the command prompt.");
            eprintln!("Press any key to exit.");
            let term = Term::stdout();
            _ = term.read_char();
        }
    }
}

fn inner_main() -> Result<()> {
    // Colors are off by default, but enable ANSI support in case the config file turns colors on again.
    #[cfg(windows)]
    let _ = ansiterm::enable_ansi_support().map_err(|_| {
        eprintln!("Failed to enable ANSI support for Windows10 users. Continuing anyway.")
    });

    // LAST UPDATED VIC3 VERSION 1.6.0
    eprintln!("This validator was made for Victoria 3 version 1.6.0 (Blackcurrant).");
    eprintln!("If you are using a newer version of Victoria 3, it may be inaccurate.");
    eprintln!("!! Currently it's inaccurate anyway because it's in beta state.");

    Game::set(Game::Vic3)?;

    let mut vic3 = find_game_directory_steam(VIC3_APP_ID, &PathBuf::from(VIC3_DIR));
    if let Some(ref mut vic3) = vic3 {
        eprintln!("Using Vic33 directory: {}", vic3.display());
        let sig = vic3.clone().join(VIC3_SIGNATURE_FILE);
        if !sig.is_file() {
            eprintln!("That does not look like a Vic3 directory.");
            bail!("Cannot find the Vic3 directory.");
        }
    } else {
        bail!("Cannot find the Vic3 directory.");
    }

    let pdx = find_paradox_directory(&PathBuf::from(VIC3_PARADOX_DIR));
    if pdx.is_none() {
        bail!("Cannot find the Paradox Vic3 directory.");
    }
    let pdx = pdx.unwrap();
    let pdxmod = pdx.join("mod");
    let pdxlogs = pdx.join("logs");

    let mut entries: Vec<_> =
        read_dir(pdxmod)?.filter_map(|entry| entry.ok()).filter(is_local_mod_entry).collect();
    entries.sort_by_key(|entry| entry.file_name());

    if entries.len() == 1 {
        validate_mod(&vic3.unwrap(), &entries[0].path(), &pdxlogs)?;
    } else if entries.is_empty() {
        bail!("Did not find any mods to validate.");
    } else {
        eprintln!("Found several possible mods to validate:");
        for (i, entry) in entries.iter().enumerate().take(35) {
            let ordinal = i + 1;
            if ordinal <= 9 {
                eprintln!("{ordinal}. {}", entry.file_name().to_str().unwrap_or(""));
            } else {
                let modkey = char::from_u32((ordinal - 10 + 'A' as usize) as u32).unwrap();
                eprintln!("{modkey}. {}", entry.file_name().to_str().unwrap_or(""));
            }
        }
        let term = Term::stdout();
        // This takes me back to the 80s...
        loop {
            eprint!("\nChoose one by typing its key: ");
            let ch = term.read_char();
            if let Ok(ch) = ch {
                let modnr = if ('1'..='9').contains(&ch) {
                    ch as usize - '1' as usize
                } else if ch.is_ascii_lowercase() {
                    9 + ch as usize - 'a' as usize
                } else if ch.is_ascii_uppercase() {
                    9 + ch as usize - 'A' as usize
                } else {
                    continue;
                };
                if modnr < entries.len() {
                    eprintln!();
                    validate_mod(&vic3.unwrap(), &entries[modnr].path(), &pdxlogs)?;
                    return Ok(());
                }
            } else {
                bail!("Cannot read user input. Giving up.");
            }
        }
    }

    Ok(())
}

fn validate_mod(vic3: &Path, modpath: &Path, logdir: &Path) -> Result<()> {
    let metadata = ModMetadata::read(modpath)?;
    eprintln!("Using mod directory: {}", modpath.display());

    let output_filename =
        format!("vic3-tiger-{}.log", modpath.file_name().unwrap().to_string_lossy());
    let output_file = &logdir.join(output_filename);
    set_output_file(output_file)?;
    eprintln!("Writing error reports to {} ...", output_file.display());
    eprintln!("This will take a few seconds.");

    let mut everything = Everything::new(None, Some(vic3), modpath, metadata.replace_paths())?;

    // Unfortunately have to disable the colors by default because
    // on Windows there's no easy way to view a file that contains those escape sequences.
    // There are workarounds but those defeat the purpose of -auto.
    // The colors can be enabled again in the vic3-tiger.conf file.
    everything.load_output_settings(false);
    everything.load_config_filtering_rules();
    emit_reports(false);

    everything.load_all();
    everything.validate_all();
    everything.check_rivers();
    emit_reports(false);

    // Properly dropping `everything` takes a noticeable amount of time, and we're exiting anyway.
    forget(everything);

    Ok(())
}

fn is_local_mod_entry(entry: &DirEntry) -> bool {
    entry.path().join(".metadata/metadata.json").is_file()
}
