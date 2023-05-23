use std::str::FromStr;

use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::item::Item;
use crate::tables::datafunctions::Args;
use crate::token::Token;

pub use crate::tables::datafunctions::{
    lookup_function, lookup_global_function, lookup_global_promote, lookup_promote, Datatype,
    LookupResult,
};

#[derive(Clone, Debug)]
pub struct CodeChain {
    // "codes" is my name for the things separated by dots in gui functions.
    // They should be a series of "promotes" followed by a final "function",
    // each of which can possibly take arguments.
    pub codes: Vec<Code>,
}

// Most "codes" are just a name followed by another dot or by the end of the code section.
// Some have arguments between parentheses, which can be single-quoted strings, or other code chains.
#[derive(Clone, Debug)]
pub struct Code {
    pub name: Token,
    pub arguments: Vec<CodeArg>,
}

// Possibly the literal arguments can themselves contain [ ] code blocks.
// I'll have to test that.
// A literal argument can be a string that starts with a (datatype) in front
// of it, such as '(int32)0'.
#[derive(Clone, Debug)]
pub enum CodeArg {
    Chain(CodeChain),
    Literal(Token),
}

impl CodeChain {
    pub fn as_gameconcept(&self) -> Option<&Token> {
        if self.codes.len() == 1 && self.codes[0].arguments.is_empty() {
            Some(&self.codes[0].name)
        } else {
            None
        }
    }
}

fn validate_argument(arg: &CodeArg, data: &Everything, expect_type: Datatype) {
    match arg {
        CodeArg::Chain(chain) => validate_datatypes(chain, data, expect_type, false),
        CodeArg::Literal(token) => {
            if token.as_str().starts_with('(') && token.as_str().contains(')') {
                // These unwraps are safe because of the checks in the if condition
                let dtype = token
                    .as_str()
                    .split(')')
                    .next()
                    .unwrap()
                    .strip_prefix('(')
                    .unwrap();
                if dtype == "hex" {
                    if expect_type != Datatype::Unknown && expect_type != Datatype::int32 {
                        let msg = format!("expected {expect_type}, got {dtype}");
                        error(token, ErrorKey::DataFunctions, &msg);
                    }
                } else if let Ok(dtype) = Datatype::from_str(dtype) {
                    if expect_type != Datatype::Unknown && expect_type != dtype {
                        let msg = format!("expected {expect_type}, got {dtype}");
                        error(token, ErrorKey::DataFunctions, &msg);
                    }
                } else {
                    let msg = format!("unrecognized datatype {dtype}");
                    error(token, ErrorKey::DataFunctions, &msg);
                }
            } else {
                if expect_type != Datatype::Unknown && expect_type != Datatype::CString {
                    error(
                        token,
                        ErrorKey::DataFunctions,
                        &format!("expected {expect_type}, got CString"),
                    );
                }
            }
        }
    }
}

// `expect_promote` is true if the chain is expected to end on a promote rather than on a function.
pub fn validate_datatypes(
    chain: &CodeChain,
    data: &Everything,
    expect_type: Datatype,
    expect_promote: bool,
) {
    let mut curtype = Datatype::Unknown;
    for (i, code) in chain.codes.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == chain.codes.len() - 1;
        let mut args = Args::NoArgs;
        let mut rtype = Datatype::Unknown;

        if code.name.is("") {
            // TODO: find out if the game engine is okay with this
            warn(&code.name, ErrorKey::DataFunctions, "empty fragment");
            return;
        }

        // The data_type logs include all game concepts as global functions.
        // We don't want them to match here, because those concepts often
        // overlap with passed-in scopes, which are not functions.
        let lookup_gf = if data.item_exists(Item::GameConcept, code.name.as_str()) {
            None
        } else {
            lookup_global_function(&code.name)
        };
        let lookup_gp = lookup_global_promote(&code.name);
        let lookup_f = lookup_function(&code.name, curtype);
        let lookup_p = lookup_promote(&code.name, curtype);

        let gf_found = lookup_gf.is_some();
        let gp_found = lookup_gp.is_some();
        let f_found = !matches!(lookup_f, LookupResult::NotFound);
        let p_found = !matches!(lookup_p, LookupResult::NotFound);

        let mut found = false;

        if is_first && is_last && !expect_promote {
            if let Some((xargs, xrtype)) = lookup_gf {
                found = true;
                args = xargs;
                rtype = xrtype;
            }
        } else if is_first && (!is_last || expect_promote) {
            if let Some((xargs, xrtype)) = lookup_gp {
                found = true;
                args = xargs;
                rtype = xrtype;
            }
        } else if !is_first && (!is_last || expect_promote) {
            match lookup_p {
                LookupResult::Found(xargs, xrtype) => {
                    found = true;
                    args = xargs;
                    rtype = xrtype;
                }
                LookupResult::WrongType => {
                    let msg = format!("{} can not follow a {curtype} promote", code.name);
                    error(&code.name, ErrorKey::DataFunctions, &msg);
                    return;
                }
                LookupResult::NotFound => (),
            }
        } else if !is_first && is_last && !expect_promote {
            match lookup_f {
                LookupResult::Found(xargs, xrtype) => {
                    found = true;
                    args = xargs;
                    rtype = xrtype;
                }
                LookupResult::WrongType => {
                    let msg = format!("{} can not follow a {curtype} promote", code.name);
                    error(&code.name, ErrorKey::DataFunctions, &msg);
                    return;
                }
                LookupResult::NotFound => (),
            }
        }

        if !found {
            // Properly reporting these errors is tricky because `code.name`
            // might be found in any or all of the functions and promotes tables.
            if is_first && (p_found || f_found) && !gp_found && !gf_found {
                let msg = format!("{} can not be the first in a chain", code.name);
                error(&code.name, ErrorKey::DataFunctions, &msg);
                return;
            }
            if is_last && (gp_found || p_found) && !gf_found && !f_found && !expect_promote {
                let msg = format!("{} can not be last in a chain", code.name);
                error(&code.name, ErrorKey::DataFunctions, &msg);
                return;
            }
            if expect_promote && (gf_found || f_found) {
                let msg = format!("{} can not be used in this field", code.name);
                error(&code.name, ErrorKey::DataFunctions, &msg);
                return;
            }
            if !is_first && (gp_found || gf_found) && !p_found && !f_found {
                let msg = format!("{} must be the first in a chain", code.name);
                error(&code.name, ErrorKey::DataFunctions, &msg);
                return;
            }
            if !is_last && (gf_found || f_found) && !gp_found && !p_found {
                let msg = format!("{} must be last in the chain", code.name);
                error(&code.name, ErrorKey::DataFunctions, &msg);
                return;
            }
            // A catch-all condition if none of the above match
            if gp_found || gf_found || p_found || f_found {
                let msg = format!("{} is improperly used here", code.name);
                error(&code.name, ErrorKey::DataFunctions, &msg);
                return;
            }

            // If `code.name` is not found at all in the tables, then
            // it can be some passed-in scope. Unfortunately we don't
            // have a complete list of those, so accept any lowercase id
            // and warn if it starts with uppercase. This is not a foolproof
            // check though.
            // TODO: it's in theory possible to build a complete list
            // of possible scope variable names
            if code.name.as_str().chars().next().unwrap().is_uppercase() {
                // TODO: If there is a Custom of the same name, suggest that
                let msg = format!("unknown datafunction {}", &code.name);
                warn(&code.name, ErrorKey::DataFunctions, &msg);
                return;
            }

            // If it's a passed-in scope, then set args and rtype appropriately.
            args = Args::NoArgs;
            // TODO: this could in theory be reduced to just the scope types.
            // That would be valuable for checks because it will find
            // the common mistake of using .Var directly after one.
            rtype = Datatype::Unknown;
        }

        if args.nargs() != code.arguments.len() {
            error(
                &code.name,
                ErrorKey::DataFunctions,
                &format!(
                    "{} takes {} arguments but was given {} here",
                    code.name,
                    args.nargs(),
                    code.arguments.len()
                ),
            );
            return;
        }

        match args {
            Args::NoArgs => (),
            Args::Arg(dt1) => validate_argument(&code.arguments[0], data, dt1),
            Args::Arg2(dt1, dt2) => {
                validate_argument(&code.arguments[0], data, dt1);
                validate_argument(&code.arguments[1], data, dt2);
            }
            Args::Arg3(dt1, dt2, dt3) => {
                validate_argument(&code.arguments[0], data, dt1);
                validate_argument(&code.arguments[1], data, dt2);
                validate_argument(&code.arguments[2], data, dt3);
            }
            Args::Arg4(dt1, dt2, dt3, dt4) => {
                validate_argument(&code.arguments[0], data, dt1);
                validate_argument(&code.arguments[1], data, dt2);
                validate_argument(&code.arguments[2], data, dt3);
                validate_argument(&code.arguments[3], data, dt4);
            }
        }

        curtype = rtype;

        if is_last
            && curtype != Datatype::Unknown
            && expect_type != Datatype::Unknown
            && curtype != expect_type
        {
            let msg = format!(
                "{} returns {curtype} but a {expect_type} is needed here",
                code.name
            );
            error(&code.name, ErrorKey::DataFunctions, &msg);
            return;
        }
    }
}
