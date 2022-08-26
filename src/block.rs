use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

pub mod validator;

use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info};
use crate::fileset::FileKind;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub enum BlockOrValue {
    Token(Token),
    Block(Block),
}

type BlockItem = (Option<Token>, Comparator, BlockOrValue);

#[derive(Clone, Debug)]
pub struct Block {
    // v can contain key = value pairs as well as unadorned values.
    // The latter are inserted as None tokens and Comparator::None
    v: Vec<BlockItem>,
    pub loc: Loc,
}

impl Block {
    pub fn new(loc: Loc) -> Self {
        Block { v: Vec::new(), loc }
    }

    pub fn add_value(&mut self, value: BlockOrValue) {
        self.v.push((None, Comparator::None, value));
    }

    pub fn add_key_value(&mut self, key: Token, cmp: Comparator, value: BlockOrValue) {
        self.v.push((Some(key), cmp, value));
    }

    pub fn append(&mut self, other: &mut Block) {
        self.v.append(&mut other.v);
    }

    pub fn filename(&self) -> Cow<str> {
        self.loc.filename()
    }

    /// Get the value of a single `name = value` assignment
    pub fn get_field_value(&self, name: &str) -> Option<&Token> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BlockOrValue::Token(t) => return Some(t),
                        BlockOrValue::Block(_) => (),
                    }
                }
            }
        }
        None
    }

    /// Get all the values of `name = value` assignments in this block
    pub fn get_field_values(&self, name: &str) -> Vec<Token> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BlockOrValue::Token(t) => vec.push(t.clone()),
                        BlockOrValue::Block(_) => (),
                    }
                }
            }
        }
        vec
    }

    /// Get the block of a `name = { ... }` assignment
    pub fn get_field_block(&self, name: &str) -> Option<&Block> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BlockOrValue::Token(_) => (),
                        BlockOrValue::Block(s) => return Some(s),
                    }
                }
            }
        }
        None
    }

    /// Get all the blocks of `name = { ... }` assignments in this block
    pub fn get_field_blocks(&self, name: &str) -> Vec<&Block> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BlockOrValue::Token(_) => (),
                        BlockOrValue::Block(s) => vec.push(s),
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
                if key.is(name) {
                    match v {
                        BlockOrValue::Token(_) => (),
                        BlockOrValue::Block(s) => {
                            return Some(s.get_values());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_field(&self, name: &str) -> Option<&BlockOrValue> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    return Some(v);
                }
            }
        }
        None
    }

    /// Get all the unkeyed values in this block
    pub fn get_values(&self) -> Vec<Token> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if k.is_none() {
                match v {
                    BlockOrValue::Token(t) => vec.push(t.clone()),
                    BlockOrValue::Block(_) => (),
                }
            }
        }
        vec
    }

    pub fn dbg_keys(&self) {
        for (k, _, _) in &self.v {
            if let Some(k) = k {
                let key = k.as_str();
                dbg!(key);
            }
        }
    }

    pub fn get_key(&self, name: &str) -> Option<&Token> {
        for (k, _, _) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    return Some(key);
                }
            }
        }
        None
    }

    pub fn iter_items(&self) -> std::slice::Iter<BlockItem> {
        self.v.iter()
    }

    pub fn iter_definitions_warn(&self) -> IterDefinitions {
        IterDefinitions {
            iter: self.v.iter(),
            warn: true,
        }
    }
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

#[derive(Clone, Debug)]
pub struct Loc {
    pub pathname: Rc<PathBuf>,
    pub kind: FileKind,
    /// line 0 means the loc applies to the file as a whole.
    pub line: usize,
    pub column: usize,
    pub offset: usize,
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
            format!("{}: ", self.pathname.display())
        } else {
            format!(
                "{}:{}:{}: ",
                self.pathname.display(),
                self.line,
                self.column
            )
        }
    }

    pub fn line_marker(&self) -> String {
        if self.line == 0 {
            format!("{}: ", self.pathname.display())
        } else {
            format!("{}:{}: ", self.pathname.display(), self.line)
        }
    }

    pub fn filename(&self) -> Cow<str> {
        self.pathname
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy()
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    s: String,
    pub loc: Loc,
}

impl Token {
    pub fn new(s: String, loc: Loc) -> Self {
        Token { s, loc }
    }

    pub fn as_str(&self) -> &str {
        &self.s
    }

    pub fn is(&self, s: &str) -> bool {
        self.s == s
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

/// A type for callers who are only interested in "definition"-style blocks, where there
/// are no comparisons and no loose blocks.
#[derive(Clone, Debug)]
pub enum DefinitionItem<'a> {
    Assignment(&'a Token, &'a Token), // key = value
    Definition(&'a Token, &'a Block), // key = { definition }
    Keyword(&'a Token),
}

#[derive(Clone, Debug)]
pub struct IterDefinitions<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterDefinitions<'a> {
    type Item = DefinitionItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for (k, cmp, v) in self.iter.by_ref() {
            if let Some(key) = k {
                if !matches!(cmp, Comparator::Eq) {
                    if self.warn {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    continue;
                }
                return match v {
                    BlockOrValue::Token(t) => Some(DefinitionItem::Assignment(key, t)),
                    BlockOrValue::Block(b) => Some(DefinitionItem::Definition(key, b)),
                };
            }
            match v {
                BlockOrValue::Token(t) => return Some(DefinitionItem::Keyword(t)),
                BlockOrValue::Block(b) => {
                    if self.warn {
                        error_info(
                            b,
                            ErrorKey::Validation,
                            "unexpected block",
                            "Did you forget an = ?",
                        );
                    }
                }
            }
        }
        None
    }
}
