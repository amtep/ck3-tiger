use crate::errorkey::ErrorKey;
use crate::errors::{warn, warn2};
use crate::scopes::Scopes;
use crate::token::Token;

/// The `ScopeContext` represents what we know about the scopes leading to the `Block`
/// currently being validated.
#[derive(Clone, Debug)]
pub struct ScopeContext {
    prev: Option<Box<ScopeHistory>>,

    // Normally, `this` starts as a `ScopeEntry::Rootref`, but there are cases where the
    // relationship to root is not known.
    this: ScopeEntry,

    // root is always a ScopeEntry::Scope
    root: ScopeEntry,
}

#[derive(Clone, Debug)]
struct ScopeHistory {
    prev: Option<Box<ScopeHistory>>,
    this: ScopeEntry,
}

#[derive(Clone, Debug)]
enum ScopeEntry {
    /// Backref is for when the current scope is made with `prev` or `this`.
    /// It counts as a scope in the chain, for purposes of `prev` and such, but any updates
    /// to it (such as narrowing of scope types) need to be propagated back to the
    /// real origin of that scope.
    ///
    /// The backref number is 0 for 'this', 1 for 'prev', +1 for each extra 'prev'.
    Backref(usize),

    /// A Rootref is for when the current scope is made with `root`. Most of the time,
    /// we also start with `this` being a Rootref.
    Rootref,

    /// `Token` is the token that's the reason why we think the `Scopes` value is what it is.
    /// It's usually the token that was the cause of the latest narrowing.
    Scope(Scopes, Token),
}

impl ScopeContext {
    pub fn new_root(root: Scopes, token: Token) -> Self {
        ScopeContext {
            prev: None,
            this: ScopeEntry::Rootref,
            root: ScopeEntry::Scope(root, token),
        }
    }

    pub fn new_unrooted(this: Scopes, token: Token) -> Self {
        ScopeContext {
            prev: None,
            this: ScopeEntry::Scope(this, token.clone()),
            root: ScopeEntry::Scope(Scopes::all(), token),
        }
    }

    pub fn open_scope(&mut self, scopes: Scopes, token: Token) {
        self.prev = Some(Box::new(ScopeHistory {
            prev: self.prev.take(),
            this: self.this.clone(),
        }));
        self.this = ScopeEntry::Scope(scopes, token);
    }

    pub fn open_builder(&mut self) {
        self.prev = Some(Box::new(ScopeHistory {
            prev: self.prev.take(),
            this: self.this.clone(),
        }));
        self.this = ScopeEntry::Backref(0);
    }

    pub fn close(&mut self) {
        let mut prev = self.prev.take().unwrap();
        self.this = prev.this.clone();
        self.prev = prev.prev.take();
    }

    pub fn replace(&mut self, scopes: Scopes, token: Token) {
        self.this = ScopeEntry::Scope(scopes, token);
    }

    pub fn replace_root(&mut self) {
        self.this = ScopeEntry::Rootref;
    }

    pub fn replace_prev(&mut self, token: &Token) {
        match self.this {
            ScopeEntry::Scope(_, _) => self.this = ScopeEntry::Backref(1),
            ScopeEntry::Backref(r) => self.this = ScopeEntry::Backref(r + 1),
            ScopeEntry::Rootref => {
                warn(token, ErrorKey::Scopes, "trying to take prev of root");
            }
        }
    }

    pub fn replace_this(&mut self) {
        self.this = ScopeEntry::Backref(0);
    }

    pub fn can_be(&self, scopes: Scopes) -> bool {
        self.scopes().intersects(scopes)
    }

    pub fn must_be(&self, scopes: Scopes) -> bool {
        self.scopes() == scopes
    }

    pub fn scopes(&self) -> Scopes {
        self.scopes_token().0
    }

    fn scopes_token(&self) -> (Scopes, &Token) {
        match self.this {
            ScopeEntry::Scope(s, ref t) => (s, t),
            ScopeEntry::Backref(r) => self._scopes_token(r),
            ScopeEntry::Rootref => match self.root {
                ScopeEntry::Scope(s, ref t) => (s, t),
                _ => unreachable!(),
            },
        }
    }

    fn _scopes_token(&self, mut back: usize) -> (Scopes, &Token) {
        let mut ptr = &self.prev;
        loop {
            if let Some(entry) = ptr {
                if back == 0 {
                    match entry.this {
                        ScopeEntry::Scope(s, ref t) => return (s, t),
                        ScopeEntry::Backref(r) => back = r + 1,
                        ScopeEntry::Rootref => match self.root {
                            ScopeEntry::Scope(s, ref t) => return (s, t),
                            _ => unreachable!(),
                        },
                    }
                }
                ptr = &entry.prev;
                back -= 1;
            } else {
                // We went further back up the scope chain than we know about.
                // Currently we just bail, and return an "any scope" value with
                // an arbitrary token.
                match self.root {
                    ScopeEntry::Scope(_, ref t) => return (Scopes::all(), t),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn _expect_check(e: &mut ScopeEntry, scopes: Scopes, token: Token) {
        match e {
            ScopeEntry::Scope(ref mut s, ref mut t) => {
                if s.intersects(scopes) {
                    // if s is narrowed by the scopes info, remember its token
                    if (*s & scopes) != *s {
                        *s &= scopes;
                        *t = token;
                    }
                } else {
                    let msg = format!("`{}` is for {} but scope seems to be {}", token, scopes, s);
                    let msg2 = format!("scope was deduced from `{}` here", t);
                    warn2(&token, ErrorKey::Scopes, &msg, t.clone(), &msg2);
                    // Suppress future warnings about the same problem
                    *s |= scopes;
                    *t = token;
                }
            }
            _ => unreachable!(),
        }
    }

    fn _expect(&mut self, scopes: Scopes, token: Token, mut back: usize) {
        // go N steps back and check/modify that scope. If the scope is itself
        // a back reference, go that much further back.

        let mut ptr = &mut self.prev;
        loop {
            if let Some(ref mut entry) = *ptr {
                if back == 0 {
                    match entry.this {
                        ScopeEntry::Scope(_, _) => {
                            Self::_expect_check(&mut entry.this, scopes, token);
                            return;
                        }
                        ScopeEntry::Backref(r) => back = r + 1,
                        ScopeEntry::Rootref => {
                            Self::_expect_check(&mut self.root, scopes, token);
                            return;
                        }
                    }
                }
                ptr = &mut entry.prev;
                back -= 1;
            } else {
                // TODO: warning of some kind?
                return;
            }
        }
    }

    pub fn expect(&mut self, scopes: Scopes, token: Token) {
        // The None scope is special, it means the scope isn't used or inspected
        if scopes == Scopes::None {
            return;
        }
        match self.this {
            ScopeEntry::Scope(_, _) => Self::_expect_check(&mut self.this, scopes, token),
            ScopeEntry::Backref(r) => self._expect(scopes, token, r),
            ScopeEntry::Rootref => Self::_expect_check(&mut self.root, scopes, token),
        }
    }

    pub fn expect_compatibility(&mut self, other: &ScopeContext) {
        // Compare restrictions on `root`
        match other.root {
            ScopeEntry::Scope(scopes, ref token) => {
                Self::_expect_check(&mut self.root, scopes, token.clone())
            }
            _ => unreachable!(),
        }

        // Compare restrictions on `this`
        let (scopes, token) = other.scopes_token();
        self.expect(scopes, token.clone());

        // TODO: walk back up the chain and compare all prev scopes too
        // Describing the results in error messages will be hard though.
    }
}

impl Drop for ScopeContext {
    fn drop(&mut self) {
        if self.prev.is_some() {
            panic!("scope chain not properly unwound");
        }
    }
}
