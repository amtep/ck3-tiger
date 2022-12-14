use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

use crate::fileset::{FileEntry, FileKind};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Loc {
    pub pathname: Rc<PathBuf>,
    pub kind: FileKind,
    /// line 0 means the loc applies to the file as a whole.
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    /// Used in macro expansions to point to the macro invocation
    pub link: Option<Rc<Loc>>,
}

impl Loc {
    pub fn for_file(pathname: Rc<PathBuf>, kind: FileKind) -> Self {
        Loc {
            pathname,
            kind,
            line: 0,
            column: 0,
            offset: 0,
            link: None,
        }
    }

    pub fn for_entry(entry: &FileEntry) -> Self {
        Self::for_file(Rc::new(entry.path().to_path_buf()), entry.kind())
    }

    pub fn marker(&self) -> String {
        if self.line == 0 {
            format!("[{}] {}: ", self.kind, self.pathname.display())
        } else {
            format!(
                "[{}] {}:{}:{}: ",
                self.kind,
                self.pathname.display(),
                self.line,
                self.column
            )
        }
    }

    pub fn line_marker(&self) -> String {
        format!("line {}", self.line)
    }

    pub fn file_marker(&self) -> String {
        format!("[{}] file {}", self.kind, self.pathname.display())
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

    pub fn split(&self, ch: char) -> Vec<Token> {
        let mut pos = 0;
        let mut vec = Vec::new();
        let mut loc = self.loc.clone();
        let mut lines = 0;
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            if c == ch {
                vec.push(Token::new(self.s[pos..i].to_string(), loc.clone()));
                pos = i + 1;
                loc.offset = self.loc.offset + i + 1;
                loc.column = self.loc.column + cols + 1;
                loc.line = self.loc.line + lines;
            }
            if c == '\n' {
                lines += 1;
            }
        }
        vec.push(Token::new(self.s[pos..].to_string(), loc));
        vec
    }

    pub fn split_once(&self, ch: char) -> Option<(Token, Token)> {
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            if c == ch {
                let token1 = Token::new(self.s[..i].to_string(), self.loc.clone());
                let mut loc = self.loc.clone();
                loc.offset += i + 1;
                loc.column += cols + 1;
                let token2 = Token::new(self.s[i + 1..].to_string(), loc);
                return Some((token1, token2));
            }
        }
        None
    }

    pub fn into_string(self) -> String {
        self.s
    }
}

/// Tokens are compared for equality regardless of their loc.
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
    }
}
impl Eq for Token {}

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
