use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use tiger_lib::everything::Everything;
use tiger_lib::gamedir::find_game_directory_steam;
use tiger_lib::modfile::ModFile;
use tiger_lib::report::{
    disable_ansi_colors, emit_reports, set_mod_root, set_show_loaded_mods, set_show_vanilla,
    set_vanilla_dir,
};

/// Steam's code for Crusader Kings 3
const CK3_APP_ID: &str = "1158310";

/// CK3 directory under steam library dir
const CK3_DIR: &str = "steamapps/common/Crusader Kings III";

/// A file that should be present if this is the CK3 directory
const CK3_SIGNATURE_FILE: &str = "game/events/witch_events.txt";

#[derive(Parser)]
struct Cli {
    /// Path to .mod file of mod to check.
    modpath: PathBuf,
    /// Path to CK3 main directory.
    #[clap(long)]
    ck3: Option<PathBuf>,
    /// Show errors in the base CK3 script code as well
    #[clap(long)]
    show_vanilla: bool,
    /// Show errors in other loaded mods as well
    #[clap(long)]
    show_mods: bool,
    /// Warn about items that are defined but unused
    #[clap(long)]
    unused: bool,
    /// Do checks specific to the Princes of Darkness mod
    #[clap(long)]
    pod: bool,
    /// Omit color from the output. False by default.
    /// Can also be configured in the ck3-tiger.conf file.
    #[clap(long)]
    no_color: bool,
}

fn main() -> Result<()> {
    let mut args = Cli::parse();

    #[cfg(windows)]
    if !args.no_color {
        let _ = ansi_term::enable_ansi_support()
            .map_err(|_| eprintln!("Failed to enable ANSI support for Windows10 users. Continuing probably without colored output."));
    }

    // LAST UPDATED VERSION 1.9.2.1
    eprintln!("This validator was made for Crusader Kings version 1.9.2.1 (Lance).");
    eprintln!("If you are using a newer version of Crusader Kings, it may be inaccurate.");
    eprintln!("!! Currently it's inaccurate anyway because it's in beta state.");

    if args.ck3.is_none() {
        args.ck3 = find_game_directory_steam(CK3_APP_ID, &PathBuf::from(CK3_DIR));
    }
    if let Some(ref mut ck3) = args.ck3 {
        eprintln!("Using CK3 directory: {}", ck3.display());
        let mut sig = ck3.clone();
        sig.push(CK3_SIGNATURE_FILE);
        if !sig.is_file() {
            eprintln!("That does not look like a CK3 directory.");
            ck3.push("..");
            eprintln!("Trying: {}", ck3.display());
            sig = ck3.clone();
            sig.push(CK3_SIGNATURE_FILE);
            if sig.is_file() {
                eprintln!("Ok.");
            } else {
                bail!("Cannot find CK3 directory. Please supply it as the --ck3 option.");
            }
        }
    } else {
        bail!("Cannot find CK3 directory. Please supply it as the --ck3 option.");
    }

    set_vanilla_dir(args.ck3.as_ref().unwrap().clone());

    if args.show_vanilla {
        eprintln!("Showing warnings for base game files too. There will be many false positives in those.");
    }

    if args.show_mods {
        eprintln!("Showing warnings for other loaded mods too.");
    }

    if args.unused {
        eprintln!("Showing warnings for unused localization. There will be many false positives.");
    }

    if args.pod {
        eprintln!("Doing special checks for the Princes of Darkness mod.");
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

    // Print a blank line between the preamble and the first report:
    eprintln!();

    everything.load_output_settings(true);
    everything.load_config_filtering_rules();
    emit_reports();

    // We must apply the --no-color flag AFTER loading and applying the config,
    // because we want it to override the config.
    if args.no_color {
        disable_ansi_colors();
    }
    // Same logic applies to showing vanilla and other mods
    if args.show_vanilla {
        set_show_vanilla(true);
    }
    if args.show_mods {
        set_show_loaded_mods(true);
    }
    everything.load_all();
    everything.validate_all();
    everything.check_rivers();
    if args.pod {
        everything.check_pod();
    }
    emit_reports();
    if args.unused {
        everything.check_unused();
    }

    Ok(())
}
