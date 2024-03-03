//! Contains the core [`Token`] and [`Loc`] types, which represent pieces of game script and where
//! in the game files they came from.

use std::borrow::{Borrow, Cow};
use std::ffi::OsStr;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::num::NonZeroU32;
use std::ops::{Bound, Range, RangeBounds};
use std::path::{Path, PathBuf};
use std::slice::SliceIndex;

use bumpalo::Bump;

use crate::date::Date;
use crate::fileset::{FileEntry, FileKind};
use crate::pathtable::{PathTable, PathTableIndex};
use crate::report::{err, untidy, ErrorKey};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Loc {
    pub(crate) idx: PathTableIndex,
    pub kind: FileKind,
    /// line 0 means the loc applies to the file as a whole.
    pub line: u32,
    pub column: u16,
    /// Used in macro expansions to point to the macro invocation
    /// in the macro table
    pub link_idx: Option<NonZeroU32>,
}

impl Loc {
    #[must_use]
    pub(crate) fn for_file(pathname: PathBuf, kind: FileKind, fullpath: PathBuf) -> Self {
        let idx = PathTable::store(pathname, fullpath);
        Loc { idx, kind, line: 0, column: 0, link_idx: None }
    }

    pub fn filename(self) -> Cow<'static, str> {
        PathTable::lookup_path(self.idx)
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy()
    }

    pub fn pathname(self) -> &'static Path {
        PathTable::lookup_path(self.idx)
    }

    pub fn fullpath(self) -> &'static Path {
        PathTable::lookup_fullpath(self.idx)
    }

    #[inline]
    pub fn same_file(self, other: Loc) -> bool {
        self.idx == other.idx
    }
}

impl From<&FileEntry> for Loc {
    fn from(entry: &FileEntry) -> Self {
        if let Some(idx) = entry.path_idx() {
            Loc { idx, kind: entry.kind(), line: 0, column: 0, link_idx: None }
        } else {
            Self::for_file(entry.path().to_path_buf(), entry.kind(), entry.fullpath().to_path_buf())
        }
    }
}

impl From<&mut FileEntry> for Loc {
    fn from(entry: &mut FileEntry) -> Self {
        (&*entry).into()
    }
}

impl From<FileEntry> for Loc {
    fn from(entry: FileEntry) -> Self {
        (&entry).into()
    }
}

impl Debug for Loc {
    /// Roll our own `Debug` implementation to handle the path field
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Loc")
            .field("pathindex", &self.idx)
            .field("pathname", &self.pathname())
            .field("fullpath", &self.fullpath())
            .field("kind", &self.kind)
            .field("line", &self.line)
            .field("column", &self.column)
            .field("linkindex", &self.link_idx)
            .finish()
    }
}

/// Leak the string, including any excess capacity.
///
/// It should only be used for large strings, rather than for small, individuals strings,
/// due to the memory overhead. Use [`bump`] instead, which uses a bump allocator to store
/// the strings.
pub(crate) fn leak(s: String) -> &'static str {
    let s = ManuallyDrop::new(s);
    unsafe {
        let s_ptr: *const str = s.as_ref();
        &*s_ptr
    }
}

thread_local!(static STR_BUMP: ManuallyDrop<Bump> = ManuallyDrop::new(Bump::new()));

/// Allocate the string on heap with a bump allocator.
///
/// SAFETY: This is safe as long as no `Bump::reset` is called to deallocate memory
/// and `STR_BUMP` is not dropped when thread exits.
pub(crate) fn bump(s: &str) -> &'static str {
    STR_BUMP.with(|bump| {
        let s = bump.alloc_str(s);
        unsafe {
            let s_ptr: *const str = s;
            &*s_ptr
        }
    })
}

/// A Token consists of a string and its location in the parsed files.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug)]
pub struct Token {
    s: &'static str,
    pub loc: Loc,
}

impl Token {
    #[must_use]
    pub fn new(s: &str, loc: Loc) -> Self {
        Token { s: bump(s), loc }
    }

    #[must_use]
    pub fn from_static_str(s: &'static str, loc: Loc) -> Self {
        Token { s, loc }
    }

    /// Create a `Token` from a substring of the given `Token`.
    #[must_use]
    pub fn subtoken<R>(&self, range: R, loc: Loc) -> Token
    where
        R: RangeBounds<usize> + SliceIndex<str, Output = str>,
    {
        Token { s: &self.s[range], loc }
    }

    /// Create a `Token` from a subtring of the given `Token`,
    /// stripping any whitespace from the created token.
    #[must_use]
    pub fn subtoken_stripped(&self, mut range: Range<usize>, mut loc: Loc) -> Token {
        let mut start = match range.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i + 1,
            Bound::Unbounded => 0,
        };
        let mut end = match range.end_bound() {
            Bound::Included(&i) => i + 1,
            Bound::Excluded(&i) => i,
            Bound::Unbounded => self.s.len(),
        };
        for (i, c) in self.s[range.clone()].char_indices() {
            if !c.is_whitespace() {
                start += i;
                range = start..end;
                break;
            }
            loc.column += 1;
        }
        for (i, c) in self.s[range.clone()].char_indices().rev() {
            if !c.is_whitespace() {
                end = start + i + c.len_utf8();
                range = start..end;
                break;
            }
        }
        Token { s: &self.s[range], loc }
    }

    pub fn as_str(&self) -> &'static str {
        self.s
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

    #[must_use]
    /// Split the token into one or more subtokens, with `ch` as the delimiter.
    /// Updates the locs for the created subtokens.
    /// This is not meant for multiline tokens.
    /// # Panics
    /// May panic if the token's column location exceeds 65535.
    pub fn split(&self, ch: char) -> Vec<Token> {
        let mut pos = 0;
        let mut vec = Vec::new();
        let mut loc = self.loc;
        let mut lines: u32 = 0;
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u16::try_from(cols).expect("internal error: 2^16 columns");
            if c == ch {
                vec.push(self.subtoken(pos..i, loc));
                pos = i + 1;
                loc.column = self.loc.column + cols + 1;
                loc.line = self.loc.line + lines;
            }
            if c == '\n' {
                lines += 1;
            }
        }
        vec.push(self.subtoken(pos.., loc));
        vec
    }

    #[must_use]
    pub fn strip_suffix(&self, sfx: &str) -> Option<Token> {
        self.s.strip_suffix(sfx).map(|pfx| Token::from_static_str(pfx, self.loc))
    }

    #[must_use]
    pub fn strip_prefix(&self, pfx: &str) -> Option<Token> {
        #[allow(clippy::cast_possible_truncation)]
        self.s.strip_prefix(pfx).map(|sfx| {
            let mut loc = self.loc;
            loc.column += pfx.chars().count() as u16;
            Token::from_static_str(sfx, loc)
        })
    }

    #[must_use]
    /// Split the token into two subtokens, with the split at the first occurrence of `ch`.
    /// Updates the locs for the created subtokens.
    /// This is not meant for multiline tokens.
    /// Returns `None` if `ch` was not found in the token.
    /// # Panics
    /// May panic if the token's column location exceeds 65535.
    pub fn split_once(&self, ch: char) -> Option<(Token, Token)> {
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u16::try_from(cols).expect("internal error: 2^16 columns");
            if c == ch {
                let token1 = self.subtoken(..i, self.loc);
                let mut loc = self.loc;
                loc.column += cols + 1;
                let token2 = self.subtoken(i + 1.., loc);
                return Some((token1, token2));
            }
        }
        None
    }

    /// Split the token into two subtokens, with the split at the first instance of `ch`, such that `ch` is part of the first returned token.
    /// Updates the locs for the created subtokens.
    /// This is not meant for multiline tokens.
    /// Returns `None` if `ch` was not found in the token.
    /// # Panics
    /// May panic if the token's column location exceeds 65535.
    #[must_use]
    pub fn split_after(&self, ch: char) -> Option<(Token, Token)> {
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u16::try_from(cols).expect("internal error: 2^16 columns");
            #[allow(clippy::cast_possible_truncation)] // chlen can't be more than 6
            if c == ch {
                let chlen = ch.len_utf8();
                let token1 = self.subtoken(..i + chlen, self.loc);
                let mut loc = self.loc;
                loc.column += cols + chlen as u16;
                let token2 = self.subtoken(i + chlen.., loc);
                return Some((token1, token2));
            }
        }
        None
    }

    /// Create a new token that is a concatenation of this token and `other`, with `c` between them.
    pub fn combine(&mut self, other: &Token, c: char) {
        let mut s = self.s.to_string();
        s.push(c);
        s.push_str(other.s);
        self.s = bump(&s);
    }

    #[must_use]
    /// Return a subtoken of this token, such that all whitespace is removed from the start and end.
    /// Will update the loc of the subtoken.
    /// This is not meant for multiline tokens.
    /// # Panics
    /// May panic if the token's column location exceeds 65535.
    pub fn trim(&self) -> Token {
        let mut real_start = None;
        let mut real_end = self.s.len();
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u16::try_from(cols).expect("internal error: 2^16 columns");
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
            let mut loc = self.loc;
            loc.column += cols;
            self.subtoken(i..real_end, loc)
        } else {
            // all spaces
            Token::from_static_str("", self.loc)
        }
    }

    pub fn expect_number(&self) -> Option<f64> {
        self.check_number();
        if let Ok(v) = self.s.parse::<f64>() {
            Some(v)
        } else {
            err(ErrorKey::Validation).msg("expected number").loc(self).push();
            None
        }
    }

    pub fn get_number(&self) -> Option<f64> {
        self.s.parse::<f64>().ok()
    }

    pub fn is_number(&self) -> bool {
        self.s.parse::<f64>().is_ok()
    }

    pub fn check_number(&self) {
        if let Some(idx) = self.s.find('.') {
            if self.s.len() - idx > 6 {
                let msg = "only 5 decimals are supported";
                let info =
                    "if you give more decimals, you get an error and the number is read as 0";
                err(ErrorKey::Validation).msg(msg).info(info).loc(self).push();
            }
        }
    }

    /// Some files seem not to have the 5-decimal limitation
    pub fn expect_precise_number(&self) -> Option<f64> {
        if let Ok(v) = self.s.parse::<f64>() {
            Some(v)
        } else {
            err(ErrorKey::Validation).msg("expected number").loc(self).push();
            None
        }
    }

    pub fn expect_integer(&self) -> Option<i64> {
        if let Ok(v) = self.s.parse::<i64>() {
            Some(v)
        } else {
            err(ErrorKey::Validation).msg("expected integer").loc(self).push();
            None
        }
    }

    pub fn get_integer(&self) -> Option<i64> {
        self.s.parse::<i64>().ok()
    }

    pub fn is_integer(&self) -> bool {
        self.s.parse::<i64>().is_ok()
    }

    pub fn expect_date(&self) -> Option<Date> {
        if let Ok(v) = self.s.parse::<Date>() {
            if self.s.ends_with('.') {
                untidy(ErrorKey::Validation).msg("trailing dot on date").loc(self).push();
            }
            Some(v)
        } else {
            err(ErrorKey::Validation).msg("expected date").loc(self).push();
            None
        }
    }

    pub fn get_date(&self) -> Option<Date> {
        self.s.parse::<Date>().ok()
    }

    pub fn is_date(&self) -> bool {
        self.s.parse::<Date>().is_ok()
    }

    #[must_use]
    pub fn linked(mut self, link_idx: Option<NonZeroU32>) -> Self {
        self.loc.link_idx = link_idx;
        self
    }
}

impl From<&Token> for Token {
    fn from(token: &Token) -> Token {
        token.clone()
    }
}

/// Tokens are compared for equality regardless of their loc.
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
    }
}

impl Eq for Token {}

impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.s.hash(state);
    }
}

impl Borrow<str> for Token {
    fn borrow(&self) -> &str {
        self.s
    }
}

impl From<Loc> for Token {
    fn from(loc: Loc) -> Self {
        Token { s: "", loc }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.s)
    }
}
