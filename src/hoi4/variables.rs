use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::is_country_tag;
use crate::item::Item;
use crate::report::{report, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_target;

// TODO: lookup builtin variables and check their scopes

pub fn validate_variable(token: &Token, data: &Everything, sc: &mut ScopeContext, sev: Severity) {
    if let Some((varpart, targetpart)) = token.split_once('@') {
        if targetpart.as_str().contains('@') {
            let msg = "could not parse two `@` in one variable";
            report(ErrorKey::Validation, sev).msg(msg).loc(token).push();
            return;
        }
        // TODO: handle the special target parts of builtin variables
        // Second part should resolve to a country, whose tag will be appended to the variable name
        // in the first part.
        validate_target(&targetpart, data, sc, Scopes::Country);

        validate_variable_inner(&varpart, data, sc, sev);
    } else {
        validate_variable_inner(token, data, sc, sev);
    }
}

fn validate_variable_inner(token: &Token, data: &Everything, sc: &mut ScopeContext, sev: Severity) {
    let parts = token.split(':');

    let start = usize::from(parts[0].is("var"));

    for part in &parts[start..] {
        if let Some((basepart, alternate)) = part.split_once('?') {
            // foo?50 will substitute 50 if foo is not defined
            alternate.expect_number();
            validate_variable_inner_part(&basepart, data, sc, sev);
        } else {
            validate_variable_inner_part(part, data, sc, sev);
        }
    }
}

fn validate_variable_inner_part(
    token: &Token,
    data: &Everything,
    _sc: &mut ScopeContext,
    sev: Severity,
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
        validate_variable_name(&arraypart, sev);
    } else {
        validate_variable_name(varname, sev);
    }
}

fn validate_variable_name(token: &Token, sev: Severity) {
    for (i, c) in token.as_str().char_indices() {
        if !(c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()) || c == '_') {
            let msg = format!("unexpected character `{c}` in variable name");
            report(ErrorKey::Validation, sev).msg(msg).loc(token).push();
            return;
        }
    }
}
