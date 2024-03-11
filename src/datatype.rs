//! Validator for the `[ ... ]` code blocks in localization and gui files.
//! The main entry points are the [`validate_datatypes`] function and the [`Datatype`] enum.

use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use once_cell::sync::Lazy;
use phf::phf_map;
use strum_macros::{Display, EnumString};

#[cfg(feature = "ck3")]
use crate::ck3::data::religions::CUSTOM_RELIGION_LOCAS;
use crate::context::ScopeContext;
use crate::data::customloca::CustomLocalization;
use crate::everything::Everything;
use crate::game::Game;
use crate::helpers::BiFnvHashMap;
use crate::item::Item;
#[cfg(feature = "ck3")]
use crate::report::err;
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;

// Load the game-specific datatype definitions
#[cfg(feature = "ck3")]
include!("ck3/tables/include/datatypes.rs");
#[cfg(feature = "vic3")]
include!("vic3/tables/include/datatypes.rs");
#[cfg(feature = "imperator")]
include!("imperator/tables/include/datatypes.rs");

/// All the object types used in `[...]` code in localization and gui files.
///
/// The names exactly match the ones in the `data_types` logs from the games,
/// which is why some of them are lowercase.
/// Most of the variants are generated directly from those logs.
///
/// The enum is divided into the "generic" datatypes, which are valid for all games and which can
/// be referenced directly in code, and the per-game lists of datatypes which are in game-specific
/// wrappers. With a few exceptions, the per-game datatypes are only referenced in the per-game tables
/// of datafunctions and promotes.
///
/// The game-specific datatypes are wrapped because otherwise they would still have name
/// collisions. This is because the list of generic datatypes is only a small selection; there are
/// many more datatypes that are in effect generic but separating them out would be pointless work.
/// (Separating them out would be made harder because the lists of variants are generated from the docs).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub enum Datatype {
    // Synthetic datatypes for our typechecking
    Unknown,
    AnyScope,

    // The generic datatypes
    CFixedPoint,
    CString,
    CUTF8String,
    CVector2f,
    CVector2i,
    CVector3f,
    CVector3i,
    CVector4f,
    CVector4i,
    Date,
    Scope,
    TopScope,
    bool,
    double,
    float,
    int16,
    int32,
    int64,
    int8,
    uint16,
    uint32,
    uint64,
    uint8,
    void,

    // Wrappers for the per-game datatypes
    #[cfg(feature = "ck3")]
    Ck3(Ck3Datatype),
    #[cfg(feature = "vic3")]
    Vic3(Vic3Datatype),
    #[cfg(feature = "imperator")]
    Imperator(ImperatorDatatype),
}

static STR_DATATYPE_MAP: phf::Map<&'static str, Datatype> = phf_map! {
    "Unknown" => Datatype::Unknown,
    "AnyScope" => Datatype::AnyScope,
    "CFixedPoint" => Datatype::CFixedPoint,
    "CString" => Datatype::CString,
    "CUTF8String" => Datatype::CUTF8String,
    "CVector2f" => Datatype::CVector2f,
    "CVector2i" => Datatype::CVector2i,
    "CVector3f" => Datatype::CVector3f,
    "CVector3i" => Datatype::CVector3i,
    "CVector4f" => Datatype::CVector4f,
    "CVector4i" => Datatype::CVector4i,
    "Date" => Datatype::Date,
    "Scope" => Datatype::Scope,
    "TopScope" => Datatype::TopScope,
    "bool" => Datatype::bool,
    "double" => Datatype::double,
    "float" => Datatype::float,
    "int16" => Datatype::int16,
    "int32" => Datatype::int32,
    "int64" => Datatype::int64,
    "int8" => Datatype::int8,
    "uint16" => Datatype::uint16,
    "uint32" => Datatype::uint32,
    "uint64" => Datatype::uint64,
    "uint8" => Datatype::uint8,
    "void" => Datatype::void,
};

impl FromStr for Datatype {
    type Err = strum::ParseError;
    /// Read a Datatype from a string, without requiring the string to use the game-specific wrappers.
    fn from_str(s: &str) -> Result<Self, strum::ParseError> {
        STR_DATATYPE_MAP.get(s).copied().ok_or(strum::ParseError::VariantNotFound).or_else(|_| {
            match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => Ck3Datatype::from_str(s).map(Datatype::Ck3),
                #[cfg(feature = "vic3")]
                Game::Vic3 => Vic3Datatype::from_str(s).map(Datatype::Vic3),
                #[cfg(feature = "imperator")]
                Game::Imperator => ImperatorDatatype::from_str(s).map(Datatype::Imperator),
            }
        })
    }
}

impl Display for Datatype {
    /// Convert a `Datatype` to string format, while leaving out the game-specific wrappers.
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        // Have to do the generic variants by hand, so that the per-game variants can be done with the macro.
        match *self {
            Datatype::Unknown => write!(f, "Unknown"),
            Datatype::AnyScope => write!(f, "AnyScope"),
            Datatype::CFixedPoint => write!(f, "CFixedPoint"),
            Datatype::CString => write!(f, "CString"),
            Datatype::CUTF8String => write!(f, "CUTF8String"),
            Datatype::CVector2f => write!(f, "CVector2f"),
            Datatype::CVector2i => write!(f, "CVector2i"),
            Datatype::CVector3f => write!(f, "CVector3f"),
            Datatype::CVector3i => write!(f, "CVector3i"),
            Datatype::CVector4f => write!(f, "CVector4f"),
            Datatype::CVector4i => write!(f, "CVector4i"),
            Datatype::Date => write!(f, "Date"),
            Datatype::Scope => write!(f, "Scope"),
            Datatype::TopScope => write!(f, "TopScope"),
            Datatype::bool => write!(f, "bool"),
            Datatype::double => write!(f, "double"),
            Datatype::float => write!(f, "float"),
            Datatype::int16 => write!(f, "int16"),
            Datatype::int32 => write!(f, "int32"),
            Datatype::int64 => write!(f, "int64"),
            Datatype::int8 => write!(f, "int8"),
            Datatype::uint16 => write!(f, "uint16"),
            Datatype::uint32 => write!(f, "uint32"),
            Datatype::uint64 => write!(f, "uint64"),
            Datatype::uint8 => write!(f, "uint8"),
            Datatype::void => write!(f, "void"),
            #[cfg(feature = "ck3")]
            Datatype::Ck3(dt) => dt.fmt(f),
            #[cfg(feature = "vic3")]
            Datatype::Vic3(dt) => dt.fmt(f),
            #[cfg(feature = "imperator")]
            Datatype::Imperator(dt) => dt.fmt(f),
        }
    }
}

/// A [`CodeChain`] represents the full string between `[` and `]` in gui and localization (except for
/// the trailing format).
/// It consists of a series of codes separated by dots.
///
/// "code" is my name for the things separated by dots. They don't have an official name.
/// They should be a series of "promotes" followed by a final "function",
/// each of which can possibly take arguments. The first code should be "global", meaning it
/// doesn't need a [`Datatype`] from the previous code as input.
///
/// There are a few exceptions that don't take a "function" at the end and are just a list of "promotes".
///
/// A `CodeChain` can also be very simple and consist of a single identifier, which should be a
/// global function because it both starts and ends the chain.
#[derive(Clone, Debug)]
pub struct CodeChain {
    pub codes: Vec<Code>,
}

/// Most codes are just a name followed by another dot or by the end of the code chain.
/// Some have comma-separated arguments between parentheses.
/// Those arguments can be single-quoted strings or other code chains.
#[derive(Clone, Debug)]
pub struct Code {
    pub name: Token,
    pub arguments: Vec<CodeArg>,
}

/// `CodeArg` represents a single argument of a [`Code`].
// Possibly the literal arguments can themselves contain [ ] code blocks.
// I'll have to test that.
// A literal argument can be a string that starts with a (datatype) in front
// of it, such as '(int32)0'.
#[derive(Clone, Debug)]
pub enum CodeArg {
    /// An argument that is itself a [`CodeChain`], though it doesn't need the `[` `]` around it.
    Chain(CodeChain),
    /// An argument that is a literal string between single quotes. The literal can start with a
    /// datatype in front of it between parentheses, such as `'(int32)0'`. If it doesn't start
    /// with a datatype, the literal's type will be `CString`.
    Literal(Token),
}

impl CodeChain {
    #[cfg(feature = "ck3")]
    pub fn as_gameconcept(&self) -> Option<&Token> {
        if self.codes.len() == 1 && self.codes[0].arguments.is_empty() {
            Some(&self.codes[0].name)
        } else if self.codes.len() == 1
            && self.codes[0].name.is("Concept")
            && self.codes[0].arguments.len() == 2
        {
            if let CodeArg::Literal(token) = &self.codes[0].arguments[0] {
                Some(token)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// [`Arg`] is the counterpart to [`CodeArg`]. Where `CodeArg` represents an actual argument given
/// in a codechain string, the `Arg` represents what kind of argument is expected by a promote or
/// function.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Arg {
    /// The argument is expected to be a code chain whose final function returns this [`Datatype`],
    /// or a literal that is encoded to be of the expected type.
    DType(Datatype),
    /// The argument is expected to be a literal containing a key to this [`Item`] type, or a code
    /// chain that returns a `CString` (in which case the `Item` lookup is not checked).
    IType(Item),
}

/// [`Args`] is the list of arguments expected by a given promote or function. The actual arguments
/// from a [`Code`] can be checked against this. The special value `Args::Unknown` means that all
/// arguments are accepted.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Args {
    Unknown,
    Args(&'static [Arg]),
}

/// Result from looking up a name in the promotes or functions tables.
#[derive(Copy, Clone, Debug)]
enum LookupResult {
    /// The name didn't occur in the table at all.
    NotFound,
    /// The name was in the table, but not associated with the given [`Datatype`].
    WrongType,
    /// Found a matching entry.
    /// Returns the expected arguments for this promote or function, and its return type.
    Found(Args, Datatype),
}

/// Internal function for validating a reference to a custom localization.
///
/// * `token`: The name of the localization.
/// * `scopes`: The scope type of the value being passed in to the custom localization.
/// * `lang`: The language being validated, can be "" when not applicable (such as in gui files).
///   Many custom localizations are only meant for one language, and the keys they use only need
///   to exist in that language.
fn validate_custom(token: &Token, data: &Everything, scopes: Scopes, lang: &'static str) {
    data.verify_exists(Item::CustomLocalization, token);
    if let Some((key, block)) = data.get_key_block(Item::CustomLocalization, token.as_str()) {
        CustomLocalization::validate_custom_call(key, block, data, token, scopes, lang, "", None);
    }
}

/// Internal function for validating an argument to a datatype code.
/// If the argument is iself a code chain, this will end up calling `validate_datatypes` recursively.
///
/// * `arg`: The actual argument being supplied.
/// * `sc`: The available named scopes.
/// * `expect_arg`: The form of argument expected by the promote or function.
/// * `lang`: The language of the localization file in which this code appears. This is just passed through.
/// * `format`: The formatting code for this code chain. This just passed through.
fn validate_argument(
    arg: &CodeArg,
    data: &Everything,
    sc: &mut ScopeContext,
    expect_arg: Arg,
    lang: &'static str,
    format: Option<&Token>,
) {
    match expect_arg {
        Arg::DType(expect_type) => {
            match arg {
                CodeArg::Chain(chain) => {
                    validate_datatypes(chain, data, sc, expect_type, lang, format, false);
                }
                CodeArg::Literal(token) => {
                    if token.as_str().starts_with('(') && token.as_str().contains(')') {
                        // These unwraps are safe because of the checks in the if condition
                        let dtype =
                            token.as_str().split(')').next().unwrap().strip_prefix('(').unwrap();
                        if dtype == "hex" {
                            if expect_type != Datatype::Unknown && expect_type != Datatype::int32 {
                                let msg = format!("expected {expect_type}, got {dtype}");
                                warn(ErrorKey::Datafunctions).msg(msg).loc(token).push();
                            }
                        } else if let Ok(dtype) = Datatype::from_str(dtype) {
                            if expect_type != Datatype::Unknown && expect_type != dtype {
                                let msg = format!("expected {expect_type}, got {dtype}");
                                warn(ErrorKey::Datafunctions).msg(msg).loc(token).push();
                            }
                        } else {
                            let msg = format!("unrecognized datatype {dtype}");
                            warn(ErrorKey::Datafunctions).msg(msg).loc(token).push();
                        }
                    } else if expect_type != Datatype::Unknown && expect_type != Datatype::CString {
                        let msg = format!("expected {expect_type}, got CString");
                        warn(ErrorKey::Datafunctions).msg(msg).loc(token).push();
                    }
                }
            }
        }
        Arg::IType(itype) => match arg {
            CodeArg::Chain(chain) => {
                validate_datatypes(chain, data, sc, Datatype::CString, lang, format, false);
            }
            CodeArg::Literal(token) => {
                data.verify_exists(itype, token);
            }
        },
    }
}

/// Validate a datafunction chain, which is the stuff between [ ] in localization.
/// * `chain` is the parsed datafunction structure.
/// * `sc` is a `ScopeContext` used to evaluate scope references in the datafunctions.
///   If nothing is known about the scope, just pass an empty `ScopeContext` with `set_strict_types(false)`.
/// * `expect_type` is the datatype that should be returned by this chain, can be `Datatype::Unknown` in many cases.
/// * `lang` is set to a specific language if `Custom` references in this chain only need to be defined for one language.
///   It can just be "" otherwise.
/// * `format` is the formatting code given after `|` in the datatype expression. It's used for
///   checking that game concepts in ck3 have `|E` formats.
/// * `expect_promote` is true iff the chain is expected to end on a promote rather than on a function.
///   Promotes and functions are very similar but they are defined separately in the datafunction tables
///   and usually only a function can end a chain.
pub fn validate_datatypes(
    chain: &CodeChain,
    data: &Everything,
    sc: &mut ScopeContext,
    expect_type: Datatype,
    lang: &'static str,
    format: Option<&Token>,
    expect_promote: bool,
) {
    let mut curtype = Datatype::Unknown;
    #[allow(unused_mut)] // vic3 does not need the mut
    let mut codes = Cow::from(&chain.codes[..]);
    #[cfg(feature = "ck3")]
    let mut macro_count = 0;
    // Have to loop with `while` instead of `for` because the array can mutate during the loop because of macro substitution
    let mut i = 0;
    while i < codes.len() {
        #[cfg(feature = "ck3")]
        if Game::is_ck3() {
            while let Some(binding) = data.data_bindings.get(codes[i].name.as_str()) {
                if let Some(replacement) = binding.replace(&codes[i]) {
                    macro_count += 1;
                    if macro_count > 255 {
                        let msg =
                            format!("substituted data bindings {macro_count} times, giving up");
                        err(ErrorKey::Macro).msg(msg).loc(&codes[i].name).push();
                        return;
                    }
                    codes.to_mut().splice(i..=i, replacement.codes);
                } else {
                    return;
                }
            }
        }

        let code = &codes[i];
        let is_first = i == 0;
        let is_last = i == codes.len() - 1;
        let mut args = Args::Args(&[]);
        let mut rtype = Datatype::Unknown;

        if code.name.is("") {
            // TODO: verify if the game engine is okay with this
            warn(ErrorKey::Datafunctions).msg("empty fragment").loc(&code.name).push();
            return;
        }

        let lookup_gf = lookup_global_function(code.name.as_str());
        let lookup_gp = lookup_global_promote(code.name.as_str());
        let lookup_f = lookup_function(code.name.as_str(), curtype);
        let lookup_p = lookup_promote(code.name.as_str(), curtype);

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
                    let msg = format!("{} cannot follow a {curtype} promote", code.name);
                    warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
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
                    let msg = format!("{} cannot follow a {curtype} promote", code.name);
                    warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                    return;
                }
                LookupResult::NotFound => (),
            }
        }

        if !found {
            // Properly reporting these errors is tricky because `code.name`
            // might be found in any or all of the functions and promotes tables.
            if is_first && (p_found || f_found) && !gp_found && !gf_found {
                let msg = format!("{} cannot be the first in a chain", code.name);
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
            if is_last && (gp_found || p_found) && !gf_found && !f_found && !expect_promote {
                let msg = format!("{} cannot be last in a chain", code.name);
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
            if expect_promote && (gf_found || f_found) {
                let msg = format!("{} cannot be used in this field", code.name);
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
            if !is_first && (gp_found || gf_found) && !p_found && !f_found {
                let msg = format!("{} must be the first in a chain", code.name);
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
            if !is_last && (gf_found || f_found) && !gp_found && !p_found {
                let msg = format!("{} must be last in the chain", code.name);
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
            // A catch-all condition if none of the above match
            if gp_found || gf_found || p_found || f_found {
                let msg = format!("{} is improperly used here", code.name);
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
        }

        #[cfg(feature = "vic3")]
        // Vic3 allows the three-letter country codes to be used unadorned as datatypes.
        if Game::is_vic3()
            && !found
            && is_first
            && data.item_exists(Item::Country, code.name.as_str())
        {
            found = true;
            args = Args::Args(&[]);
            rtype = Datatype::Vic3(Vic3Datatype::Country);
        }

        #[cfg(feature = "imperator")]
        if Game::is_imperator()
            && !found
            && is_first
            && data.item_exists(Item::Country, code.name.as_str())
        {
            found = true;
            args = Args::Args(&[]);
            rtype = Datatype::Imperator(ImperatorDatatype::Country);
        }

        // In vic3, game concepts are unadorned, like [concept_ideology]
        // Each concept also generates a [concept_ideology_desc]
        #[cfg(feature = "vic3")]
        if Game::is_vic3()
            && !found
            && is_first
            && is_last
            && code.name.as_str().starts_with("concept_")
        {
            found = true;
            if let Some(concept) = code.name.as_str().strip_suffix("_desc") {
                data.verify_exists_implied(Item::GameConcept, concept, &code.name);
            } else {
                data.verify_exists(Item::GameConcept, &code.name);
            }
            args = Args::Args(&[]);
            rtype = Datatype::CString;
        }

        #[cfg(feature = "ck3")]
        if Game::is_ck3()
            && !found
            && is_first
            && is_last
            && data.item_exists(Item::GameConcept, code.name.as_str())
        {
            let game_concept_formatting = format
                .map_or(false, |fmt| fmt.as_str().contains('E') || fmt.as_str().contains('e'));
            // In ck3, allow unadorned game concepts as long as they end with _i
            // (which means they are just an icon). This is a heuristic.
            // TODO: should also allow unadorned game concepts if inside another format
            // Many strings leave out the |E from flavor text and the like.
            // if !code.name.as_str().ends_with("_i") && !game_concept_formatting {
            //     let msg = "game concept should have |E formatting";
            //     warn(ErrorKey::Localization).weak().msg(msg).loc(&code.name).push();
            // }

            // If the game concept is also a passed-in scope, the game concept takes precedence.
            // This is worth warning about.
            // Real life example: [ROOT.Char.Custom2('RelationToMeShort', schemer)]
            if sc.is_name_defined(code.name.as_str()).is_some() && !game_concept_formatting {
                let msg = format!("`{}` is both a named scope and a game concept here", &code.name);
                let info = format!("The game concept will take precedence. Do `{}.Self` if you want the named scope.", &code.name);
                warn(ErrorKey::Datafunctions).msg(msg).info(info).loc(&code.name).push();
            }

            found = true;
            args = Args::Args(&[]);
            rtype = Datatype::CString;
        }

        // See if it's a passed-in scope.
        // It may still be a passed-in scope even if this check doesn't pass, because sc might be a non-strict scope
        // where the scope names are not known. That's handled heuristically below.
        if !found && is_first {
            if let Some(scopes) = sc.is_name_defined(code.name.as_str()) {
                found = true;
                args = Args::Args(&[]);
                rtype = datatype_from_scopes(scopes);
            }
        }

        // If `code.name` is not found yet, then it can be some passed-in scope we don't know about.
        // Unfortunately we don't have a complete list of those, so accept any id that starts
        // with a lowercase letter or a number. This is not a foolproof check though.
        // TODO: it's in theory possible to build a complete list of possible scope variable names
        let first_char = code.name.as_str().chars().next().unwrap();
        if !found
            && is_first
            && !sc.is_strict()
            && (first_char.is_lowercase() || first_char.is_ascii_digit())
        {
            found = true;
            args = Args::Args(&[]);
            // TODO: this could in theory be reduced to just the scope types.
            // That would be valuable for checks because it will find
            // the common mistake of using .Var directly after one.
            rtype = Datatype::Unknown;
        }

        // If it's still not found, warn and exit.
        if !found {
            // TODO: If there is a Custom of the same name, suggest that
            let msg = format!("unknown datafunction {}", &code.name);
            if let Some(alternative) = lookup_alternative(code.name.as_str()) {
                let info = format!("did you mean {alternative}?");
                warn(ErrorKey::Datafunctions).msg(msg).info(info).loc(&code.name).push();
            } else {
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
            }
            return;
        }

        // This `if let` skips this check if args is `Args::Unknown`
        if let Args::Args(a) = args {
            if a.len() != code.arguments.len() {
                let msg = format!(
                    "{} takes {} arguments but was given {} here",
                    code.name,
                    a.len(),
                    code.arguments.len()
                );
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
        }

        // TODO: validate the Faith customs
        #[cfg(feature = "ck3")]
        if Game::is_ck3()
            && curtype != Datatype::Ck3(Ck3Datatype::Faith)
            && (code.name.is("Custom") && code.arguments.len() == 1)
            || (code.name.is("Custom2") && code.arguments.len() == 2)
        {
            // TODO: for Custom2, get the datatype of the second argument and use it to initialize scope:second
            if let CodeArg::Literal(ref token) = code.arguments[0] {
                if let Some(scopes) = scope_from_datatype(curtype) {
                    validate_custom(token, data, scopes, lang);
                } else if (curtype == Datatype::Unknown
                    || curtype == Datatype::AnyScope
                    || curtype == Datatype::TopScope)
                    && !CUSTOM_RELIGION_LOCAS.contains(&token.as_str())
                {
                    // TODO: is a TopScope even valid to pass to .Custom? verify
                    validate_custom(token, data, Scopes::all(), lang);
                }
            }
        }

        #[cfg(feature = "vic3")]
        if Game::is_vic3() && code.name.is("GetCustom") && code.arguments.len() == 1 {
            if let CodeArg::Literal(ref token) = code.arguments[0] {
                if let Some(scopes) = scope_from_datatype(curtype) {
                    validate_custom(token, data, scopes, lang);
                } else if curtype == Datatype::Unknown
                    || curtype == Datatype::AnyScope
                    || curtype == Datatype::TopScope
                {
                    // TODO: is a TopScope even valid to pass to .GetCustom? verify
                    validate_custom(token, data, Scopes::all(), lang);
                }
            }
        }

        #[cfg(feature = "imperator")]
        if Game::is_imperator() && code.name.is("Custom") && code.arguments.len() == 1 {
            if let CodeArg::Literal(ref token) = code.arguments[0] {
                if let Some(scopes) = scope_from_datatype(curtype) {
                    validate_custom(token, data, scopes, lang);
                } else if curtype == Datatype::Unknown
                    || curtype == Datatype::AnyScope
                    || curtype == Datatype::TopScope
                {
                    // TODO: is a TopScope even valid to pass to .Custom? verify
                    validate_custom(token, data, Scopes::all(), lang);
                }
            }
        }

        // TODO: vic3 docs say that `Localize` can take a `CustomLocalization` as well
        if code.name.is("Localize") && code.arguments.len() == 1 {
            if let CodeArg::Literal(ref token) = code.arguments[0] {
                // The is_ascii check is to weed out some localizations (looking at you, Russian)
                // that do a lot of Localize on already localized strings. There's no reason for
                // it, but I guess it makes them happy.
                if token.as_str().is_ascii() {
                    data.localization.verify_exists_lang(token, lang);
                }
            }
        }

        if let Args::Args(a) = args {
            for (i, arg) in a.iter().enumerate() {
                // Handle |E that contain a SelectLocalization that chooses between two gameconcepts
                if code.name.is("SelectLocalization") && i > 0 {
                    if let CodeArg::Chain(chain) = &code.arguments[i] {
                        if chain.codes.len() == 1
                            && chain.codes[0].arguments.is_empty()
                            && data.item_exists(Item::GameConcept, chain.codes[0].name.as_str())
                        {
                            continue;
                        }
                    }
                }
                validate_argument(&code.arguments[i], data, sc, *arg, lang, format);
            }
        }

        curtype = rtype;

        if is_last
            && curtype != Datatype::Unknown
            && expect_type != Datatype::Unknown
            && curtype != expect_type
        {
            if expect_type == Datatype::AnyScope {
                if scope_from_datatype(curtype).is_none() {
                    let msg =
                        format!("{} returns {curtype} but a scope type is needed here", code.name);
                    warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                    return;
                }
            } else {
                let msg =
                    format!("{} returns {curtype} but a {expect_type} is needed here", code.name);
                warn(ErrorKey::Datafunctions).msg(msg).loc(&code.name).push();
                return;
            }
        }

        i += 1;
    }
}

fn lookup_global_promote(lookup_name: &str) -> Option<(Args, Datatype)> {
    let global_promotes_map = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => &crate::ck3::tables::datafunctions::GLOBAL_PROMOTES_MAP,
        #[cfg(feature = "vic3")]
        Game::Vic3 => &crate::vic3::tables::datafunctions::GLOBAL_PROMOTES_MAP,
        #[cfg(feature = "imperator")]
        Game::Imperator => &crate::imperator::tables::datafunctions::GLOBAL_PROMOTES_MAP,
    };

    if let result @ Some(_) = global_promotes_map.get(lookup_name).copied() {
        return result;
    }

    // Datatypes can be used directly as global promotes, taking their value from the gui context.
    if let Ok(dtype) = Datatype::from_str(lookup_name) {
        return Some((Args::Args(&[]), dtype));
    }

    None
}

fn lookup_global_function(lookup_name: &str) -> Option<(Args, Datatype)> {
    let global_functions_map = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => &crate::ck3::tables::datafunctions::GLOBAL_FUNCTIONS_MAP,
        #[cfg(feature = "vic3")]
        Game::Vic3 => &crate::vic3::tables::datafunctions::GLOBAL_FUNCTIONS_MAP,
        #[cfg(feature = "imperator")]
        Game::Imperator => &crate::imperator::tables::datafunctions::GLOBAL_FUNCTIONS_MAP,
    };
    global_functions_map.get(lookup_name).copied()
}

fn lookup_promote_or_function(ltype: Datatype, vec: &[(Datatype, Args, Datatype)]) -> LookupResult {
    let mut possible_args = None;
    let mut possible_rtype = None;

    for (intype, args, rtype) in vec.iter().copied() {
        if ltype == Datatype::Unknown {
            if possible_rtype.is_none() {
                possible_args = Some(args);
                possible_rtype = Some(rtype);
            } else {
                if possible_rtype != Some(rtype) {
                    possible_rtype = Some(Datatype::Unknown);
                }
                if possible_args != Some(args) {
                    possible_args = Some(Args::Unknown);
                }
            }
        } else if ltype == intype {
            return LookupResult::Found(args, rtype);
        }
    }

    if ltype == Datatype::Unknown {
        LookupResult::Found(possible_args.unwrap(), possible_rtype.unwrap())
    } else {
        // If it was the right type, it would already have been returned as `Found`, above.
        LookupResult::WrongType
    }
}

fn lookup_promote(lookup_name: &str, ltype: Datatype) -> LookupResult {
    let promotes_map = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => &crate::ck3::tables::datafunctions::PROMOTES_MAP,
        #[cfg(feature = "vic3")]
        Game::Vic3 => &crate::vic3::tables::datafunctions::PROMOTES_MAP,
        #[cfg(feature = "imperator")]
        Game::Imperator => &crate::imperator::tables::datafunctions::PROMOTES_MAP,
    };

    promotes_map
        .get(lookup_name)
        .map_or(LookupResult::NotFound, |x| lookup_promote_or_function(ltype, x))
}

fn lookup_function(lookup_name: &str, ltype: Datatype) -> LookupResult {
    let functions_map = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => &crate::ck3::tables::datafunctions::FUNCTIONS_MAP,
        #[cfg(feature = "vic3")]
        Game::Vic3 => &crate::vic3::tables::datafunctions::FUNCTIONS_MAP,
        #[cfg(feature = "imperator")]
        Game::Imperator => &crate::imperator::tables::datafunctions::FUNCTIONS_MAP,
    };

    functions_map
        .get(lookup_name)
        .map_or(LookupResult::NotFound, |x| lookup_promote_or_function(ltype, x))
}

pub struct CaseInsensitiveStr(pub(crate) &'static str);

impl PartialEq for CaseInsensitiveStr {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for CaseInsensitiveStr {}

impl std::hash::Hash for CaseInsensitiveStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

/// Find an alternative datafunction to suggest when `lookup_name` has not been found.
/// This is a fairly expensive lookup.
/// Currently it only looks for different-case variants.
// TODO: make it consider misspellings as well
fn lookup_alternative(lookup_name: &'static str) -> Option<&'static str> {
    let lowercase_datatype_set = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => &crate::ck3::tables::datafunctions::LOWERCASE_DATATYPE_SET,
        #[cfg(feature = "vic3")]
        Game::Vic3 => &crate::vic3::tables::datafunctions::LOWERCASE_DATATYPE_SET,
        #[cfg(feature = "imperator")]
        Game::Imperator => &crate::imperator::tables::datafunctions::LOWERCASE_DATATYPE_SET,
    };

    lowercase_datatype_set.get(&CaseInsensitiveStr(lookup_name)).map(|x| x.0)
}

fn datatype_and_scope_map() -> &'static Lazy<BiFnvHashMap<Datatype, Scopes>> {
    match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => &crate::ck3::tables::datafunctions::DATATYPE_AND_SCOPE_MAP,
        #[cfg(feature = "vic3")]
        Game::Vic3 => &crate::vic3::tables::datafunctions::DATATYPE_AND_SCOPE_MAP,
        #[cfg(feature = "imperator")]
        Game::Imperator => &crate::imperator::tables::datafunctions::DATATYPE_AND_SCOPE_MAP,
    }
}

/// Return the scope type that best matches `dtype`, or `None` if there is no match.
/// Nearly every scope type has a matching datatype, but there are far more datatypes than scope types.
fn scope_from_datatype(dtype: Datatype) -> Option<Scopes> {
    datatype_and_scope_map().get_by_left(&dtype).copied()
}

/// Return the datatype that best matches `scopes`, or `Datatype::Unknown` if there is no match.
/// Nearly every scope type has a matching datatype, but there are far more datatypes than scope types.
/// Note that only `Scopes` values that are narrowed down to a single scope type can be matched.
fn datatype_from_scopes(scopes: Scopes) -> Datatype {
    datatype_and_scope_map().get_by_right(&scopes).copied().unwrap_or(Datatype::Unknown)
}
