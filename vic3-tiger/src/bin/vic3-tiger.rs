use std::mem::forget;
use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use tiger_lib::{
    disable_ansi_colors, emit_reports, find_game_directory_steam, set_mod_root,
    set_show_loaded_mods, set_show_vanilla, set_vanilla_dir, Everything, Game,
};

/// Steam's code for Victoria 3
const VIC3_APP_ID: &str = "529340";

/// VIC3 directory under steam library dir
const VIC3_DIR: &str = "steamapps/common/Victoria 3";

/// A file that should be present if this is the VIC3 directory
const VIC3_SIGNATURE_FILE: &str = "game/events/titanic_events.txt";

#[derive(Parser)]
struct Cli {
    /// Path to folder of mod to check.
    modpath: PathBuf,
    /// Path to Vic3 directory.
    #[clap(long)]
    vic3: Option<PathBuf>,
    /// Show errors in the base Vic3 script code as well.
    #[clap(long)]
    show_vanilla: bool,
    /// Show errors in other loaded mods as well.
    #[clap(long)]
    show_mods: bool,
    /// Output the reports in JSON format
    #[clap(long)]
    json: bool,
    /// Warn about items that are defined but unused.
    #[clap(long)]
    unused: bool,
    /// Omit color from the output.
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

    // LAST UPDATED VERSION VIC3 1.3.6
    eprintln!("This validator was made for Victoria 3 version 1.3.6 (Thé à la menthe).");
    eprintln!("If you are using a newer version of Victoria 3, it may be inaccurate.");
    eprintln!("!! Currently it's inaccurate anyway because it's in beta state.");

    Game::set(Game::Vic3)?;

    if args.vic3.is_none() {
        args.vic3 = find_game_directory_steam(VIC3_APP_ID, &PathBuf::from(VIC3_DIR));
    }
    if let Some(ref mut vic3) = args.vic3 {
        eprintln!("Using Vic3 directory: {}", vic3.display());
        let mut sig = vic3.clone();
        sig.push(VIC3_SIGNATURE_FILE);
        if !sig.is_file() {
            eprintln!("That does not look like a Vic3 directory.");
            vic3.push("..");
            eprintln!("Trying: {}", vic3.display());
            sig = vic3.clone();
            sig.push(VIC3_SIGNATURE_FILE);
            if sig.is_file() {
                eprintln!("Ok.");
            } else {
                bail!("Cannot find Vic3 directory. Please supply it as the --vic3 option.");
            }
        }
    } else {
        bail!("Cannot find Vic3 directory. Please supply it as the --vic3 option.");
    }

    set_vanilla_dir(args.vic3.as_ref().unwrap().clone());

    if args.show_vanilla {
        eprintln!("Showing warnings for base game files too. There will be many false positives in those.");
    }

    if args.show_mods {
        eprintln!("Showing warnings for other loaded mods too.");
    }

    if args.unused {
        eprintln!("Showing warnings for unused localization. There will be many false positives.");
    }

    if args.no_color {
        // Disable colors both here and after reading the config, because reading the modfile and config may emit errors.
        disable_ansi_colors();
    }

    if args.modpath.is_dir() {
        let mut sig = args.modpath.clone();
        sig.push(".metadata/metadata.json");
        if !sig.is_file() {
            bail!("{} does not look like a mod directory.", args.modpath.display());
        }
    }
    eprintln!("Using mod directory: {}", args.modpath.display());

    set_mod_root(args.modpath.clone());

    let mut everything = Everything::new(args.vic3.as_deref(), &args.modpath, Vec::new())?;

    // Print a blank line between the preamble and the first report:
    eprintln!();

    everything.load_output_settings(true);
    everything.load_config_filtering_rules();
    if !args.json {
        emit_reports(false);
    }

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
    emit_reports(args.json);
    if args.unused {
        everything.check_unused();
    }

    // Properly dropping `everything` takes a noticeable amount of time, and we're exiting anyway.
    forget(everything);

    Ok(())
}
