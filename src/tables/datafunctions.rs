#![allow(non_camel_case_types)]

// Validate the "code" blocks in localization files and in the gui files.
// The include/ files are converted from the game's data_type_* output files.

include!("include/datatypes.rs");

enum Args {
    NoArgs,
    Arg(DataType),
    Arg2(DataType, DataType),
    Arg3(DataType, DataType, DataType),
    Arg4(DataType, DataType, DataType, DataType),
}

use Args::*;
use DataType::*;

const GLOBAL_PROMOTES: &[(&str, Args, DataType)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, DataType)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(DataType, &str, Args, DataType)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(DataType, &str, Args, DataType)] = include!("include/data_functions.rs");
