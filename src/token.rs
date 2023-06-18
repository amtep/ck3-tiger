use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

use crate::block::Date;
use crate::errorkey::ErrorKey;
use crate::errors::error;
use crate::fileset::{FileEntry, FileKind};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Loc {
    pub pathname: Rc<PathBuf>,
    pub kind: FileKind,
    /// line 0 means the loc applies to the file as a whole.
    pub line: usize,
    pub column: usize,
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

    pub fn lowercase_is(&self, s: &str) -> bool {
        self.s.to_lowercase() == s
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.s.starts_with(s)
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
                loc.column += cols + 1;
                let token2 = Token::new(self.s[i + 1..].to_string(), loc);
                return Some((token1, token2));
            }
        }
        None
    }

    /// Split the token at the first instance of ch, such that ch is part of the first returned token.
    pub fn split_after(&self, ch: char) -> Option<(Token, Token)> {
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            if c == ch {
                let chlen = ch.len_utf8();
                let token1 = Token::new(self.s[..i + chlen].to_string(), self.loc.clone());
                let mut loc = self.loc.clone();
                loc.column += cols + chlen;
                let token2 = Token::new(self.s[i + chlen..].to_string(), loc);
                return Some((token1, token2));
            }
        }
        None
    }

    pub fn trim(&self) -> Token {
        let mut real_start = None;
        let mut real_end = self.s.len();
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            if c != ' ' {
                real_start = Some((cols, i));
                break;
            }
        }
        // looping over the indices is safe here because we're only skipping spaces
        while real_end > 0 && &self.s[real_end - 1..real_end] == " " {
            real_end -= 1;
        }
        if let Some((cols, i)) = real_start {
            let mut loc = self.loc.clone();
            loc.column += cols;
            Token::new(self.s[i..real_end].to_string(), loc)
        } else {
            // all spaces
            Token::new(String::new(), self.loc.clone())
        }
    }

    pub fn into_string(self) -> String {
        self.s
    }

    pub fn expect_number(&self) -> Option<f64> {
        if let Ok(v) = self.s.parse::<f64>() {
            Some(v)
        } else {
            error(self, ErrorKey::Validation, "expected number");
            None
        }
    }

    pub fn is_number(&self) -> bool {
        self.s.parse::<f64>().is_ok()
    }

    pub fn expect_integer(&self) -> Option<i64> {
        if let Ok(v) = self.s.parse::<i64>() {
            Some(v)
        } else {
            error(self, ErrorKey::Validation, "expected integer");
            None
        }
    }

    pub fn is_integer(&self) -> bool {
        self.s.parse::<i64>().is_ok()
    }

    pub fn expect_date(&self) -> Option<Date> {
        if let Ok(v) = self.s.parse::<Date>() {
            Some(v)
        } else {
            error(self, ErrorKey::Validation, "expected date");
            None
        }
    }

    pub fn is_date(&self) -> bool {
        self.s.parse::<Date>().is_ok()
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
