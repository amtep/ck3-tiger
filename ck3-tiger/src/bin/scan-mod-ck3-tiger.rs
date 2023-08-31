//! This is just a proof of concept for a utility that scans a mod for defined items.
//! It uses the [`Everything::iter_keys()`] method.

use std::fs::write;
use std::mem::forget;
use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use serde_json::{json, to_string_pretty, Value};
use strum::IntoEnumIterator;

use tiger_lib::{Everything, FileKind, Game, Item, ModFile};

#[derive(Parser)]
struct Cli {
    /// Path to .mod file of mod to scan.
    modpath: PathBuf,
    /// Where to write the resulting JSON file
    #[clap(short, long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let mut args = Cli::parse();

    // LAST UPDATED VERSION 1.10.1
    eprintln!("This scanner was made for Crusader Kings version 1.10.1 (Quill).");
    eprintln!("If you are using a newer version of Crusader Kings, it may be inaccurate.");
    eprintln!("The scanner does not find 100% of types of items in the mod.");

    Game::set(Game::Ck3)?;

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

    let mut everything = Everything::new(None, &modpath, modfile.replace_paths())?;

    everything.load_all();

    let mut grand_json = json!({});
    for itype in Item::iter() {
        let mut vec = Vec::new();
        for token in everything.iter_keys(itype).filter(|token| token.loc.kind == FileKind::Mod) {
            let json = json!({
                "key": token.to_string(),
                "file": token.loc.pathname(),
                "line": if token.loc.line == 0 { None } else { Some(token.loc.line) },
                "column": if token.loc.column == 0 { None } else { Some(token.loc.column) },
            });
            vec.push(json);
        }
        if !vec.is_empty() {
            grand_json[itype.to_string()] = Value::Array(vec);
        }
    }

    let output = to_string_pretty(&grand_json)?;
    eprintln!("Writing to {}", args.output.display());
    write(&args.output, output)?;

    // Properly dropping `everything` takes a noticeable amount of time, and we're exiting anyway.
    forget(everything);

    Ok(())
}
