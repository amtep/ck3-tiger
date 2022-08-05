use std::path::Path;
use std::rc::Rc;

use crate::scope::{Comparator, Loc, Scope, Token};

enum State {
    Neutral,
    QString,
    Id,
    Comparator,
    Comment,
}

trait CharExt {
    fn is_id_char(self) -> bool;
    fn is_comparator_char(self) -> bool;
}

impl CharExt for char {
    fn is_id_char(self) -> bool {
        self.is_alphanumeric() || self == '.' || self == ':' || self == '$'
    }

    fn is_comparator_char(self) -> bool {
        self == '<' || self == '>' || self == '!' || self == '='
    }
}

pub fn parse_pdx(pathname: &Path, content: &str) -> Scope {
    let pathname = Rc::new(pathname.to_path_buf());
    let mut state = State::Neutral;
    let mut loc = Loc::new(pathname, 0, 1, 1);
    let mut token_start = 0;
    let mut scope = Scope::new(loc.clone());

    for (i, c) in content.char_indices() {
        loc.offset = i;
        let next_i = i + c.len_utf8();

        match state {
            State::Neutral => {
                if c.is_whitespace() {
                    ()
                } else if c == '"' {
                    state = State::QString;
                } else if c == '#' {
                    state = State::Comment;
                } else if c.is_comparator_char() {
                    state = State::Comparator;
                } else if c.is_id_char() {
                    state = State::Id;
                } else {
                    let token = Token::new(c.to_string(), loc.clone());
                    scope.warn(token, format!("Unrecognized character {}", c));
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
                    scope.warn(token, "Quoted string not closed".to_string());
                }
            }
            State::Id => {
                if c == '"' {
                    // The quoted string actually becomes part of this id
                    state = State::QString;
                } else if c.is_id_char() {
                    ()
                } else {
                    let id = content[token_start..i].to_string();
                    let token = Token::new(id, loc.clone());
                    // TODO: record token

                    if c.is_comparator_char() {
                        state = State::Comparator;
                    } else if c.is_whitespace() {
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
                    }
                    token_start = i;
                }
            }
            State::Comparator => {
                if c.is_comparator_char() {
                    ()
                } else {
                    let s = &content[token_start..i];
                    let cmp = Comparator::from_str(s).unwrap_or_else(|| {
                        let token = Token::new(s.to_string(), loc.clone());
                        scope.warn(token, format!("Unrecognized comparator {}", s));
                        Comparator::Eq
                    });
                    // TODO: record comparator

                    if c == '"' {
                        state = State::QString;
                    } else if c.is_id_char() {
                        state = State::Id;
                    } else if c.is_whitespace() {
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
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
    scope
}
