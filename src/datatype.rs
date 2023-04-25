use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::everything::Everything;
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

fn validate_argument(arg: &CodeArg, _data: &Everything, expect_type: Datatype) {
    match arg {
        CodeArg::Chain(chain) => validate_datatypes(chain, _data, expect_type),
        CodeArg::Literal(token) => {
            if token.as_str().starts_with('(') {
                // TODO: parse datatype from string
            } else {
                if expect_type != Datatype::Unknown && expect_type != Datatype::CString {
                    error(
                        token,
                        ErrorKey::DataFunctions,
                        &format!("expected {}, got CString", expect_type),
                    );
                }
            }
        }
    }
}

pub fn validate_datatypes(chain: &CodeChain, data: &Everything, expect_type: Datatype) {
    let mut curtype = Datatype::Unknown;
    for (i, code) in chain.codes.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == chain.codes.len() - 1;
        let args;
        let rtype;
        if let Some((xargs, xrtype)) = lookup_global_promote(&code.name) {
            if !is_first {
                error(
                    &code.name,
                    ErrorKey::DataFunctions,
                    &format!("{} must be the first in a chain", code.name),
                );
                return;
            }
            if is_last {
                error(
                    &code.name,
                    ErrorKey::DataFunctions,
                    &format!("{} can not be last in a chain", code.name),
                );
                return;
            }
            args = xargs;
            rtype = xrtype;
        } else if let Some((xargs, xrtype)) = lookup_global_function(&code.name) {
            if !(is_first && is_last) {
                error(
                    &code.name,
                    ErrorKey::DataFunctions,
                    &format!("{} must be used on its own", code.name),
                );
                return;
            }
            args = xargs;
            rtype = xrtype;
        } else {
            match lookup_promote(&code.name, curtype) {
                LookupResult::Found(xargs, xrtype) => {
                    if is_first {
                        error(
                            &code.name,
                            ErrorKey::DataFunctions,
                            &format!("{} can not be the first in a chain", code.name),
                        );
                        return;
                    }
                    args = xargs;
                    rtype = xrtype;
                }
                LookupResult::WrongType => {
                    error(
                        &code.name,
                        ErrorKey::DataFunctions,
                        &format!("{} can not follow a {} promote", code.name, curtype),
                    );
                    return;
                }
                LookupResult::NotFound => {
                    match lookup_function(&code.name, curtype) {
                        LookupResult::Found(xargs, xrtype) => {
                            if is_first {
                                error(
                                    &code.name,
                                    ErrorKey::DataFunctions,
                                    &format!("{} can not be the first in a chain", code.name),
                                );
                                return;
                            }
                            if !is_last {
                                error(
                                    &code.name,
                                    ErrorKey::DataFunctions,
                                    &format!("{} must be last in the chain", code.name),
                                );
                                return;
                            }
                            args = xargs;
                            rtype = xrtype;
                        }
                        LookupResult::WrongType => {
                            error(
                                &code.name,
                                ErrorKey::DataFunctions,
                                &format!("{} can not follow a {} promote", code.name, curtype),
                            );
                            return;
                        }
                        LookupResult::NotFound => {
                            // It's some passed-in scope
                            args = Args::NoArgs;
                            rtype = Datatype::Unknown;
                        }
                    }
                }
            }
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
        } else {
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
        }

        curtype = rtype;

        if is_last
            && curtype != Datatype::Unknown
            && expect_type != Datatype::Unknown
            && curtype != expect_type
        {
            error(
                &code.name,
                ErrorKey::DataFunctions,
                &format!(
                    "{} returns {} but a {} is needed here",
                    code.name, curtype, expect_type
                ),
            );
            return;
        }
    }
}
