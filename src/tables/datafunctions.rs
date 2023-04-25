#![allow(non_camel_case_types)]

use strum_macros::{Display, EnumString};

use crate::token::Token;

// Validate the "code" blocks in localization files and in the gui files.
// The include/ files are converted from the game's data_type_* output files.

include!("include/datatypes.rs");

#[derive(Copy, Clone, Debug)]
pub enum Args {
    NoArgs,
    Arg(Datatype),
    Arg2(Datatype, Datatype),
    Arg3(Datatype, Datatype, Datatype),
    Arg4(Datatype, Datatype, Datatype, Datatype),
}

impl Args {
    pub fn nargs(&self) -> usize {
        match self {
            NoArgs => 0,
            Arg(_) => 1,
            Arg2(_, _) => 2,
            Arg3(_, _, _) => 3,
            Arg4(_, _, _, _) => 4,
        }
    }
}

pub enum LookupResult {
    NotFound,
    WrongType,
    Found(Args, Datatype),
}

pub fn lookup_global_promote(lookup_name: &Token) -> Option<(Args, Datatype)> {
    for (name, args, rtype) in GLOBAL_PROMOTES {
        if lookup_name.is(name) {
            return Some((*args, *rtype));
        }
    }
    None
}

pub fn lookup_global_function(lookup_name: &Token) -> Option<(Args, Datatype)> {
    for (name, args, rtype) in GLOBAL_FUNCTIONS {
        if lookup_name.is(name) {
            return Some((*args, *rtype));
        }
    }
    None
}

pub fn lookup_promote(lookup_name: &Token, ltype: Datatype) -> LookupResult {
    let mut found_any = false;
    let mut possible_args = None;
    let mut possible_rtype = None;
    for (intype, name, args, rtype) in PROMOTES {
        if lookup_name.is(name) {
            found_any = true;
            if ltype == Datatype::Unknown {
                if possible_rtype.is_none() {
                    possible_args = Some(*args);
                    possible_rtype = Some(*rtype);
                } else if possible_rtype != Some(*rtype) {
                    possible_rtype = Some(Datatype::Unknown);
                }
            } else if ltype == *intype {
                return LookupResult::Found(*args, *rtype);
            }
        }
    }
    if found_any {
        if ltype == Datatype::Unknown {
            LookupResult::Found(possible_args.unwrap(), possible_rtype.unwrap())
        } else {
            LookupResult::WrongType
        }
    } else {
        LookupResult::NotFound
    }
}

pub fn lookup_function(lookup_name: &Token, ltype: Datatype) -> LookupResult {
    let mut found_any = false;
    let mut possible_args = None;
    let mut possible_rtype = None;
    for (intype, name, args, rtype) in FUNCTIONS {
        if lookup_name.is(name) {
            found_any = true;
            if ltype == Datatype::Unknown {
                if possible_rtype.is_none() {
                    possible_args = Some(*args);
                    possible_rtype = Some(*rtype);
                } else if possible_rtype != Some(*rtype) {
                    possible_rtype = Some(Datatype::Unknown);
                }
            } else if ltype == *intype {
                return LookupResult::Found(*args, *rtype);
            }
        }
    }
    if found_any {
        if ltype == Datatype::Unknown {
            LookupResult::Found(possible_args.unwrap(), possible_rtype.unwrap())
        } else {
            LookupResult::WrongType
        }
    } else {
        LookupResult::NotFound
    }
}

use Args::*;
use Datatype::*;

const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(Datatype, &str, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(Datatype, &str, Args, Datatype)] = include!("include/data_functions.rs");
