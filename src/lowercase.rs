//! Type-safety wrapper for strings that must be lowercase

use std::borrow::{Borrow, Cow};
use std::fmt::{Display, Error, Formatter};
#[cfg(any(feature = "vic3", feature = "imperator"))]
use std::slice::SliceIndex;
#[cfg(any(feature = "vic3", feature = "imperator"))]
use std::str::RMatchIndices;

/// Wraps a string (either owned or `&str`) and guarantees that it's lowercase.
///
/// This allows interfaces that expect lowercase strings to declare this expectation in their
/// argument type, so that the caller can choose how to fulfill it.
///
/// Only ASCII characters are lowercased. This is faster than full unicode casemapping, and it
/// matches what the game engine does.
///
/// The lowercase string is a [`Cow<str>`] internally, so it can be either owned or borrowed.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Lowercase<'a>(Cow<'a, str>);

impl<'a> Lowercase<'a> {
    /// Take a string and return the lowercased version.
    pub fn new(s: &'a str) -> Self {
        // Avoid allocating if it's not necessary
        if s.chars().any(|c| c.is_ascii_uppercase()) {
            Lowercase(Cow::Owned(s.to_ascii_lowercase()))
        } else {
            Lowercase(Cow::Borrowed(s))
        }
    }

    /// Take a string that is known to already be lowercase and return a `Lowercase` wrapper for it.
    ///
    /// This operation is free.
    pub fn new_unchecked(s: &'a str) -> Self {
        Lowercase(Cow::Borrowed(s))
    }

    /// Take an owned `String` that is known to already be lowercase and return a `Lowercase` wrapper
    pub fn from_string_unchecked(s: String) -> Self {
        Lowercase(Cow::Owned(s))
    }

    pub fn empty() -> &'static Self {
        const EMPTY_LOWERCASE: Lowercase = Lowercase(Cow::Borrowed(""));
        &EMPTY_LOWERCASE
    }

    pub fn as_str(&'a self) -> &'a str {
        &self.0
    }

    pub fn into_cow(self) -> Cow<'a, str> {
        self.0
    }

    pub fn to_uppercase(&self) -> String {
        self.as_str().to_ascii_uppercase()
    }

    /// Like [`str::strip_prefix`]. Takes a prefix that is known to be already lowercase.
    pub fn strip_prefix_unchecked<S: Borrow<str>>(&'a self, prefix: S) -> Option<Lowercase<'a>> {
        self.0.strip_prefix(prefix.borrow()).map(|s| Self(Cow::Borrowed(s)))
    }

    /// Like [`str::strip_suffix`]. Takes a suffix that is known to be already lowercase.
    pub fn strip_suffix_unchecked<S: Borrow<str>>(&'a self, suffix: S) -> Option<Lowercase<'a>> {
        self.0.strip_suffix(suffix.borrow()).map(|s| Self(Cow::Borrowed(s)))
    }

    #[allow(dead_code)]
    pub fn contains_unchecked<S: Borrow<str>>(&self, infix: S) -> bool {
        self.0.contains(infix.borrow())
    }

    #[cfg(any(feature = "vic3", feature = "imperator"))]
    pub fn rmatch_indices_unchecked(&self, separator: char) -> RMatchIndices<char> {
        self.0.rmatch_indices(separator)
    }

    #[cfg(any(feature = "vic3", feature = "imperator"))]
    pub fn slice<R: 'a + SliceIndex<str, Output = str>>(&'a self, range: R) -> Self {
        Lowercase::new_unchecked(&self.0[range])
    }
}

impl Display for Lowercase<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl<'a> Borrow<Cow<'a, str>> for Lowercase<'a> {
    fn borrow(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl Borrow<str> for Lowercase<'_> {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Default for Lowercase<'static> {
    fn default() -> Lowercase<'static> {
        Lowercase::new_unchecked("")
    }
}

impl PartialEq<str> for Lowercase<'_> {
    fn eq(&self, s: &str) -> bool {
        self.as_str() == s
    }
}

impl PartialEq<&str> for Lowercase<'_> {
    fn eq(&self, s: &&str) -> bool {
        self.as_str() == *s
    }
}

impl PartialEq<String> for &Lowercase<'_> {
    fn eq(&self, s: &String) -> bool {
        self.as_str() == s
    }
}
