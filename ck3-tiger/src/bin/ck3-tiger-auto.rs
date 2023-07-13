use std::fs::{read_dir, DirEntry};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use anyhow::{bail, Result};
use console::Term;

use tiger_lib::everything::Everything;
use tiger_lib::gamedir::{find_game_directory_steam, find_paradox_directory};
use tiger_lib::modfile::ModFile;
use tiger_lib::report::{emit_reports, set_mod_root, set_output_file, set_vanilla_dir};

/// Steam's code for Crusader Kings 3
const CK3_APP_ID: &str = "1158310";

/// CK3 directory under steam library dir
const CK3_DIR: &str = "steamapps/common/Crusader Kings III";

/// A file that should be present if this is the CK3 directory
const CK3_SIGNATURE_FILE: &str = "game/events/witch_events.txt";

/// The directory under the Paradox Interactive directory for local files
const CK3_PARADOX_DIR: &str = "Crusader Kings III";

const ERROR_WAIT_SECONDS: u64 = 5;

fn main() -> Result<()> {
    #[cfg(windows)]
    let _ = ansi_term::enable_ansi_support()
            .map_err(|_| eprintln!("Failed to enable ANSI support for Windows10 users. Continuing probably without colored output."));

    // LAST UPDATED CK3 VERSION 1.9.2.1
    eprintln!("This validator was made for Crusader Kings version 1.9.2.1 (Lance).");
    eprintln!("If you are using a newer version of Crusader Kings, it may be inaccurate.");
    eprintln!("!! Currently it's inaccurate anyway because it's in beta state.");

    let mut ck3 = find_game_directory_steam(CK3_APP_ID, &PathBuf::from(CK3_DIR));
    if let Some(ref mut ck3) = ck3 {
        eprintln!("Using CK3 directory: {}", ck3.display());
        let sig = ck3.clone().join(CK3_SIGNATURE_FILE);
        if !sig.is_file() {
            eprintln!("That does not look like a CK3 directory.");
            eprintln!("Cannot find the CK3 directory.");
            eprintln!("Please try the main ck3-tiger executable from the command prompt.");
            sleep(Duration::from_secs(ERROR_WAIT_SECONDS));
            bail!("Giving up.");
        }
    } else {
        eprintln!("Cannot find the CK3 directory.");
        eprintln!("Please try the main ck3-tiger executable from the command prompt.");
        sleep(Duration::from_secs(ERROR_WAIT_SECONDS));
        bail!("Giving up.");
    }

    set_vanilla_dir(ck3.as_ref().unwrap().clone());

    let pdx = find_paradox_directory(&PathBuf::from(CK3_PARADOX_DIR));
    if pdx.is_none() {
        eprintln!("Cannot find the Paradox CK3 directory.");
        eprintln!("Please try the main ck3-tiger executable from the command prompt.");
        sleep(Duration::from_secs(ERROR_WAIT_SECONDS));
        bail!("Giving up.");
    }
    let pdx = pdx.unwrap().join("mod");
    let mut entries: Vec<_> =
        read_dir(pdx)?.filter_map(|entry| entry.ok()).filter(is_local_modfile_entry).collect();
    entries.sort_by_key(|entry| entry.file_name());

    if entries.len() == 1 {
        validate_mod(&ck3.unwrap(), &entries[0].path())?;
    } else if entries.is_empty() {
        eprintln!("Did not find any mods to validate.");
        eprintln!("Please try the main ck3-tiger executable from the command prompt.");
        sleep(Duration::from_secs(ERROR_WAIT_SECONDS));
        bail!("Giving up.");
    } else {
        eprintln!("Found several possible mods to validate:");
        for (i, entry) in entries.iter().enumerate().take(9) {
            eprintln!("{}. {}", i + 1, entry.file_name().to_str().unwrap_or(""));
        }
        let term = Term::stdout();
        // This takes me back to the 80s...
        loop {
            eprint!("\nChoose one by typing its number: ");
            let ch = term.read_char();
            if let Ok(ch) = ch {
                if ch >= '1' && ch <= '9' && ch as usize - '1' as usize <= entries.len() {
                    eprintln!();
                    validate_mod(&ck3.unwrap(), &entries[ch as usize - '1' as usize].path())?;
                    return Ok(());
                }
            } else {
                bail!("Cannot read user input. Giving up.");
            }
        }
    }

    Ok(())
}

fn validate_mod(ck3: &Path, modpath: &Path) -> Result<()> {
    let modfile = ModFile::read(modpath)?;
    let modpath = modfile.modpath();
    if !modpath.is_dir() {
        eprintln!("Looking for mod in {}", modpath.display());
        eprintln!("Cannot find mod directory. Please make sure the .mod file is correct.");
        sleep(Duration::from_secs(ERROR_WAIT_SECONDS));
        bail!("Giving up.");
    }
    eprintln!("Using mod directory: {}", modpath.display());

    set_mod_root(modpath.clone());
    let output_file = &modpath.clone().join("ck3-tiger.log");
    set_output_file(output_file)?;
    eprintln!("Writing error reports to {} ...", output_file.display());

    let mut everything = Everything::new(ck3, &modpath, modfile.replace_paths())?;

    everything.load_output_settings();
    everything.load_config_filtering_rules();
    emit_reports();

    everything.load_all();
    everything.validate_all();
    everything.check_rivers();
    emit_reports();

    Ok(())
}

fn is_local_modfile_entry(entry: &DirEntry) -> bool {
    let filename = entry.file_name();
    let name = filename.to_string_lossy();
    name.ends_with(".mod") && !name.starts_with("pdx_") && !name.starts_with("ugc")
}
