use std::mem::forget;
use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use tiger_lib::{
    disable_ansi_colors, emit_reports, find_game_directory_steam, set_show_loaded_mods,
    set_show_vanilla, validate_config_file, Everything, Game, ModFile,
};

/// Steam's code for Imperator
const IMPERATOR_APP_ID: &str = "859580";

/// Imperator directory under steam library dir
const IMPERATOR_DIR: &str = "steamapps/common/ImperatorRome";

/// A file that should be present if this is the Imperator directory
const IMPERATOR_SIGNATURE_FILE: &str = "game/events/000_johan_debug.txt";

#[derive(Parser)]
struct Cli {
    /// Path to folder of mod to check.
    modpath: PathBuf,
    /// Path to Imperator directory.
    #[clap(long)]
    imperator: Option<PathBuf>,
    /// Path to custom .conf file.
    #[clap(long)]
    config: Option<PathBuf>,
    /// Show errors in the base Imperator script code as well.
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

    // LAST UPDATED VERSION Imperator 2.0.4
    eprintln!("This validator was made for Imperator Rome version 2.0.4.");
    eprintln!("If you are using an older version of Imperator Rome, it may be inaccurate.");
    eprintln!("!! Currently it's inaccurate anyway because it's in beta state.");

    Game::set(Game::Imperator)?;

    if args.imperator.is_none() {
        args.imperator = find_game_directory_steam(IMPERATOR_APP_ID, &PathBuf::from(IMPERATOR_DIR));
    }
    if let Some(ref mut imperator) = args.imperator {
        eprintln!("Using Imperator directory: {}", imperator.display());
        let mut sig = imperator.clone();
        sig.push(IMPERATOR_SIGNATURE_FILE);
        if !sig.is_file() {
            eprintln!("That does not look like a Imperator directory.");
            imperator.push("..");
            eprintln!("Trying: {}", imperator.display());
            sig = imperator.clone();
            sig.push(IMPERATOR_SIGNATURE_FILE);
            if sig.is_file() {
                eprintln!("Ok.");
            } else {
                bail!(
                    "Cannot find Imperator directory. Please supply it as the --imperator option."
                );
            }
        }
    } else {
        bail!("Cannot find Imperator directory. Please supply it as the --imperator option.");
    }

    args.config = validate_config_file(args.config);

    if args.show_vanilla {
        eprintln!("Showing warnings for base game files too. There will be many false positives in those.");
    }

    if args.show_mods {
        eprintln!("Showing warnings for other loaded mods too.");
    }

    if args.unused {
        eprintln!("Showing warnings for unused localization. There will be many false positives.");
    }

    if args.modpath.is_dir() {
        args.modpath.push("descriptor.mod");
    }
    if args.no_color {
        // Disable colors both here and after reading the config, because reading the modfile and config may emit errors.
        disable_ansi_colors();
    }

    let modfile = ModFile::read(&args.modpath)?;
    let modpath = modfile.modpath();
    if !modpath.exists() {
        eprintln!("Looking for mod in {}", modpath.display());
        bail!("Cannot find mod directory. Please make sure the .mod file is correct.");
    }
    eprintln!("Using mod directory: {}", modpath.display());

    let mut everything =
        Everything::new(args.config.as_deref(), args.imperator.as_deref(), &modpath, Vec::new())?;

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

    // Properly dropping 'everything' takes a noticable amount of time and we're exiting anyway.
    forget(everything);

    Ok(())
}
