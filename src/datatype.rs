use std::borrow::Cow;
use std::str::FromStr;

#[cfg(feature = "ck3")]
use crate::ck3::data::religions::CUSTOM_RELIGION_LOCAS;
#[cfg(feature = "ck3")]
pub use crate::ck3::tables::datafunctions::{
    lookup_alternative, lookup_function, lookup_global_function, lookup_global_promote,
    lookup_promote, scope_from_datatype, Arg, Args, Datatype, LookupResult,
};
use crate::data::customloca::CustomLocalization;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{error, old_warn, warn_info, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
#[cfg(feature = "vic3")]
pub use crate::vic3::tables::datafunctions::{
    lookup_alternative, lookup_function, lookup_global_function, lookup_global_promote,
    lookup_promote, scope_from_datatype, Arg, Args, Datatype, LookupResult,
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

fn validate_custom(token: &Token, data: &Everything, scopes: Scopes, lang: &'static str) {
    data.verify_exists(Item::CustomLocalization, token);
    if let Some((key, block)) = data
        .database
        .get_key_block(Item::CustomLocalization, token.as_str())
    {
        CustomLocalization::validate_custom_call(key, block, data, token, scopes, lang, "", None);
    }
}

fn validate_argument(arg: &CodeArg, data: &Everything, expect_arg: Arg, lang: &'static str) {
    match expect_arg {
        Arg::DType(expect_type) => {
            match arg {
                CodeArg::Chain(chain) => validate_datatypes(chain, data, expect_type, lang, false),
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
                                error(token, ErrorKey::Datafunctions, &msg);
                            }
                        } else if let Ok(dtype) = Datatype::from_str(dtype) {
                            if expect_type != Datatype::Unknown && expect_type != dtype {
                                let msg = format!("expected {expect_type}, got {dtype}");
                                error(token, ErrorKey::Datafunctions, &msg);
                            }
                        } else {
                            let msg = format!("unrecognized datatype {dtype}");
                            error(token, ErrorKey::Datafunctions, &msg);
                        }
                    } else if expect_type != Datatype::Unknown && expect_type != Datatype::CString {
                        let msg = format!("expected {expect_type}, got CString");
                        error(token, ErrorKey::Datafunctions, &msg);
                    }
                }
            }
        }
        Arg::IType(itype) => match arg {
            CodeArg::Chain(chain) => {
                validate_datatypes(chain, data, Datatype::CString, lang, false);
            }
            CodeArg::Literal(token) => {
                data.verify_exists(itype, token);
            }
        },
    }
}

// `expect_promote` is true if the chain is expected to end on a promote rather than on a function.
pub fn validate_datatypes(
    chain: &CodeChain,
    data: &Everything,
    expect_type: Datatype,
    lang: &'static str,
    expect_promote: bool,
) {
    let mut curtype = Datatype::Unknown;
    let mut codes = Cow::from(&chain.codes[..]);
    #[cfg(feature = "ck3")]
    let mut macro_count = 0;
    // Have to loop with `while` instead of `for` because the array can mutate during the loop because of macro substitution
    let mut i = 0;
    while i < codes.len() {
        #[cfg(feature = "ck3")]
        while let Some(binding) = data.data_bindings.get(codes[i].name.as_str()) {
            if let Some(replacement) = binding.replace(&codes[i]) {
                macro_count += 1;
                if macro_count > 255 {
                    let msg = format!("substituted data bindings {macro_count} times, giving up");
                    error(&codes[i].name, ErrorKey::Macro, &msg);
                    return;
                }
                codes.to_mut().splice(i..=i, replacement.codes);
            } else {
                return;
            }
        }

        let code = &codes[i];
        let is_first = i == 0;
        let is_last = i == codes.len() - 1;
        let mut args = Args::NoArgs;
        let mut rtype = Datatype::Unknown;

        if code.name.is("") {
            // TODO: find out if the game engine is okay with this
            old_warn(&code.name, ErrorKey::Datafunctions, "empty fragment");
            return;
        }

        // The data_type logs include all game concepts as global functions.
        // We don't want them to match here, because those concepts often
        // overlap with passed-in scopes, which are not functions.
        let lookup_gf = if data.item_exists(Item::GameConcept, code.name.as_str()) {
            None
        } else {
            lookup_global_function(code.name.as_str())
        };
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
                    let msg = format!("{} can not follow a {curtype} promote", code.name);
                    error(&code.name, ErrorKey::Datafunctions, &msg);
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
                    error(&code.name, ErrorKey::Datafunctions, &msg);
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
                error(&code.name, ErrorKey::Datafunctions, &msg);
                return;
            }
            if is_last && (gp_found || p_found) && !gf_found && !f_found && !expect_promote {
                let msg = format!("{} can not be last in a chain", code.name);
                error(&code.name, ErrorKey::Datafunctions, &msg);
                return;
            }
            if expect_promote && (gf_found || f_found) {
                let msg = format!("{} can not be used in this field", code.name);
                error(&code.name, ErrorKey::Datafunctions, &msg);
                return;
            }
            if !is_first && (gp_found || gf_found) && !p_found && !f_found {
                let msg = format!("{} must be the first in a chain", code.name);
                error(&code.name, ErrorKey::Datafunctions, &msg);
                return;
            }
            if !is_last && (gf_found || f_found) && !gp_found && !p_found {
                let msg = format!("{} must be last in the chain", code.name);
                error(&code.name, ErrorKey::Datafunctions, &msg);
                return;
            }
            // A catch-all condition if none of the above match
            if gp_found || gf_found || p_found || f_found {
                let msg = format!("{} is improperly used here", code.name);
                error(&code.name, ErrorKey::Datafunctions, &msg);
                return;
            }

            // If `code.name` is not found at all in the tables, then it can be some passed-in scope.
            // Unfortunately we don't have a complete list of those, so accept any lowercase id and
            // warn if it starts with uppercase. This is not a foolproof check though.
            // TODO: it's in theory possible to build a complete list of possible scope variable names
            if code.name.as_str().chars().next().unwrap().is_uppercase() {
                // TODO: If there is a Custom of the same name, suggest that
                let msg = format!("unknown datafunction {}", &code.name);
                if let Some(alternative) = lookup_alternative(
                    code.name.as_str(),
                    data,
                    is_first,
                    is_last && !expect_promote,
                ) {
                    let info = format!("did you mean {alternative}?");
                    warn_info(&code.name, ErrorKey::Datafunctions, &msg, &info);
                } else {
                    old_warn(&code.name, ErrorKey::Datafunctions, &msg);
                }
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
                ErrorKey::Datafunctions,
                &format!(
                    "{} takes {} arguments but was given {} here",
                    code.name,
                    args.nargs(),
                    code.arguments.len()
                ),
            );
            return;
        }

        // TODO: validate the Faith customs
        #[cfg(feature = "ck3")]
        if curtype != Datatype::Faith && (code.name.is("Custom") && code.arguments.len() == 1)
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

        if code.name.is("Localize") && code.arguments.len() == 1 {
            if let CodeArg::Literal(ref token) = code.arguments[0] {
                data.verify_exists(Item::Localization, token);
            }
        }

        match args {
            Args::NoArgs => (),
            Args::Arg1(dt1) => validate_argument(&code.arguments[0], data, dt1, lang),
            Args::Arg2(dt1, dt2) => {
                validate_argument(&code.arguments[0], data, dt1, lang);
                validate_argument(&code.arguments[1], data, dt2, lang);
            }
            Args::Arg3(dt1, dt2, dt3) => {
                validate_argument(&code.arguments[0], data, dt1, lang);
                validate_argument(&code.arguments[1], data, dt2, lang);
                validate_argument(&code.arguments[2], data, dt3, lang);
            }
            Args::Arg4(dt1, dt2, dt3, dt4) => {
                validate_argument(&code.arguments[0], data, dt1, lang);
                validate_argument(&code.arguments[1], data, dt2, lang);
                validate_argument(&code.arguments[2], data, dt3, lang);
                validate_argument(&code.arguments[3], data, dt4, lang);
            }
            Args::Arg5(dt1, dt2, dt3, dt4, dt5) => {
                validate_argument(&code.arguments[0], data, dt1, lang);
                validate_argument(&code.arguments[1], data, dt2, lang);
                validate_argument(&code.arguments[2], data, dt3, lang);
                validate_argument(&code.arguments[3], data, dt4, lang);
                validate_argument(&code.arguments[4], data, dt5, lang);
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
                    let msg = format!(
                        "{} returns {curtype} but a scope type is needed here",
                        code.name
                    );
                    error(&code.name, ErrorKey::Datafunctions, &msg);
                    return;
                }
            } else {
                let msg = format!(
                    "{} returns {curtype} but a {expect_type} is needed here",
                    code.name
                );
                error(&code.name, ErrorKey::Datafunctions, &msg);
                return;
            }
        }

        i += 1;
    }
}
