use anyhow::{bail, Result};
use std::mem::swap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::errors::{ErrorKey, Errors};
use crate::scope::{Comparator, Loc, Scope, ScopeValue, Token};

#[derive(Copy, Clone, Debug)]
enum State {
    Neutral,
    QString,
    Id,
    Comparator,
    Comment,
}

#[allow(clippy::wrong_self_convention)]
trait CharExt {
    fn is_id_char(self) -> bool;
    fn is_comparator_char(self) -> bool;
}

impl CharExt for char {
    fn is_id_char(self) -> bool {
        self.is_ascii_alphanumeric() || self == '.' || self == ':' || self == '$' || self == '_'
    }

    fn is_comparator_char(self) -> bool {
        self == '<' || self == '>' || self == '!' || self == '='
    }
}

struct Parser<'a> {
    pathname: Rc<PathBuf>,
    errors: &'a mut Errors,
    current: ParseLevel,
    stack: Vec<ParseLevel>,
    brace_error: bool,
}

struct ParseLevel {
    scope: Scope,
    key: Option<Token>,
    comp: Option<(Comparator, Token)>,
}

impl<'a> Parser<'a> {
    fn unknown_char(&mut self, c: char, loc: Loc) {
        let token = Token::new(c.to_string(), loc);
        self.errors.error(
            token,
            ErrorKey::ParseError,
            format!("Unrecognized character {}", c),
        );
    }

    fn token(&mut self, token: Token) {
        if let Some(key) = self.current.key.take() {
            if let Some((comp, _)) = self.current.comp.take() {
                self.current
                    .scope
                    .add_key_value(key, comp, ScopeValue::Token(token));
            } else {
                self.current.scope.add_value(ScopeValue::Token(key));
                self.current.key = Some(token);
            }
        } else {
            self.current.key = Some(token);
        }
    }

    fn scope_value(&mut self, scope: Scope) {
        // Like token(), but scope values cannot become keys
        if let Some(key) = self.current.key.take() {
            if let Some((comp, _)) = self.current.comp.take() {
                self.current
                    .scope
                    .add_key_value(key, comp, ScopeValue::Scope(scope));
            } else {
                self.current.scope.add_value(ScopeValue::Token(key));
                self.current.scope.add_value(ScopeValue::Scope(scope));
            }
        } else {
            self.current.scope.add_value(ScopeValue::Scope(scope));
        }
    }

    fn comparator(&mut self, token: Token) {
        let cmp = Comparator::from_token(&token).unwrap_or_else(|| {
            self.errors.error(
                token.clone(),
                ErrorKey::ParseError,
                format!("Unrecognized comparator '{}'", token),
            );
            Comparator::Eq
        });

        if self.current.key.is_none() {
            let msg = format!("Unexpected comparator '{}'", token);
            self.errors.error(token, ErrorKey::ParseError, msg);
        } else {
            if self.current.comp.is_some() {
                let msg = format!("Double comparator '{}'", token);
                self.errors.error(token.clone(), ErrorKey::ParseError, msg);
            }
            self.current.comp = Some((cmp, token));
        }
    }

    fn end_assign(&mut self) {
        if let Some(key) = self.current.key.take() {
            if let Some((_, comp_token)) = self.current.comp.take() {
                let msg = "Comparator without value".to_string();
                self.errors.error(comp_token, ErrorKey::ParseError, msg);
            }
            self.current.scope.add_value(ScopeValue::Token(key));
        }
    }

    fn open_brace(&mut self, loc: Loc) {
        let mut new_level = ParseLevel {
            scope: Scope::new(loc),
            key: None,
            comp: None,
        };
        swap(&mut new_level, &mut self.current);
        self.stack.push(new_level);
    }

    fn close_brace(&mut self, loc: Loc) {
        self.end_assign();
        if let Some(mut prev_level) = self.stack.pop() {
            swap(&mut self.current, &mut prev_level);
            self.scope_value(prev_level.scope);
        } else {
            self.brace_error = true;
            self.errors.error(
                Token::new("}".to_string(), loc),
                ErrorKey::ParseError,
                "Unexpected }".to_string(),
            );
        }
    }

    fn eof(mut self) -> Result<Scope> {
        self.end_assign();
        while let Some(mut prev_level) = self.stack.pop() {
            self.brace_error = true;
            self.errors.error(
                Token::new("{".to_string(), self.current.scope.loc.clone()),
                ErrorKey::ParseError,
                "Opening { was never closed".to_string(),
            );
            swap(&mut self.current, &mut prev_level);
            self.scope_value(prev_level.scope);
        }
        // Brace errors mean we shoudln't try to use the file at all,
        // since its structure is unclear. Validating such a file would
        // just produce a cascade of irrelevant errors.
        if self.brace_error {
            bail!(
                "Could not parse {} due to brace mismatch",
                self.pathname.display()
            );
        } else {
            Ok(self.current.scope)
        }
    }
}

pub fn parse_pdx(pathname: &Path, content: &str, errors: &mut Errors) -> Result<Scope> {
    let pathname = Rc::new(pathname.to_path_buf());
    let mut loc = Loc::new(pathname.clone());
    let mut parser = Parser {
        pathname,
        errors,
        current: ParseLevel {
            scope: Scope::new(loc.clone()),
            key: None,
            comp: None,
        },
        stack: Vec::new(),
        brace_error: false,
    };
    let mut state = State::Neutral;
    let mut token_start = 0;

    for (i, c) in content.char_indices() {
        loc.offset = i;
        let next_i = i + c.len_utf8();

        match state {
            State::Neutral => {
                if c.is_whitespace() {
                } else if c == '"' {
                    state = State::QString;
                } else if c == '#' {
                    state = State::Comment;
                } else if c.is_comparator_char() {
                    state = State::Comparator;
                } else if c.is_id_char() {
                    state = State::Id;
                } else if c == '{' {
                    parser.open_brace(loc.clone());
                } else if c == '}' {
                    parser.close_brace(loc.clone());
                } else {
                    parser.unknown_char(c, loc.clone());
                }
                token_start = i;
            }
            State::Comment => {
                if c == '\n' {
                    state = State::Neutral;
                }
            }
            State::QString => {
                if c == '"' {
                    state = State::Id;
                } else if c == '\n' {
                    let s = content[token_start..next_i].to_string();
                    let token = Token::new(s, loc.clone());
                    parser.errors.warn(
                        token,
                        ErrorKey::ParseError,
                        "Quoted string not closed".to_string(),
                    );
                }
            }
            State::Id => {
                if c == '"' {
                    // The quoted string actually becomes part of this id
                    state = State::QString;
                } else if c.is_id_char() {
                } else {
                    let id = content[token_start..i].to_string();
                    let token = Token::new(id, loc.clone());
                    parser.token(token);

                    if c.is_comparator_char() {
                        state = State::Comparator;
                    } else if c.is_whitespace() {
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c == '{' {
                        parser.open_brace(loc.clone());
                        state = State::Neutral;
                    } else if c == '}' {
                        parser.close_brace(loc.clone());
                        state = State::Neutral;
                    } else {
                        parser.unknown_char(c, loc.clone());
                        state = State::Neutral;
                    }
                    token_start = i;
                }
            }
            State::Comparator => {
                if c.is_comparator_char() {
                } else {
                    let s = content[token_start..i].to_string();
                    let token = Token::new(s, loc.clone());
                    parser.comparator(token);

                    if c == '"' {
                        state = State::QString;
                    } else if c.is_id_char() {
                        state = State::Id;
                    } else if c.is_whitespace() {
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c == '{' {
                        parser.open_brace(loc.clone());
                        state = State::Neutral;
                    } else if c == '}' {
                        parser.close_brace(loc.clone());
                        state = State::Neutral;
                    } else {
                        parser.unknown_char(c, loc.clone());
                        state = State::Neutral;
                    }
                    token_start = i;
                }
            }
        }

        if c == '\n' {
            loc.line += 1;
            loc.column = 1;
        } else {
            loc.column += 1;
        }
    }

    // Deal with state at end of file
    match state {
        State::QString => {
            let s = content[token_start..].to_string();
            let token = Token::new(s, loc);
            parser.errors.error(
                token.clone(),
                ErrorKey::ParseError,
                "Quoted string not closed".to_string(),
            );
            parser.token(token);
        }
        State::Id => {
            let s = content[token_start..].to_string();
            let token = Token::new(s, loc);
            parser.token(token);
        }
        State::Comparator => {
            let s = content[token_start..].to_string();
            let token = Token::new(s, loc);
            parser.comparator(token);
        }
        _ => (),
    }

    parser.eof()
}
