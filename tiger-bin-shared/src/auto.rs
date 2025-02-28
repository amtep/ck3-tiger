use std::fs::{DirEntry, read_dir};
use std::mem::forget;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use console::Term;
#[cfg(any(feature = "ck3", feature = "imperator"))]
use tiger_lib::ModFile;
#[cfg(feature = "vic3")]
use tiger_lib::ModMetadata;
use tiger_lib::{Everything, emit_reports, set_output_file};

use crate::GameConsts;
use crate::gamedir::{find_game_directory_steam, find_paradox_directory};

/// Run the automatic version of the tiger application.
///
/// It can search the paradox mod folder, detect mods and list them for user selection. However,
/// it has **no** command line arguments and hence less customizable compared to the main application.
pub fn run(game_consts: &GameConsts) -> Result<()> {
    let &GameConsts { name, name_short, version, app_id, signature_file, paradox_dir } =
        game_consts;

    // Colors are off by default, but enable ANSI support in case the config file turns colors on again.
    #[cfg(windows)]
    let _ = ansiterm::enable_ansi_support().map_err(|_| {
        eprintln!("Failed to enable ANSI support for Windows10 users. Continuing anyway.")
    });

    eprintln!("This validator was made for {name} version {version}.");
    eprintln!("If you are using a newer version of {name}, it may be inaccurate.");
    eprintln!("!! Currently it's inaccurate anyway because it's in beta state.");

    let game = find_game_directory_steam(app_id).context("Cannot find the game directory.")?;
    eprintln!("Using {name_short} directory: {}", game.display());
    let sig = game.clone().join(signature_file);
    if !sig.is_file() {
        eprintln!("That does not look like a {name_short} directory.");
        bail!("Cannot find the game directory.");
    }

    let pdx = get_paradox_directory(&PathBuf::from(paradox_dir))?;
    let pdxmod = pdx.join("mod");
    let pdxlogs = pdx.join("logs");

    let mut entries: Vec<_> =
        read_dir(pdxmod)?.filter_map(Result::ok).filter(is_local_mod_entry).collect();
    entries.sort_by_key(DirEntry::file_name);

    if entries.len() == 1 {
        validate_mod(name_short, &game, &entries[0].path(), &pdxlogs)?;
    } else if entries.is_empty() {
        bail!("Did not find any mods to validate.");
    } else {
        eprintln!("Found several possible mods to validate:");
        for (i, entry) in entries.iter().enumerate().take(35) {
            #[allow(clippy::cast_possible_truncation)] // known to be <= 35
            let ordinal = (i + 1) as u32;
            if ordinal <= 9 {
                eprintln!("{}. {}", ordinal, entry.file_name().to_str().unwrap_or(""));
            } else {
                let modkey = char::from_u32(ordinal - 10 + 'A' as u32).unwrap_or('?');
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
                    validate_mod(name_short, &game, &entries[modnr].path(), &pdxlogs)?;
                    return Ok(());
                }
            } else {
                bail!("Cannot read user input. Giving up.");
            }
        }
    }

    Ok(())
}

#[allow(unused_mut)]
fn validate_mod(
    name_short: &'static str,
    game: &Path,
    modpath: &Path,
    logdir: &Path,
) -> Result<()> {
    let mut everything;
    let mut modpath = modpath;

    #[cfg(any(feature = "ck3", feature = "imperator"))]
    let modfile = ModFile::read(modpath)?;
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    let modpath_owned = modfile.modpath();
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    {
        modpath = &modpath_owned;
        if !modpath.is_dir() {
            eprintln!("Looking for mod in {}", modpath.display());
            bail!("Cannot find mod directory. Please make sure the .mod file is correct.");
        }
    }

    eprintln!("Using mod directory: {}", modpath.display());
    let output_filename =
        format!("{name_short}-tiger-{}.log", modpath.file_name().unwrap().to_string_lossy());
    let output_file = &logdir.join(output_filename);
    set_output_file(output_file)?;
    eprintln!("Writing error reports to {} ...", output_file.display());
    eprintln!("This will take a few seconds.");

    #[cfg(any(feature = "ck3", feature = "imperator"))]
    {
        everything = Everything::new(None, Some(game), modpath, modfile.replace_paths())?;
    }
    #[cfg(feature = "vic3")]
    {
        let metadata = ModMetadata::read(modpath)?;
        everything = Everything::new(None, Some(game), modpath, metadata.replace_paths())?;
    }

    // Unfortunately have to disable the colors by default because
    // on Windows there's no easy way to view a file that contains those escape sequences.
    // There are workarounds but those defeat the purpose of -auto.
    // The colors can be enabled again in the config file.
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
    #[cfg(any(feature = "ck3", feature = "imperator"))]
    {
        let filename = entry.file_name();
        let name = filename.to_string_lossy();
        name.ends_with(".mod") && !name.starts_with("pdx_") && !name.starts_with("ugc")
    }
    #[cfg(feature = "vic3")]
    {
        entry.path().join(".metadata/metadata.json").is_file()
    }
}

fn get_paradox_directory(paradox_dir: &Path) -> Result<PathBuf> {
    if let Some(pdx) = find_paradox_directory(paradox_dir) {
        Ok(pdx)
    } else {
        bail!("Cannot find the Paradox directory.");
    }
}
