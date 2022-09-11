use crate::errorkey::ErrorKey;
use crate::errors::{warn, warn2};
use crate::scopes::Scopes;
use crate::token::Token;

/// The `ScopeContext` represents what we know about the scopes leading to the `Block`
/// currently being validated.
#[derive(Clone, Debug)]
pub struct ScopeContext {
    /// if `prev` is `None` then this is the root.
    prev: Option<Box<ScopeContext>>,

    entry: ScopeEntry,
}

#[derive(Clone, Debug)]
enum ScopeEntry {
    /// Backref is for when the current scope is made with "root" or "prev" or "this".
    /// It counts as a scope in the chain, for purposes of "prev" and such, but any updates
    /// to it (such as narrowing of scope types) need to be propagated back to the
    /// real origin of that scope.
    ///
    /// The backref number is 1 for 'this', 2 for 'prev', +1 for each extra 'prev'.
    Backref(usize),

    /// `Token` is the token that's the reason why we think the `Scopes` value is what it is.
    /// It's usually the token that was the cause of the latest narrowing.
    Scope(Scopes, Token),
}

impl ScopeContext {
    pub fn new(root: Scopes, token: Token) -> Self {
        ScopeContext {
            prev: None,
            entry: ScopeEntry::Scope(root, token),
        }
    }

    pub fn open_scope(&mut self, scopes: Scopes, token: Token) {
        self.prev = Some(Box::new(ScopeContext {
            prev: self.prev.take(),
            entry: self.entry.clone(),
        }));
        self.entry = ScopeEntry::Scope(scopes, token);
    }

    pub fn open_builder(&mut self) {
        self.prev = Some(Box::new(ScopeContext {
            prev: self.prev.take(),
            entry: self.entry.clone(),
        }));
        self.entry = ScopeEntry::Backref(1);
    }

    pub fn close(&mut self) {
        let mut prev = self.prev.take().unwrap();
        self.entry = prev.entry.clone();
        self.prev = prev.prev.take();
    }

    pub fn replace(&mut self, scopes: Scopes, token: Token) {
        self.entry = ScopeEntry::Scope(scopes, token);
    }

    pub fn replace_root(&mut self) {
        self.entry = ScopeEntry::Backref(self.len());
    }

    pub fn replace_prev(&mut self) {
        self.entry = ScopeEntry::Backref(2);
    }

    pub fn replace_this(&mut self) {
        self.entry = ScopeEntry::Backref(1);
    }

    pub fn can_be(&self, scopes: Scopes) -> bool {
        self.scopes().intersects(scopes)
    }

    pub fn must_be(&self, scopes: Scopes) -> bool {
        self.scopes() == scopes
    }

    pub fn scopes(&self) -> Scopes {
        match self.entry {
            ScopeEntry::Scope(s, _) => s,
            ScopeEntry::Backref(r) => self._scopes(r),
        }
    }

    pub fn _scopes(&self, back: usize) -> Scopes {
        if back == 0 {
            match self.entry {
                ScopeEntry::Scope(s, _) => s,
                ScopeEntry::Backref(r) => self._scopes(r),
            }
        } else {
            match &self.prev {
                None => Scopes::None,
                Some(p) => p._scopes(back - 1),
            }
        }
    }

    pub fn expect(&mut self, scopes: Scopes, token: Token) {
        // The None scope is special, it means the scope isn't used or inspected
        if scopes == Scopes::None {
            return;
        }
        match self.entry {
            ScopeEntry::Scope(_, _) => self._expect(scopes, token, 0),
            ScopeEntry::Backref(r) => self._expect(scopes, token, r),
        }
    }

    fn _expect(&mut self, scopes: Scopes, token: Token, back: usize) {
        // recurse N steps back and check/modify that scope. If the scope is itself
        // a back reference, go that much further back.

        if back == 0 {
            match self.entry {
                ScopeEntry::Scope(ref mut s, ref mut t) => {
                    if s.intersects(scopes) {
                        *s &= scopes;
                    } else {
                        let msg =
                            format!("`{}` is for {} but scope seems to be {}", token, scopes, s);
                        let msg2 = format!("scope was deduced from `{}` here", t);
                        warn2(&token, ErrorKey::Scopes, &msg, t.clone(), &msg2);
                        // Suppress future warnings about the same problem
                        *s |= scopes;
                        *t = token;
                    }
                }
                ScopeEntry::Backref(r) => self._expect(scopes, token, r),
            }
        } else {
            match self.prev {
                None => {
                    // TODO: make sure this doesn't flood the error log.
                    warn(token, ErrorKey::Scopes, "trying to take prev of root scope");
                }
                Some(ref mut p) => p._expect(scopes, token, back - 1),
            }
        }
    }

    pub fn len(&self) -> usize {
        if let Some(prev) = &self.prev {
            1 + prev.len()
        } else {
            1
        }
    }
}

impl Drop for ScopeContext {
    fn drop(&mut self) {
        if self.prev.is_some() {
            panic!("scope chain not properly unwound");
        }
    }
}
