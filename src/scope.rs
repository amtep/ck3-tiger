use std::ffi::OsStr;
use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Scope {
    // v can contain key = value pairs as well as unadorned values.
    // The latter are inserted as None tokens and Comparator::None
    v: Vec<(Option<Token>, Comparator, ScopeValue)>,
    loc: Loc,
}

#[derive(Clone, Debug)]
pub enum ScopeValue {
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

    pub fn add_value(&mut self, value: ScopeValue) {
        self.v.push((None, Comparator::None, value));
    }

    pub fn add_key_value(&mut self, key: Token, cmp: Comparator, value: ScopeValue) {
        self.v.push((Some(key), cmp, value));
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

impl Loc {
    pub fn new(pathname: Rc<PathBuf>, line: usize, column: usize, offset: usize) -> Self {
        Loc {
            pathname,
            line,
            column,
            offset,
        }
    }

    pub fn marker(&self) -> String {
        let fname = self
            .pathname
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy();
        format!("{}:{}:{}: ", fname, self.line, self.column)
    }
}

impl Token {
    pub fn new(s: String, loc: Loc) -> Self {
        Token { s, loc }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.s)
    }
}
