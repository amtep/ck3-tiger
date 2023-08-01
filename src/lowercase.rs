//! Type-safety wrapper for strings that must be lowercase

use std::borrow::{Borrow, Cow};

/// Wraps a string (either owned or `&str`) and guarantees that it's lowercase.
///
/// This allows interfaces that expect lowercase strings to declare this expectation in their
/// argument type, so that the caller can choose how to fulfill it.
///
/// The lowercase string is a [`Cow<str>`] internally, so it can be either owned or borrowed.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Lowercase<'a>(Cow<'a, str>);

impl<'a> Lowercase<'a> {
    /// Take a string and return the lowercased version.
    pub fn new(s: &'a str) -> Self {
        // Avoid allocating if it's not necessary
        if s.chars().all(char::is_lowercase) {
            Lowercase(Cow::Borrowed(s))
        } else {
            Lowercase(Cow::Owned(s.to_lowercase()))
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

    pub fn to_uppercase(&self) -> String {
        self.as_str().to_uppercase()
    }
}

impl<'a> Borrow<Cow<'a, str>> for Lowercase<'a> {
    fn borrow(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl<'a> Borrow<str> for Lowercase<'a> {
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

impl PartialEq<String> for &Lowercase<'_> {
    fn eq(&self, s: &String) -> bool {
        self.as_str() == s
    }
}
