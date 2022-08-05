use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Scope {
    // v can contain key = value pairs as well as unadorned values.
    // The latter are inserted as None tokens and Comparator::None
    v: Vec<(Option<Token>, Comparator, ScopeValue)>,
    loc: Loc,
    warnings: Vec<(Token, String)>,
}

#[derive(Clone, Debug)]
pub enum ScopeValue {
    Token(Token),
    Scope(Scope),
}

#[derive(Clone, Debug)]
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
    loc: Loc,
}

impl Scope {
    pub fn new(loc: Loc) -> Self {
        Scope { v: Vec::new(), loc, warnings: Vec::new() }
    }

    pub fn warn(&mut self, token: Token, msg: String) {
        // TODO: also log warn! here
        self.warnings.push((token, msg));
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
}

impl Loc {
    pub fn new(pathname: Rc<PathBuf>, line: usize, column: usize, offset: usize) -> Self {
        Loc { pathname, line, column, offset }
    }
}

impl Token {
    pub fn new(s: String, loc: Loc) -> Self {
        Token { s, loc }
    }
}
