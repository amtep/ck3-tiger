use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::{is_country_tag, stringify_choices};
use crate::hoi4::tables::variables::{Suffix, ARRAYS_MAP, VARIABLES_MAP};
use crate::item::Item;
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::report::{report, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Array {
    Yes,
    No,
}

pub fn validate_variable(token: &Token, data: &Everything, sc: &mut ScopeContext, sev: Severity) {
    if let Some((varpart, targetpart)) = token.split_once('@') {
        if targetpart.as_str().contains('@') {
            let msg = "could not parse two `@` in one variable";
            report(ErrorKey::Validation, sev).msg(msg).loc(token).push();
            return;
        }
        validate_variable_inner(&varpart, data, sc, sev, Some(&targetpart));
    } else {
        validate_variable_inner(token, data, sc, sev, None);
    }
}

fn validate_variable_inner(
    token: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    sev: Severity,
    target: Option<&Token>,
) {
    let parts = token.split(':');

    let start = usize::from(parts[0].is("var"));
    let last = parts.len() - 1;

    for (i, part) in parts.iter().enumerate().skip(start) {
        let target = if i == last { target } else { None };
        if let Some((basepart, alternate)) = part.split_once('?') {
            // foo?50 will substitute 50 if foo is not defined
            alternate.expect_number();
            validate_variable_inner_part(&basepart, data, sc, sev, target);
        } else {
            validate_variable_inner_part(part, data, sc, sev, target);
        }
    }
}

fn validate_variable_inner_part(
    token: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    sev: Severity,
    target: Option<&Token>,
) {
    let parts = token.split('.');

    if parts.len() > 1 {
        let mut ok = parts[0].lowercase_is("global") && parts.len() == 2;
        ok = ok || parts[0].lowercase_is("root") && parts.len() == 2;
        ok = ok || parts[0..parts.len() - 1].iter().all(|p| p.lowercase_is("from"));
        ok = ok || parts[0..parts.len() - 1].iter().all(|p| p.lowercase_is("prev"));
        if !ok && parts[0].is_integer() {
            data.verify_exists(Item::State, &parts[0]);
            ok = parts.len() == 2;
        }
        if !ok && is_country_tag(parts[0].as_str()) {
            data.verify_exists(Item::CountryTag, &parts[0]);
            ok = parts.len() == 2;
        }
        if !ok {
            let msg = "could not parse variable's qualifier";
            report(ErrorKey::Validation, sev).msg(msg).loc(token).push();
            return;
        }
    }

    let varname = &parts[parts.len() - 1];
    if let Some((arraypart, arrayidx)) = varname.split_once('^') {
        if !arrayidx.is("num") {
            arrayidx.expect_integer();
        }
        validate_variable_name(&arraypart, data, sc, sev, target, Array::Yes);
    } else {
        validate_variable_name(varname, data, sc, sev, target, Array::No);
    }
}

fn validate_variable_name(
    token: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
    sev: Severity,
    target: Option<&Token>,
    array: Array,
) {
    for (i, c) in token.as_str().char_indices() {
        if !(c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()) || c == '_') {
            let msg = format!("unexpected character `{c}` in variable name");
            report(ErrorKey::Validation, sev).msg(msg).loc(token).push();
            return;
        }
    }

    #[allow(clippy::collapsible_else_if)]
    if array == Array::Yes {
        if let Some((_, _, suffix_type)) = ARRAYS_MAP.get(token.as_str()) {
            validate_suffix(token, target, *suffix_type, "array", sc, data, sev);
        } else {
            // Second part should resolve to a country, whose tag will be appended to the array name
            // in the first part.
            if let Some(target) = target {
                validate_target(target, data, sc, Scopes::Country);
                data.variables.verify_list_prefix_exists(token, sev);
            } else {
                data.variables.verify_list_exists(token, sev);
            }
        }
    } else {
        if let Some((_, _, suffix_type)) = VARIABLES_MAP.get(token.as_str()) {
            validate_suffix(token, target, *suffix_type, "variable", sc, data, sev);
        } else {
            // Second part should resolve to a country, whose tag will be appended to the variable name
            // in the first part.
            if let Some(target) = target {
                validate_target(target, data, sc, Scopes::Country);
                data.variables.verify_variable_prefix_exists(token, sev);
            } else {
                data.variables.verify_variable_exists(token, sev);
            }
        }
    }
}

fn validate_suffix(
    name: &Token,
    target: Option<&Token>,
    suffix_type: Suffix,
    what: &str,
    sc: &mut ScopeContext,
    data: &Everything,
    sev: Severity,
) {
    if let Some(target) = target {
        match suffix_type {
            Suffix::None => {
                let msg = format!("unexpected @target on this builtin {what}");
                report(ErrorKey::Variables, sev).msg(msg).loc(target).push();
            }
            Suffix::Scope(scopes) => {
                validate_target(target, data, sc, scopes);
            }
            Suffix::Item(itype) => {
                data.verify_exists(itype, target);
            }
            Suffix::OptionalChoice(choices) => {
                if !choices.contains(&target.as_str()) {
                    let msg = format!("expected one of {}", stringify_choices(choices));
                    report(ErrorKey::Variables, sev).msg(msg).loc(target).push();
                }
            }
            Suffix::Modif => {
                verify_modif_exists(target, data, ModifKinds::all(), sev);
            }
            Suffix::ShipTypes => {
                let choices = &["carrier", "capital", "screen", "submarine"];
                if !choices.contains(&target.as_str()) {
                    data.verify_exists(Item::SubUnit, target);
                }
            }
        }
    } else {
        match suffix_type {
            Suffix::None | Suffix::OptionalChoice(_) => (),
            _ => {
                let msg = "expected @target after this builtin variable";
                report(ErrorKey::Variables, sev).msg(msg).loc(name).push();
            }
        }
    }
}
