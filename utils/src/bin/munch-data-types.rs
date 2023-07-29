use std::collections::{HashMap, HashSet};
use std::fs::{read_dir, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use strum_macros::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum, Display)]
enum Game {
    /// Crusader Kings 3
    Ck3,
    /// Victoria 3
    Vic3,
}

#[derive(Debug, Parser)]
struct Cli {
    /// Which game the logs are for
    #[arg(long)]
    game: Game,

    /// Game concepts to filter out (CK3 only)
    #[arg(long)]
    concepts: Option<PathBuf>,

    /// Directory with the datatype logs (input)
    #[arg(long)]
    logs: PathBuf,

    /// Directory with the Rust include files (output)
    #[arg(long)]
    out: PathBuf,
}

// fn load_types(fname: PathBuf) -> Result<HashSet<String>> {
//     let types = read_to_string(&fname)?;
//     if let Some((_, middle)) = types.split_once('{') {
//         if let Some((middle, _)) = middle.split_once('}') {
//             let mut set: HashSet<_> = middle.split(",").map(str::trim).map(str::to_owned).collect();
//             set.remove("");
//             set.remove("Unknown");
//             set.remove("AnyScope");
//             return Ok(set);
//         }
//     }
//     bail!("could not parse Datatype from {}", fname.display());
// }

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

fn write_types(mut types: HashSet<String>, fname: PathBuf, game: Game) -> Result<()> {
    let mut outf = File::create(fname)?;
    writeln!(outf, "#[derive(Copy, Clone, Debug, Eq, PartialEq, Display, EnumString)]")?;
    writeln!(outf, "pub enum {game}Datatype {{")?;
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
struct Global {
    name: String,
    args: Vec<String>,
    rtype: String,
}

impl Global {
    fn new(name: String, args: Vec<String>, rtype: String) -> Self {
        Self { name, args, rtype }
    }

    fn print<F: Write>(&self, outf: &mut F) -> Result<()> {
        writeln!(
            outf,
            "    (\"{}\", Args(&[{}]), {}),",
            self.name,
            self.args.join(", "),
            self.rtype
        )
        .context("print")
    }
}

fn load_globals(fname: PathBuf, game: Game) -> Result<HashMap<String, Global>> {
    let mut globals = HashMap::new();
    let global = read_to_string(&fname)?;
    if let Some((_, middle)) = global.split_once("[\n") {
        if let Some((middle, _)) = middle.rsplit_once(']') {
            for line in middle.lines() {
                let line = line.strip_prefix("    (\"").context("parse error1")?;
                let (name, line) = line.split_once("\", Args(&[").context("parse error2")?;
                let line = line.strip_suffix("),").context("parse error3")?;
                let (line, mut rtype) = line.rsplit_once("]), ").context("parse error4")?;
                let args: Vec<_> = if line.is_empty() {
                    Vec::new()
                } else {
                    line.split(", ").map(|s| s.to_owned()).collect()
                };
                let store;
                if !GENERIC_TYPES.contains(&rtype) {
                    store = format!("{game}({rtype})");
                    rtype = &store;
                }
                globals
                    .insert(name.to_owned(), Global::new(name.to_owned(), args, rtype.to_owned()));
            }
            return Ok(globals);
        }
    }
    bail!("could not parse globals from {}", fname.display());
}

fn merge_globals(globals: &mut HashMap<String, Global>, mut new_globals: HashMap<String, Global>) {
    globals.retain(|k, _| new_globals.contains_key(k));

    for (k, v) in new_globals.drain() {
        if let Some(old) = globals.get(&k) {
            if old.args.len() == v.args.len() && (v.rtype == "Unknown" || old.rtype == v.rtype) {
                continue;
            }
        }
        globals.insert(k, v);
    }
}

fn write_globals(mut globals: HashMap<String, Global>, fname: PathBuf) -> Result<()> {
    let mut outf = File::create(fname)?;
    writeln!(outf, "&[")?;
    let mut globals: Vec<_> = globals.drain().map(|(_, v)| v).collect();
    globals.sort();
    for g in globals {
        g.print(&mut outf)?;
    }
    writeln!(outf, "]")?;
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
            "    (\"{}\", {}, Args(&[{}]), {}),",
            self.name,
            self.dtype,
            self.args.join(", "),
            self.rtype
        )
        .context("print")
    }
}

fn load_nonglobals(fname: PathBuf, game: Game) -> Result<HashMap<String, NonGlobal>> {
    let mut nonglobals = HashMap::new();
    let nonglobal = read_to_string(&fname)?;
    if let Some((_, middle)) = nonglobal.split_once("[\n") {
        if let Some((middle, _)) = middle.rsplit_once(']') {
            for line in middle.lines() {
                let line = line.strip_prefix("    (\"").context("parse error1")?;
                let (name, line) = line.split_once("\", ").context("parse error2")?;
                let (mut dtype, line) = line.split_once(", Args(&[").context("parse error2b")?;
                let line = line.strip_suffix("),").context("parse error3")?;
                let (line, mut rtype) = line.rsplit_once("]), ").context("parse error4")?;
                let store;
                if !GENERIC_TYPES.contains(&rtype) {
                    store = format!("{game}({rtype})");
                    rtype = &store;
                }
                let args: Vec<_> = if line.is_empty() {
                    Vec::new()
                } else {
                    line.split(", ").map(|s| s.to_owned()).collect()
                };
                let idx = format!("{dtype}.{name}");
                let store2;
                if !GENERIC_TYPES.contains(&dtype) {
                    store2 = format!("{game}({dtype})");
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
            nonglobals.insert(k, v);
        }
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

fn parse_datafunction(
    item: &str,
    types: &mut HashSet<String>,
    global_promotes: &mut HashMap<String, Global>,
    global_functions: &mut HashMap<String, Global>,
    promotes: &mut HashMap<String, NonGlobal>,
    functions: &mut HashMap<String, NonGlobal>,
    game: Game,
) {
    if item.is_empty() {
        return;
    }
    let header = item.lines().next().unwrap();

    if item.contains("Definition type: Type") {
        types.insert(header.to_owned());
        return;
    }

    if item.contains("Definition type: Global macro") {
        return;
    }

    let mut name = header;
    let mut nargs = 0;
    if let Some((s1, s2)) = header.split_once('(') {
        name = s1;
        nargs = s2.split(',').count();
    }
    let mut args = Vec::new();
    for _ in 0..nargs {
        args.push("DType(Unknown)".to_string());
    }

    let mut rtype = "";
    if let Some((_, s2)) = item.split_once("Return type: ") {
        rtype = s2.trim();
    }
    if rtype == "[unregistered]" {
        rtype = "Unknown";
    } else if rtype == "_null_type_" {
        rtype = "void";
    }
    let store;
    if !GENERIC_TYPES.contains(&rtype) {
        store = format!("{game}({rtype})");
        rtype = &store;
    }

    if item.contains("Definition type: Global promote") {
        global_promotes
            .insert(name.to_owned(), Global::new(name.to_owned(), args, rtype.to_owned()));
        return;
    } else if item.contains("Definition type: Global function") {
        global_functions
            .insert(name.to_owned(), Global::new(name.to_owned(), args, rtype.to_owned()));
        return;
    }

    let (mut dtype, barename) = name.split_once('.').unwrap();
    let store2;
    if !GENERIC_TYPES.contains(&dtype) {
        store2 = format!("{game}({dtype})");
        dtype = &store2;
    }
    if barename == "Self" || barename == "AccessSelf" {
        rtype = dtype;
    }

    if item.contains("Definition type: Promote") {
        promotes.insert(
            name.to_owned(),
            NonGlobal::new(barename.to_owned(), dtype.to_owned(), args, rtype.to_owned()),
        );
        return;
    }
    if item.contains("Definition type: Function") {
        functions.insert(
            name.to_owned(),
            NonGlobal::new(barename.to_owned(), dtype.to_owned(), args, rtype.to_owned()),
        );
        return;
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    if args.game == Game::Ck3 && args.concepts.is_none() {
        bail!("When loading ck3 datatypes, must provide --concepts argument");
    }
    let concepts_file = args.concepts.map(read_to_string).unwrap_or(Ok(String::new()))?;
    let concepts: HashSet<_> = concepts_file.split_whitespace().collect();

    // let types = load_types(args.out.join("datatypes.rs"))?;
    let mut global_promotes = load_globals(args.out.join("data_global_promotes.rs"), args.game)?;
    let mut global_functions = load_globals(args.out.join("data_global_functions.rs"), args.game)?;
    let mut promotes = load_nonglobals(args.out.join("data_promotes.rs"), args.game)?;
    let mut functions = load_nonglobals(args.out.join("data_functions.rs"), args.game)?;

    let mut new_types = HashSet::new();
    let mut new_global_promotes = HashMap::new();
    let mut new_global_functions = HashMap::new();
    let mut new_promotes = HashMap::new();
    let mut new_functions = HashMap::new();

    for entry in read_dir(args.logs)? {
        let entry = entry?;
        if !entry.file_name().to_string_lossy().ends_with(".txt") {
            continue;
        }
        let content = read_to_string(entry.path())?;
        for item in content.split("\n-----------------------\n\n") {
            parse_datafunction(
                item,
                &mut new_types,
                &mut new_global_promotes,
                &mut new_global_functions,
                &mut new_promotes,
                &mut new_functions,
                args.game,
            );
        }
    }

    for c in concepts {
        new_global_functions.remove(c);
    }

    // Root seems to work as well as ROOT
    new_global_promotes
        .insert("Root".to_string(), Global::new("Root".to_string(), vec![], "Scope".to_string()));

    merge_globals(&mut global_promotes, new_global_promotes);
    merge_globals(&mut global_functions, new_global_functions);
    merge_nonglobals(&mut promotes, new_promotes);
    merge_nonglobals(&mut functions, new_functions);

    write_types(new_types, args.out.join("datatypes.rs"), args.game)?;
    write_globals(global_promotes, args.out.join("data_global_promotes.rs"))?;
    write_globals(global_functions, args.out.join("data_global_functions.rs"))?;
    write_nonglobals(promotes, args.out.join("data_promotes.rs"))?;
    write_nonglobals(functions, args.out.join("data_functions.rs"))?;

    Ok(())
}
