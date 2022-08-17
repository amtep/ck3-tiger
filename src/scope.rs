use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

pub mod validator;

use crate::everything::FileKind;

#[derive(Clone, Debug)]
pub struct Scope {
    // v can contain key = value pairs as well as unadorned values.
    // The latter are inserted as None tokens and Comparator::None
    v: Vec<(Option<Token>, Comparator, ScopeOrValue)>,
    pub loc: Loc,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub enum ScopeOrValue {
    Token(Token),
    Scope(Scope),
}

#[derive(Copy, Clone, Debug)]
pub enum Comparator {
    None,
    Eq, // Eq is also Assign
    Lt,
    Gt,
    Le,
    Ge,
    Ne,
}

#[derive(Clone, Debug)]
pub struct Loc {
    pub pathname: Rc<PathBuf>,
    pub kind: FileKind,
    /// line 0 means the loc applies to the file as a whole.
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Clone, Debug)]
pub struct Token {
    s: String,
    pub loc: Loc,
}

impl Scope {
    pub fn new(loc: Loc) -> Self {
        Scope { v: Vec::new(), loc }
    }

    pub fn add_value(&mut self, value: ScopeOrValue) {
        self.v.push((None, Comparator::None, value));
    }

    pub fn add_key_value(&mut self, key: Token, cmp: Comparator, value: ScopeOrValue) {
        self.v.push((Some(key), cmp, value));
    }

    pub fn filename(&self) -> Cow<str> {
        self.loc.filename()
    }

    pub fn token(&self) -> Token {
        Token::new(String::new(), self.loc.clone())
    }

    /// Get the value of a single `name = value` assignment
    pub fn get_field_value(&self, name: &str) -> Option<Token> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.as_str() == name {
                    match v {
                        ScopeOrValue::Token(t) => return Some(t.clone()),
                        ScopeOrValue::Scope(_) => (),
                    }
                }
            }
        }
        None
    }

    /// Get all the values of `name = value` assignments in this scope
    pub fn get_field_values(&self, name: &str) -> Vec<Token> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    match v {
                        ScopeOrValue::Token(t) => vec.push(t.clone()),
                        ScopeOrValue::Scope(_) => (),
                    }
                }
            }
        }
        vec
    }

    /// Get the values of a single `name = { value ... }` assignment
    pub fn get_field_list(&self, name: &str) -> Option<Vec<Token>> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.as_str() == name {
                    match v {
                        ScopeOrValue::Token(_) => (),
                        ScopeOrValue::Scope(s) => {
                            return Some(s.get_values());
                        }
                    }
                }
            }
        }
        None
    }

    /// Get all the unkeyed values in this scope
    pub fn get_values(&self) -> Vec<Token> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if k.is_none() {
                match v {
                    ScopeOrValue::Token(t) => vec.push(t.clone()),
                    ScopeOrValue::Scope(_) => (),
                }
            }
        }
        vec
    }
}

impl Comparator {
    pub fn from_str(s: &str) -> Option<Self> {
        if s == "=" {
            Some(Comparator::Eq)
        } else if s == "<" {
            Some(Comparator::Lt)
        } else if s == ">" {
            Some(Comparator::Gt)
        } else if s == "<=" {
            Some(Comparator::Le)
        } else if s == ">=" {
            Some(Comparator::Ge)
        } else if s == "!=" {
            Some(Comparator::Ne)
        } else {
            None
        }
    }

    pub fn from_token(token: &Token) -> Option<Self> {
        Self::from_str(&token.s)
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Comparator::Eq => write!(f, "="),
            Comparator::Lt => write!(f, "<"),
            Comparator::Gt => write!(f, ">"),
            Comparator::Le => write!(f, "<="),
            Comparator::Ge => write!(f, ">="),
            Comparator::Ne => write!(f, "!="),
            Comparator::None => Ok(()),
        }
    }
}

impl Loc {
    pub fn new(pathname: Rc<PathBuf>, kind: FileKind) -> Self {
        Loc {
            pathname,
            kind,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    pub fn for_file(pathname: Rc<PathBuf>, kind: FileKind) -> Self {
        Loc {
            pathname,
            kind,
            line: 0,
            column: 0,
            offset: 0,
        }
    }

    pub fn marker(&self) -> String {
        if self.line == 0 {
            format!("{}: ", self.filename())
        } else {
            format!("{}:{}:{}: ", self.filename(), self.line, self.column)
        }
    }

    pub fn line_marker(&self) -> String {
        if self.line == 0 {
            format!("{}: ", self.filename())
        } else {
            format!("{}:{}: ", self.filename(), self.line)
        }
    }

    pub fn filename(&self) -> Cow<str> {
        self.pathname
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy()
    }
}

impl Token {
    pub fn new(s: String, loc: Loc) -> Self {
        Token { s, loc }
    }

    pub fn as_str(&self) -> &str {
        &self.s
    }
}

impl From<Loc> for Token {
    fn from(loc: Loc) -> Self {
        Token {
            s: String::new(),
            loc,
        }
    }
}

impl From<&Loc> for Token {
    fn from(loc: &Loc) -> Self {
        Token {
            s: String::new(),
            loc: loc.clone(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.s)
    }
}
