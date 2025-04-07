use std::collections::{HashMap, HashSet};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    /// `loc_objects_documentation.md` file (input)
    #[arg(long)]
    doc: PathBuf,

    /// Directory with the Rust include files (output)
    #[arg(long)]
    out: PathBuf,
}

fn remove_game_wrapper(sometype: &str) -> &str {
    if let Some(sfx) = sometype.strip_prefix("Hoi4(") {
        if let Some(result) = sfx.strip_suffix(')') {
            return result;
        }
    }
    sometype
}

// Copied from src/datatypes.rs
const GENERIC_TYPES: &[&str] = &[
    "Unknown",
    "AnyScope",
    "CFixedPoint",
    "CString",
    "CUTF8String",
    "CVector2f",
    "CVector2i",
    "CVector3f",
    "CVector3i",
    "CVector4f",
    "CVector4i",
    "Date",
    "Scope",
    "TopScope",
    "bool",
    "double",
    "float",
    "int16",
    "int32",
    "int64",
    "int8",
    "uint16",
    "uint32",
    "uint64",
    "uint8",
    "void",
];

fn write_types(mut types: HashSet<String>, fname: PathBuf) -> Result<()> {
    let mut outf = File::create(fname)?;
    writeln!(outf, "#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Display, EnumString)]")?;
    writeln!(outf, "#[strum(use_phf)]")?;
    writeln!(outf, "pub enum Hoi4Datatype {{")?;
    let mut types: Vec<_> = types.drain().collect();
    types.sort();
    for t in types {
        if !GENERIC_TYPES.contains(&&*t) {
            writeln!(outf, "    {t},")?;
        }
    }
    writeln!(outf, "}}")?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NonGlobal {
    name: String,
    dtype: String,
    args: Vec<String>,
    rtype: String,
}

impl NonGlobal {
    fn new(name: String, dtype: String, args: Vec<String>, rtype: String) -> Self {
        Self { name, dtype, args, rtype }
    }

    fn print<F: Write>(&self, outf: &mut F) -> Result<()> {
        writeln!(
            outf,
            "    (\"{}\", {}, Args::Args(&[{}]), {}),",
            self.name,
            self.dtype,
            self.args.join(", "),
            self.rtype
        )
        .context("print")
    }
}

fn load_nonglobals(fname: &Path) -> Result<HashMap<String, NonGlobal>> {
    let mut nonglobals = HashMap::new();
    let nonglobal = read_to_string(fname)?;
    if let Some((_, middle)) = nonglobal.split_once("[\n") {
        if let Some((middle, _)) = middle.rsplit_once(']') {
            for line in middle.lines() {
                let line = line.strip_prefix("    (\"").context("parse error1")?;
                let (name, line) = line.split_once("\", ").context("parse error2")?;
                let (dtype, line) = line.split_once(", Args::Args(&[").context("parse error2b")?;
                let line = line.strip_suffix("),").context("parse error3")?;
                let (line, rtype) = line.rsplit_once("]), ").context("parse error4")?;
                let mut dtype = remove_game_wrapper(dtype);
                let mut rtype = remove_game_wrapper(rtype);
                let store;
                if !GENERIC_TYPES.contains(&rtype) {
                    store = format!("Hoi4({rtype})");
                    rtype = &store;
                }
                let args: Vec<_> = if line.is_empty() {
                    Vec::new()
                } else {
                    line.split(", ").map(ToOwned::to_owned).collect()
                };
                let idx = format!("{dtype}.{name}");
                let store2;
                if !GENERIC_TYPES.contains(&dtype) {
                    store2 = format!("Hoi4({dtype})");
                    dtype = &store2;
                }
                nonglobals.insert(
                    idx,
                    NonGlobal::new(name.to_owned(), dtype.to_owned(), args, rtype.to_owned()),
                );
            }
            return Ok(nonglobals);
        }
    }
    bail!("could not parse nonglobals from {}", fname.display());
}

fn merge_nonglobals(
    nonglobals: &mut HashMap<String, NonGlobal>,
    mut new_nonglobals: HashMap<String, NonGlobal>,
) {
    nonglobals.retain(|k, _| new_nonglobals.contains_key(k));

    for (k, v) in new_nonglobals.drain() {
        if let Some(old) = nonglobals.get(&k) {
            if old.args.len() == v.args.len() && (v.rtype == "Unknown" || old.rtype == v.rtype) {
                continue;
            }
        }
        nonglobals.insert(k, v);
    }
}

fn write_nonglobals(mut nonglobals: HashMap<String, NonGlobal>, fname: PathBuf) -> Result<()> {
    let mut outf = File::create(fname)?;
    writeln!(outf, "&[")?;
    let mut nonglobals: Vec<_> = nonglobals.drain().map(|(_, v)| v).collect();
    nonglobals.sort();
    for n in nonglobals {
        n.print(&mut outf)?;
    }
    writeln!(outf, "]")?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut promotes = load_nonglobals(&args.out.join("data_promotes.rs"))?;
    let mut functions = load_nonglobals(&args.out.join("data_functions.rs"))?;

    let mut new_types = HashSet::new();
    let mut new_promotes = HashMap::new();
    let mut new_functions = HashMap::new();

    let mut parent = "";
    let mut is_promotion = false;
    let content = read_to_string(args.doc)?;
    for line in content.lines() {
        if line.is_empty() || line.starts_with("* ") {
            continue;
        }

        if let Some(cat) = line.strip_prefix("## ") {
            if cat == "Table of Content" {
                continue;
            }
            parent = cat;
            is_promotion = false;
            new_types.insert(parent.to_owned());
        }

        if line.starts_with("### Promotions") {
            is_promotion = true;
            continue;
        } else if line.starts_with("### Properties") {
            is_promotion = false;
            continue;
        }

        if let Some(sfx) = line.strip_prefix("**") {
            if let Some(name) = sfx.strip_suffix("**") {
                let mut dtype = parent;
                let store;
                if !GENERIC_TYPES.contains(&dtype) {
                    store = format!("Hoi4({parent})");
                    dtype = &store;
                }
                let key = format!("{dtype}.{name}");
                if is_promotion {
                    new_promotes.insert(
                        key,
                        NonGlobal::new(
                            name.to_owned(),
                            dtype.to_owned(),
                            Vec::new(),
                            "Unknown".to_owned(),
                        ),
                    );
                } else {
                    new_functions.insert(
                        key,
                        NonGlobal::new(
                            name.to_owned(),
                            dtype.to_owned(),
                            Vec::new(),
                            "Unknown".to_owned(),
                        ),
                    );
                }
            }
        }
    }

    merge_nonglobals(&mut promotes, new_promotes);
    merge_nonglobals(&mut functions, new_functions);

    write_types(new_types, args.out.join("datatypes.rs"))?;
    write_nonglobals(promotes, args.out.join("data_promotes.rs"))?;
    write_nonglobals(functions, args.out.join("data_functions.rs"))?;

    Ok(())
}
