use fnv::FnvHashMap;
use std::borrow::Borrow;

use crate::errorkey::ErrorKey;
use crate::errors::{warn2, warn3};
use crate::scopes::Scopes;
use crate::token::Token;

/// The `ScopeContext` represents what we know about the scopes leading to the `Block`
/// currently being validated.
#[derive(Clone, Debug)]
pub struct ScopeContext {
    prev: Option<Box<ScopeHistory>>,

    /// Normally, `this` starts as a `ScopeEntry::Rootref`, but there are cases where the
    /// relationship to root is not known.
    this: ScopeEntry,

    /// root is always a `ScopeEntry::Scope`
    root: ScopeEntry,

    /// Names of named scopes; the values are indices into the `named` vector.
    /// Names should only be added, never removed, and indices should stay consistent.
    /// This is because the indices are also used by `ScopeEntry::Named` values throughout this `ScopeContext`.
    /// `names` and `list_names` occupy separate namespaces, but index into the same `named` array.
    names: FnvHashMap<String, usize>,
    list_names: FnvHashMap<String, usize>,

    /// Named scope values are `ScopeEntry::Scope` or `ScopeEntry::Named` or `ScopeEntry::Rootref`.
    /// Invariant: there are no cycles in the array via `ScopeEntry::Named` entries.
    /// The `bool` value indicates whether this entry is a list.
    named: Vec<ScopeEntry>,

    is_builder: bool,
    is_unrooted: bool,
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
    /// The backref number is 0 for 'this', 1 for 'prev'
    Backref(usize),

    /// A Rootref is for when the current scope is made with `root`. Most of the time,
    /// we also start with `this` being a Rootref.
    Rootref,

    /// `Token` is the token that's the reason why we think the `Scopes` value is what it is.
    /// It's usually the token that was the cause of the latest narrowing.
    Scope(Scopes, Token),

    /// The current scope takes its value from a named scope. The `usize` is an index into the `ScopeContext::named` vector.
    Named(usize, Token),
}

impl ScopeContext {
    pub fn new_root<T: Borrow<Token>>(root: Scopes, token: T) -> Self {
        ScopeContext {
            prev: None,
            this: ScopeEntry::Rootref,
            root: ScopeEntry::Scope(root, token.borrow().clone()),
            names: FnvHashMap::default(),
            list_names: FnvHashMap::default(),
            named: Vec::new(),
            is_builder: false,
            is_unrooted: false,
        }
    }

    pub fn new_unrooted<T: Borrow<Token>>(this: Scopes, token: T) -> Self {
        ScopeContext {
            prev: Some(Box::new(ScopeHistory {
                prev: None,
                this: ScopeEntry::Scope(this, token.borrow().clone()),
            })),
            this: ScopeEntry::Scope(this, token.borrow().clone()),
            root: ScopeEntry::Scope(Scopes::all(), token.borrow().clone()),
            names: FnvHashMap::default(),
            list_names: FnvHashMap::default(),
            named: Vec::new(),
            is_builder: false,
            is_unrooted: true,
        }
    }

    pub fn change_root<T: Borrow<Token>>(&mut self, root: Scopes, token: T) {
        self.root = ScopeEntry::Scope(root, token.borrow().clone());
    }

    pub fn define_name(&mut self, name: &str, scopes: Scopes, token: Token) {
        if let Some(&idx) = self.names.get(name) {
            self._break_chains_to(idx);
            self.named[idx] = ScopeEntry::Scope(scopes, token);
        } else {
            self.names.insert(name.to_string(), self.named.len());
            self.named.push(ScopeEntry::Scope(scopes, token));
        }
    }

    pub fn save_current_scope(&mut self, name: &str) {
        if let Some(&idx) = self.names.get(name) {
            self._break_chains_to(idx);
            let entry = self._resolve_backrefs();
            // Check against `scope:foo = { save_scope_as = foo }`
            if let ScopeEntry::Named(i, _) = entry {
                if *i == idx {
                    // Leave the scope as its original value
                    return;
                }
            }
            self.named[idx] = entry.clone();
        } else {
            self.names.insert(name.to_string(), self.named.len());
            self.named.push(self._resolve_backrefs().clone());
        }
    }

    pub fn define_or_expect_list(&mut self, name: &Token) {
        if let Some(&idx) = self.list_names.get(name.as_str()) {
            let (s, t) = self._resolve_named(idx);
            self.expect(s, &t.clone());
        } else {
            self.list_names.insert(name.to_string(), self.named.len());
            self.named.push(self._resolve_backrefs().clone());
        }
    }

    pub fn expect_list(&mut self, name: &Token) {
        if let Some(&idx) = self.list_names.get(name.as_str()) {
            let (s, t) = self._resolve_named(idx);
            self.expect(s, &t.clone());
        } else {
            // only with strict scope checking
            // let msg = format!("unknown list");
            //warn(name, ErrorKey::UnknownList, &msg);
        }
    }

    /// Cut `idx` out of any `ScopeEntry::Named` chains
    fn _break_chains_to(&mut self, idx: usize) {
        for i in 0..self.named.len() {
            if i == idx {
                continue;
            }
            if let ScopeEntry::Named(ni, _) = self.named[i] {
                if ni == idx {
                    self.named[i] = self.named[idx].clone();
                }
            }
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
        self.is_builder = true;
    }

    pub fn finalize_builder(&mut self) {
        self.is_builder = false;
    }

    pub fn close(&mut self) {
        let mut prev = self.prev.take().unwrap();
        self.this = prev.this.clone();
        self.prev = prev.prev.take();
        self.is_builder = false;
    }

    pub fn replace(&mut self, scopes: Scopes, token: Token) {
        self.this = ScopeEntry::Scope(scopes, token);
    }

    pub fn replace_root(&mut self) {
        self.this = ScopeEntry::Rootref;
    }

    pub fn replace_prev(&mut self) {
        self.this = ScopeEntry::Backref(1);
    }

    pub fn replace_this(&mut self) {
        self.this = ScopeEntry::Backref(0);
    }

    pub fn replace_named_scope(&mut self, name: &str, token: &Token) {
        self.this = ScopeEntry::Named(self._named_index(name, token), token.clone());
    }

    pub fn replace_list_entry(&mut self, name: &str, token: &Token) {
        self.this = ScopeEntry::Named(self._named_list_index(name, token), token.clone());
    }

    fn _named_index(&mut self, name: &str, token: &Token) -> usize {
        if let Some(&idx) = self.names.get(name) {
            idx
        } else {
            let idx = self.named.len();
            self.names.insert(name.to_string(), idx);
            self.named
                .push(ScopeEntry::Scope(Scopes::all(), token.clone()));
            idx
        }
    }

    fn _named_list_index(&mut self, name: &str, token: &Token) -> usize {
        if let Some(&idx) = self.list_names.get(name) {
            idx
        } else {
            let idx = self.named.len();
            self.list_names.insert(name.to_string(), idx);
            self.named
                .push(ScopeEntry::Scope(Scopes::all(), token.clone()));
            idx
        }
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

    fn _resolve_root(&self) -> (Scopes, &Token) {
        match self.root {
            ScopeEntry::Scope(s, ref t) => (s, t),
            _ => unreachable!(),
        }
    }

    // Resolve a `ScopeEntry` until it's either a `ScopeEntry::Scope` or a `ScopeEntry::Rootref`
    fn _resolve_named(&self, idx: usize) -> (Scopes, &Token) {
        match self.named[idx] {
            ScopeEntry::Scope(s, ref t) => (s, t),
            ScopeEntry::Rootref => self._resolve_root(),
            ScopeEntry::Named(idx, _) => self._resolve_named(idx),
            ScopeEntry::Backref(_) => unreachable!(),
        }
    }

    fn _resolve_backrefs(&self) -> &ScopeEntry {
        match self.this {
            ScopeEntry::Backref(r) => self._resolve_backrefs_inner(r),
            _ => &self.this,
        }
    }

    fn _resolve_backrefs_inner(&self, mut back: usize) -> &ScopeEntry {
        let mut ptr = &self.prev;
        loop {
            if let Some(entry) = ptr {
                if back == 0 {
                    match entry.this {
                        ScopeEntry::Backref(r) => back = r + 1,
                        _ => return &entry.this,
                    }
                }
                ptr = &entry.prev;
                back -= 1;
            } else {
                // We went further back up the scope chain than we know about.
                // TODO: do something sensible here
                return &self.root;
            }
        }
    }

    pub fn scopes_token(&self) -> (Scopes, &Token) {
        match self.this {
            ScopeEntry::Scope(s, ref t) => (s, t),
            ScopeEntry::Backref(r) => self._scopes_token(r),
            ScopeEntry::Rootref => self._resolve_root(),
            ScopeEntry::Named(idx, _) => self._resolve_named(idx),
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
                        ScopeEntry::Rootref => return self._resolve_root(),
                        ScopeEntry::Named(idx, _) => return self._resolve_named(idx),
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

    fn _expect_check(e: &mut ScopeEntry, scopes: Scopes, token: &Token) {
        match e {
            ScopeEntry::Scope(ref mut s, ref mut t) => {
                if s.intersects(scopes) {
                    // if s is narrowed by the scopes info, remember its token
                    if (*s & scopes) != *s {
                        *s &= scopes;
                        *t = token.clone();
                    }
                } else {
                    let msg = format!("`{token}` is for {scopes} but scope seems to be {s}");
                    let msg2 = format!("scope was deduced from `{t}` here");
                    warn2(token, ErrorKey::Scopes, &msg, &*t, &msg2);
                }
            }
            _ => unreachable!(),
        }
    }

    fn _expect_check3(
        e: &mut ScopeEntry,
        scopes: Scopes,
        token: &Token,
        key: &Token,
        report: &str,
    ) {
        match e {
            ScopeEntry::Scope(ref mut s, ref mut t) => {
                if s.intersects(scopes) {
                    // if s is narrowed by the scopes info, remember its token
                    if (*s & scopes) != *s {
                        *s &= scopes;
                        *t = token.clone();
                    }
                } else {
                    let msg = format!(
                        "`{key}` expects {report} to be {scopes} but {report} seems to be {s}"
                    );
                    let msg2 = format!("expected {report} was deduced from `{token}` here");
                    let msg3 = format!("actual {report} was deduced from `{t}` here");
                    warn3(key, ErrorKey::Scopes, &msg, token, &msg2, &*t, &msg3);
                }
            }
            _ => unreachable!(),
        }
    }

    // TODO: find a way to report the chain of Named tokens to the user
    fn _expect_named(&mut self, mut idx: usize, scopes: Scopes, token: &Token) {
        loop {
            match self.named[idx] {
                ScopeEntry::Scope(_, _) => {
                    Self::_expect_check(&mut self.named[idx], scopes, token);
                    return;
                }
                ScopeEntry::Rootref => {
                    Self::_expect_check(&mut self.root, scopes, token);
                    return;
                }
                ScopeEntry::Named(i, _) => idx = i,
                ScopeEntry::Backref(_) => unreachable!(),
            }
        }
    }

    fn _expect_named3(
        &mut self,
        mut idx: usize,
        scopes: Scopes,
        token: &Token,
        key: &Token,
        report: &str,
    ) {
        loop {
            match self.named[idx] {
                ScopeEntry::Scope(_, _) => {
                    Self::_expect_check3(&mut self.named[idx], scopes, token, key, report);
                    return;
                }
                ScopeEntry::Rootref => {
                    Self::_expect_check3(&mut self.root, scopes, token, key, report);
                    return;
                }
                ScopeEntry::Named(i, _) => idx = i,
                ScopeEntry::Backref(_) => unreachable!(),
            }
        }
    }

    fn _expect(&mut self, scopes: Scopes, token: &Token, mut back: usize) {
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
                        ScopeEntry::Named(idx, _) => {
                            self._expect_named(idx, scopes, token);
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

    fn _expect3(
        &mut self,
        scopes: Scopes,
        token: &Token,
        mut back: usize,
        key: &Token,
        report: &str,
    ) {
        // go N steps back and check/modify that scope. If the scope is itself
        // a back reference, go that much further back.

        let mut ptr = &mut self.prev;
        loop {
            if let Some(ref mut entry) = *ptr {
                if back == 0 {
                    match entry.this {
                        ScopeEntry::Scope(_, _) => {
                            Self::_expect_check3(&mut entry.this, scopes, token, key, report);
                            return;
                        }
                        ScopeEntry::Backref(r) => back = r + 1,
                        ScopeEntry::Rootref => {
                            Self::_expect_check3(&mut self.root, scopes, token, key, report);
                            return;
                        }
                        ScopeEntry::Named(idx, ref _t) => {
                            self._expect_named3(idx, scopes, token, key, report);
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

    pub fn expect(&mut self, scopes: Scopes, token: &Token) {
        // The None scope is special, it means the scope isn't used or inspected
        if scopes == Scopes::None {
            return;
        }
        match self.this {
            ScopeEntry::Scope(_, _) => Self::_expect_check(&mut self.this, scopes, token),
            ScopeEntry::Backref(r) => self._expect(scopes, token, r),
            ScopeEntry::Rootref => Self::_expect_check(&mut self.root, scopes, token),
            ScopeEntry::Named(idx, ref _t) => self._expect_named(idx, scopes, token),
        }
    }

    pub fn expect3(&mut self, scopes: Scopes, token: &Token, key: &Token) {
        // The None scope is special, it means the scope isn't used or inspected
        if scopes == Scopes::None {
            return;
        }
        match self.this {
            ScopeEntry::Scope(_, _) => {
                Self::_expect_check3(&mut self.this, scopes, token, key, "scope");
            }
            ScopeEntry::Backref(r) => self._expect3(scopes, token, r, key, "scope"),
            ScopeEntry::Rootref => {
                Self::_expect_check3(&mut self.root, scopes, token, key, "scope");
            }
            ScopeEntry::Named(idx, ref _t) => self._expect_named3(idx, scopes, token, key, "scope"),
        }
    }

    pub fn expect_compatibility(&mut self, other: &ScopeContext, key: &Token) {
        // Compare restrictions on `root`
        match other.root {
            ScopeEntry::Scope(scopes, ref token) => {
                Self::_expect_check3(&mut self.root, scopes, token, key, "root");
            }
            _ => unreachable!(),
        }

        // Compare restrictions on `this`
        let (scopes, token) = other.scopes_token();
        self.expect3(scopes, token, key);

        // Compare restrictions on `prev`
        // In practice, we don't need to go further than one `prev` back, because of how expect_compatibility is used.
        let (scopes, token) = other._scopes_token(0);
        self._expect3(scopes, token, usize::from(self.is_builder), key, "prev");

        // Compare restrictions on named scopes
        for (name, &oidx) in &other.names {
            // Don't do anything if our scope doesn't have that name; this will change when we get strict name checking.
            if self.names.contains_key(name) {
                let (s, t) = other._resolve_named(oidx);
                let idx = self._named_index(name, key);
                let report = format!("scope:{name}");
                self._expect_named3(idx, s, t, key, &report);
            }
        }
    }
}

impl Drop for ScopeContext {
    fn drop(&mut self) {
        if self.is_unrooted {
            assert!(
                self.prev.take().unwrap().prev.is_none(),
                "unrooted scope chain not properly unwound"
            );
        } else {
            assert!(self.prev.is_none(), "scope chain not properly unwound");
        }
    }
}
