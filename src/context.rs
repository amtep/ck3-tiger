//! [`ScopeContext`] tracks our knowledge of the scope types used in script and validates its consistency.

use std::borrow::Cow;

use fnv::FnvHashMap;

use crate::game::Game;
use crate::helpers::stringify_choices;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;

/// When reporting an unknown scope, list alternative scope names if there are not more than this.
const MAX_SCOPE_NAME_LIST: usize = 6;

/// The `ScopeContext` represents what we know about the scopes leading to the `Block`
/// currently being validated.
#[derive(Clone, Debug)]
pub struct ScopeContext {
    /// `prev` is a chain of all the known previous scopes.
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
    named: Vec<ScopeEntry>,

    /// Same indices as `named`, is a token iff the named scope is expected to be set on entry to the current scope context.
    /// Invariant: `named` and `is_input` are the same length.
    is_input: Vec<Option<Token>>,

    /// Is this scope level a level in progress? `is_builder` is used when evaluating scope chains
    /// like `root.liege.primary_title`. It affects the handling of `prev`, because the builder
    /// scope is not a real scope level yet.
    is_builder: bool,

    /// Was this `ScopeContext` created as an unrooted context? Unrooted means we do not know
    /// whether `this` and `root` are the same at the start. Unrooted scopes start with an extra
    /// `prev` level, so they need to be cleaned up differently.
    is_unrooted: bool,

    /// Is this scope context one where all the named scopes are (or should be) known in advance?
    /// If `strict_scopes` is false, then the `ScopeContext` will assume any name might be a valid
    /// scope name that we just don't know about yet.
    strict_scopes: bool,

    /// A special flag for scope contexts that are known to be wrong. It's used for the
    /// `scope_override` config file feature. If `no_warn` is set then this ScopeContext will not
    /// emit any reports.
    no_warn: bool,
}

#[derive(Clone, Debug)]
/// One previous scope level in a chain of previous scopes.
///
/// Used for handling `prev`, and also used when closing a scope: the most recent
/// `ScopeHistory` in the chain gets popped back as the current scope.
struct ScopeHistory {
    prev: Option<Box<ScopeHistory>>,
    this: ScopeEntry,
}

#[derive(Clone, Debug)]
/// `ScopeEntry` is a description of what we know of a scope's type and its connection to other
/// scopes.
///
/// It is used both to look up a scope's type, and to propagate knowledge about that type backward
/// to the scope's source. For example if `this` is a Rootref, and we find out that `this` is a
/// `Character`, then `root` must be a `Character` too.
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
    Scope(Scopes, Reason),

    /// The current scope takes its value from a named scope. The `usize` is an index into the `ScopeContext::named` vector.
    Named(usize, Reason),
}

/// This enum records the reason why we think a certain scope has the type it does.
/// It is used for error reporting.
///
/// TODO: make a `ReasonRef` that contains an `&Token`, and a `Borrow` impl for it.
/// This will avoid some cloning.
#[derive(Clone, Debug)]
pub enum Reason {
    /// The reason can be explained by pointing at some token
    Token(Token),
    /// The scope type was deduced from a named scope's name; the `Token` points at that name in
    /// the script.
    Name(Token),
    /// The scope was supplied by the game engine. The `Token` points at a key explaining this, for
    /// example the key of an `Item` or the field key of a trigger or effect in an item.
    Builtin(Token),
}

impl Reason {
    pub fn token(&self) -> &Token {
        match self {
            Reason::Token(t) | Reason::Name(t) | Reason::Builtin(t) => t,
        }
    }

    // TODO: change this to Display ?
    pub fn msg(&self) -> Cow<str> {
        match self {
            Reason::Token(t) => Cow::Owned(format!("deduced from `{t}` here")),
            Reason::Name(_) => Cow::Borrowed("deduced from the scope's name"),
            Reason::Builtin(_) => Cow::Borrowed("supplied by the game engine"),
        }
    }
}

impl ScopeEntry {
    fn deduce<T: Into<Token>>(token: T) -> ScopeEntry {
        let token = token.into();
        if let Some(scopes) = scope_type_from_name(token.as_str()) {
            ScopeEntry::Scope(scopes, Reason::Name(token))
        } else {
            ScopeEntry::Scope(Scopes::all(), Reason::Token(token))
        }
    }
}

impl ScopeContext {
    /// Make a new `ScopeContext`, with `this` and `root` the same, and `root` of the given scope
    /// types. `token` is used when reporting errors about the use of `root`.
    pub fn new<T: Into<Token>>(root: Scopes, token: T) -> Self {
        ScopeContext {
            prev: None,
            this: ScopeEntry::Rootref,
            root: ScopeEntry::Scope(root, Reason::Builtin(token.into())),
            names: FnvHashMap::default(),
            list_names: FnvHashMap::default(),
            named: Vec::new(),
            is_input: Vec::new(),
            is_builder: false,
            is_unrooted: false,
            strict_scopes: true,
            no_warn: false,
        }
    }

    /// Make a new `ScopeContext`, with `this` and `root` unconnected, and `this` of the given scope
    /// types. `token` is used when reporting errors about the use of `this`, `root`, or `prev`.
    ///
    /// This function is useful for the scope contexts created for scripted effects, scripted
    /// triggers, and script values. In those, it's not known what the caller's `root` is.
    pub fn new_unrooted<T: Into<Token>>(this: Scopes, token: T) -> Self {
        let token = token.into();
        ScopeContext {
            prev: Some(Box::new(ScopeHistory {
                prev: None,
                this: ScopeEntry::Scope(Scopes::all(), Reason::Token(token.clone())),
            })),
            this: ScopeEntry::Scope(this, Reason::Token(token.clone())),
            root: ScopeEntry::Scope(Scopes::all(), Reason::Token(token)),
            names: FnvHashMap::default(),
            list_names: FnvHashMap::default(),
            named: Vec::new(),
            is_input: Vec::new(),
            is_builder: false,
            is_unrooted: true,
            strict_scopes: true,
            no_warn: false,
        }
    }

    /// Declare whether all the named scopes in this scope context are known. Default is true.
    ///
    /// Set this to false in for example events, which start with the scopes defined by their
    /// triggering context.
    ///
    /// Having strict scopes set to true makes the `ScopeContext` emit errors when encountering
    /// unknown scope names.
    pub fn set_strict_scopes(&mut self, strict: bool) {
        self.strict_scopes = strict;
    }

    /// Return whether this `ScopeContext` has strict scopes set to true.
    /// See [`Self::set_strict_scopes`].
    pub fn is_strict(&self) -> bool {
        self.strict_scopes
    }

    /// Set whether this `ScopeContext` should emit reports at all. `no_warn` defaults to false.
    ///
    /// It's used for scope contexts that are known to be wrong, related to the `scope_override` config file feature.
    pub fn set_no_warn(&mut self, no_warn: bool) {
        self.no_warn = no_warn;
    }

    /// Change the scope type and related token of `root` for this `ScopeContext`.
    ///
    /// This function is mainly used in the setup of a `ScopeContext` before using it.
    /// It's a bit of a hack and shouldn't be used.
    /// TODO: get rid of this.
    #[cfg(feature = "ck3")] // happens not to be used by vic3
    pub fn change_root<T: Into<Token>>(&mut self, root: Scopes, token: T) {
        self.root = ScopeEntry::Scope(root, Reason::Builtin(token.into()));
    }

    #[doc(hidden)]
    fn define_name_internal(&mut self, name: &str, scopes: Scopes, reason: Reason) {
        if let Some(&idx) = self.names.get(name) {
            self.break_chains_to(idx);
            self.named[idx] = ScopeEntry::Scope(scopes, reason);
        } else {
            self.names.insert(name.to_string(), self.named.len());
            self.named.push(ScopeEntry::Scope(scopes, reason));
            self.is_input.push(None);
        }
    }

    /// Declare that this `ScopeContext` contains a named scope of the given name and type,
    /// supplied by the game engine.
    ///
    /// The associated `token` will be used in error reports related to this named scope.
    pub fn define_name<T: Into<Token>>(&mut self, name: &str, scopes: Scopes, token: T) {
        self.define_name_internal(name, scopes, Reason::Builtin(token.into()));
    }

    /// Declare that this `ScopeContext` contains a named scope of the given name and type,
    /// *not* supplied by the game engine but deduced from script.
    ///
    /// The associated `token` will be used in error reports related to this named scope.
    /// The token should reflect why we think the named scope has the scope type it has.
    pub fn define_name_token<T: Into<Token>>(&mut self, name: &str, scopes: Scopes, token: T) {
        self.define_name_internal(name, scopes, Reason::Token(token.into()));
    }

    /// Look up a named scope and return its scope types if it's known.
    ///
    /// Callers should probably check [`Self::is_strict()`] as well.
    pub fn is_name_defined(&mut self, name: &str) -> Option<Scopes> {
        if let Some(&idx) = self.names.get(name) {
            #[allow(clippy::match_on_vec_items)] // invariant guarantees no panic
            Some(match self.named[idx] {
                ScopeEntry::Scope(s, _) => s,
                ScopeEntry::Backref(_) => unreachable!(),
                ScopeEntry::Rootref => self.resolve_root().0,
                ScopeEntry::Named(idx, _) => self.resolve_named(idx).0,
            })
        } else {
            None
        }
    }

    /// This is called when the script does `exists = scope:name`.
    ///
    /// It records `name` as "known", but with no scope type information, and records that the
    /// caller is expected to provide this scope.
    ///
    /// The `ScopeContext` is not smart enough to track optionally existing scopes. It assumes
    /// that if you do `exists` on a scope, then from that point on it exists. Improving this would
    /// be a big project.
    pub fn exists_scope<T: Into<Token>>(&mut self, name: &str, token: T) {
        if !self.names.contains_key(name) {
            let idx = self.named.len();
            self.names.insert(name.to_string(), idx);
            self.named.push(ScopeEntry::deduce(token));
            self.is_input.push(None);
        }
    }

    #[doc(hidden)]
    fn define_list_internal(&mut self, name: &str, scopes: Scopes, reason: Reason) {
        if let Some(&idx) = self.list_names.get(name) {
            self.break_chains_to(idx);
            self.named[idx] = ScopeEntry::Scope(scopes, reason);
        } else {
            self.list_names.insert(name.to_string(), self.named.len());
            self.named.push(ScopeEntry::Scope(scopes, reason));
            self.is_input.push(None);
        }
    }

    /// Declare that this `ScopeContext` contains a list of the given name and type,
    /// supplied by the game engine.
    ///
    /// The associated `token` will be used in error reports related to this list.
    ///
    /// Lists and named scopes exist in different namespaces, but under the hood
    /// `ScopeContext` treats them the same. This means that lists are expected to
    /// contain items of a single scope type, which sometimes leads to false positives.
    pub fn define_list<T: Into<Token>>(&mut self, name: &str, scopes: Scopes, token: T) {
        self.define_list_internal(name, scopes, Reason::Builtin(token.into()));
    }

    /// This is like [`Self::define_name()`], but `scope:name` is declared equal to the current `this`.
    pub fn save_current_scope(&mut self, name: &str) {
        if let Some(&idx) = self.names.get(name) {
            self.break_chains_to(idx);
            let entry = self.resolve_backrefs();
            // Guard against `scope:foo = { save_scope_as = foo }`
            if let ScopeEntry::Named(i, _) = entry {
                if *i == idx {
                    // Leave the scope as its original value
                    return;
                }
            }
            self.named[idx] = entry.clone();
        } else {
            self.names.insert(name.to_string(), self.named.len());
            self.named.push(self.resolve_backrefs().clone());
            self.is_input.push(None);
        }
    }

    /// If list `name` exists, narrow its scope type down to `this`, otherwise define it
    /// as having the same scope type as `this`.
    // TODO: I don't think this is doing the right thing for most callers.
    pub fn define_or_expect_list(&mut self, name: &Token) {
        if let Some(&idx) = self.list_names.get(name.as_str()) {
            let (s, reason) = self.resolve_named(idx);
            let reason = reason.clone(); // TODO: remove need to clone
            self.expect(s, &reason);
            // It often happens that an iterator does is_in_list before add_to_list,
            // and in those cases we want the add_to_list to take precedence: conclude that the
            // list is being built here, and isn't an input list.
            self.is_input[idx] = None;
        } else {
            self.list_names.insert(name.to_string(), self.named.len());
            self.named.push(self.resolve_backrefs().clone());
            self.is_input.push(None);
        }
    }

    /// Expect list `name` to be known and (with strict scopes) warn if it isn't.
    /// Narrow the type of `this` down to the list's type.
    pub fn expect_list(&mut self, name: &Token) {
        if let Some(&idx) = self.list_names.get(name.as_str()) {
            let (s, reason) = self.resolve_named(idx);
            let reason = reason.clone(); // TODO: remove need to clone
            self.expect3(s, &reason, name);
        } else if self.strict_scopes {
            let msg = "unknown list";
            err(ErrorKey::UnknownList).weak().msg(msg).loc(name).push();
        }
    }

    /// Cut `idx` out of any [`ScopeEntry::Named`] chains. This avoids infinite loops.
    #[doc(hidden)]
    fn break_chains_to(&mut self, idx: usize) {
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

    /// Open a new scope level of `scopes` scope type. Record `token` as the reason for this type.
    ///
    /// This is mostly used by iterators.
    /// `prev` will refer to the previous scope level.
    pub fn open_scope(&mut self, scopes: Scopes, token: Token) {
        self.prev =
            Some(Box::new(ScopeHistory { prev: self.prev.take(), this: self.this.clone() }));
        self.this = ScopeEntry::Scope(scopes, Reason::Token(token));
    }

    /// Open a new, temporary scope level. Initially it will have its `this` the same as the
    /// previous level's `this`.
    ///
    /// The purpose is to handle scope chains like `root.liege.primary_title`. Call the `replace_`
    /// functions to update the value of `this`, and at the end either confirm the new scope level
    /// with [`Self::finalize_builder()`] or discard it with [`Self::close()`].
    pub fn open_builder(&mut self) {
        self.prev =
            Some(Box::new(ScopeHistory { prev: self.prev.take(), this: self.this.clone() }));
        self.this = ScopeEntry::Backref(0);
        self.is_builder = true;
    }

    /// Declare that the temporary scope level opened with [`Self::open_builder()`] is a real scope level.
    pub fn finalize_builder(&mut self) {
        self.is_builder = false;
    }

    /// Exit a scope level and return to the previous level.
    pub fn close(&mut self) {
        let mut prev = self.prev.take().unwrap();
        self.this = prev.this.clone();
        self.prev = prev.prev.take();
        self.is_builder = false;
    }

    /// Replace the `this` in a temporary scope level with the given `scopes` type and record
    /// `token` as the reason for this type.
    ///
    /// This is used when a scope chain starts with something absolute like `faith:catholic`.
    pub fn replace(&mut self, scopes: Scopes, token: Token) {
        self.this = ScopeEntry::Scope(scopes, Reason::Token(token));
    }

    /// Replace the `this` in a temporary scope level with a reference to `root`.
    pub fn replace_root(&mut self) {
        self.this = ScopeEntry::Rootref;
    }

    /// Replace the `this` in a temporary scope level with a reference to the previous scope level.
    pub fn replace_prev(&mut self) {
        if Game::is_imperator() {
            // Allow `prev.prev` for imperator.
            match self.this {
                ScopeEntry::Backref(r) => self.this = ScopeEntry::Backref(r + 1),
                _ => self.this = ScopeEntry::Backref(1),
            }
        } else {
            self.this = ScopeEntry::Backref(1);
        }
    }

    /// Replace the `this` in a temporary scope level with a reference to the real level below it.
    ///
    /// This is usually a no-op, used when scope chains start with `this`. If a scope chain has
    /// `this` in the middle of the chain (which itself will trigger a warning) then it resets the
    /// temporary scope level to the way it started.
    pub fn replace_this(&mut self) {
        self.this = ScopeEntry::Backref(0);
    }

    /// Replace the `this` in a temporary scope level with a reference to the named scope `name`.
    ///
    /// This is used when a scope chain starts with `scope:name`. The `token` is expected to be the
    /// `scope:name` token.
    pub fn replace_named_scope(&mut self, name: &str, token: Token) {
        self.this = ScopeEntry::Named(self.named_index(name, &token), Reason::Token(token));
    }

    /// Replace the `this` in a temporary scope level with a reference to the scope type of the
    /// list `name`.
    ///
    /// This is used in list iterators. The `token` is expected to be the token for the name of the
    /// list.
    pub fn replace_list_entry(&mut self, name: &str, token: &Token) {
        self.this =
            ScopeEntry::Named(self.named_list_index(name, token), Reason::Token(token.clone()));
    }

    /// Get the internal index of named scope `name`, either its existing index or a newly created
    /// one.
    ///
    /// If a new index has to be created, and `strict_scopes` is on, then a warning will be emitted.
    #[doc(hidden)]
    fn named_index(&mut self, name: &str, token: &Token) -> usize {
        if let Some(&idx) = self.names.get(name) {
            idx
        } else {
            let idx = self.named.len();
            self.named.push(ScopeEntry::deduce(token));
            if self.strict_scopes {
                if !self.no_warn {
                    let msg = format!("scope:{name} might not be available here");
                    let mut builder = err(ErrorKey::StrictScopes).weak().msg(msg);
                    if self.names.len() <= MAX_SCOPE_NAME_LIST && !self.names.is_empty() {
                        let mut names: Vec<_> = self.names.keys().map(String::as_str).collect();
                        names.sort_unstable();
                        let info = format!("available names are {}", stringify_choices(&names));
                        builder = builder.info(info);
                    }
                    builder.loc(token).push();
                }
                // Don't treat it as an input scope, because we already warned about it
                self.is_input.push(None);
            } else {
                self.is_input.push(Some(token.clone()));
            }
            // do this after the warnings above, so that it's not listed as available
            self.names.insert(name.to_string(), idx);
            idx
        }
    }

    /// Same as [`Self::named_index()`], but for lists. No warning is emitted if a new list is created.
    #[doc(hidden)]
    fn named_list_index(&mut self, name: &str, token: &Token) -> usize {
        if let Some(&idx) = self.list_names.get(name) {
            idx
        } else {
            let idx = self.named.len();
            self.list_names.insert(name.to_string(), idx);
            self.named.push(ScopeEntry::Scope(Scopes::all(), Reason::Token(token.clone())));
            self.is_input.push(Some(token.clone()));
            idx
        }
    }

    /// Return true iff it's possible that `this` is the same type as one of the `scopes` types.
    pub fn can_be(&self, scopes: Scopes) -> bool {
        self.scopes().intersects(scopes)
    }

    /// Return true iff `this` is known to be one of the types of `scopes`
    pub fn must_be(&self, scopes: Scopes) -> bool {
        scopes.contains(self.scopes())
    }

    /// Return the possible scope types of this scope level.
    // TODO: maybe specialize this function for performance?
    pub fn scopes(&self) -> Scopes {
        self.scopes_reason().0
    }

    /// Return the possible scope types of `root`, and the reason why we think it has those types
    #[doc(hidden)]
    fn resolve_root(&self) -> (Scopes, &Reason) {
        match self.root {
            ScopeEntry::Scope(s, ref reason) => (s, reason),
            _ => unreachable!(),
        }
    }

    /// Return the possible scope types of a named scope or list, and the reason why we think it
    /// has those types.
    ///
    /// The `idx` must be an index from the `names` or `list_names` vectors.
    #[doc(hidden)]
    fn resolve_named(&self, idx: usize) -> (Scopes, &Reason) {
        #[allow(clippy::match_on_vec_items)]
        match self.named[idx] {
            ScopeEntry::Scope(s, ref reason) => (s, reason),
            ScopeEntry::Rootref => self.resolve_root(),
            ScopeEntry::Named(idx, _) => self.resolve_named(idx),
            ScopeEntry::Backref(_) => unreachable!(),
        }
    }

    /// Search through the scope levels to find out what `this` actually refers to.
    ///
    /// The returned `ScopeEntry` will not be a `ScopeEntry::Backref`.
    #[doc(hidden)]
    fn resolve_backrefs(&self) -> &ScopeEntry {
        match self.this {
            ScopeEntry::Backref(r) => self.resolve_backrefs_inner(r),
            _ => &self.this,
        }
    }

    #[doc(hidden)]
    fn resolve_backrefs_inner(&self, mut back: usize) -> &ScopeEntry {
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

    /// Return the possible scope types for the current scope layer, together with the reason why
    /// we think that.
    pub fn scopes_reason(&self) -> (Scopes, &Reason) {
        match self.this {
            ScopeEntry::Scope(s, ref reason) => (s, reason),
            ScopeEntry::Backref(r) => self.scopes_reason_internal(r),
            ScopeEntry::Rootref => self.resolve_root(),
            ScopeEntry::Named(idx, _) => self.resolve_named(idx),
        }
    }

    #[doc(hidden)]
    fn scopes_reason_internal(&self, mut back: usize) -> (Scopes, &Reason) {
        let mut ptr = &self.prev;
        loop {
            if let Some(entry) = ptr {
                if back == 0 {
                    match entry.this {
                        ScopeEntry::Scope(s, ref reason) => return (s, reason),
                        ScopeEntry::Backref(r) => back = r + 1,
                        ScopeEntry::Rootref => return self.resolve_root(),
                        ScopeEntry::Named(idx, _) => return self.resolve_named(idx),
                    }
                }
                ptr = &entry.prev;
                back -= 1;
            } else {
                // We went further back up the scope chain than we know about.
                // Currently we just bail, and return an "any scope" value with
                // an arbitrary token.
                match self.root {
                    ScopeEntry::Scope(_, ref reason) => return (Scopes::all(), reason),
                    _ => unreachable!(),
                }
            }
        }
    }

    #[doc(hidden)]
    fn expect_check(e: &mut ScopeEntry, scopes: Scopes, reason: &Reason) {
        match e {
            ScopeEntry::Scope(ref mut s, ref mut r) => {
                if s.intersects(scopes) {
                    // if s is narrowed by the scopes info, remember why
                    if (*s & scopes) != *s {
                        *s &= scopes;
                        *r = reason.clone();
                    }
                } else {
                    let token = reason.token();
                    let msg = format!("`{token}` is for {scopes} but scope seems to be {s}");
                    let msg2 = format!("scope was {}", r.msg());
                    warn(ErrorKey::Scopes).msg(msg).loc(token).loc_msg(r.token(), msg2).push();
                }
            }
            _ => unreachable!(),
        }
    }

    #[doc(hidden)]
    fn expect_check3(
        e: &mut ScopeEntry,
        scopes: Scopes,
        reason: &Reason,
        key: &Token,
        report: &str,
    ) {
        match e {
            ScopeEntry::Scope(ref mut s, ref mut r) => {
                if s.intersects(scopes) {
                    // if s is narrowed by the scopes info, remember its token
                    if (*s & scopes) != *s {
                        *s &= scopes;
                        *r = reason.clone();
                    }
                } else {
                    let msg = format!(
                        "`{key}` expects {report} to be {scopes} but {report} seems to be {s}"
                    );
                    let msg2 = format!("expected {report} was {}", reason.msg());
                    let msg3 = format!("actual {report} was {}", r.msg());
                    warn(ErrorKey::Scopes)
                        .msg(msg)
                        .loc(key)
                        .loc_msg(reason.token(), msg2)
                        .loc_msg(r.token(), msg3)
                        .push();
                }
            }
            _ => unreachable!(),
        }
    }

    // TODO: find a way to report the chain of Named tokens to the user
    #[doc(hidden)]
    fn expect_named(&mut self, mut idx: usize, scopes: Scopes, reason: &Reason) {
        loop {
            #[allow(clippy::match_on_vec_items)]
            match self.named[idx] {
                ScopeEntry::Scope(_, _) => {
                    Self::expect_check(&mut self.named[idx], scopes, reason);
                    return;
                }
                ScopeEntry::Rootref => {
                    Self::expect_check(&mut self.root, scopes, reason);
                    return;
                }
                ScopeEntry::Named(i, _) => idx = i,
                ScopeEntry::Backref(_) => unreachable!(),
            }
        }
    }

    #[doc(hidden)]
    fn expect_named3(
        &mut self,
        mut idx: usize,
        scopes: Scopes,
        reason: &Reason,
        key: &Token,
        report: &str,
    ) {
        loop {
            #[allow(clippy::match_on_vec_items)]
            match self.named[idx] {
                ScopeEntry::Scope(_, _) => {
                    Self::expect_check3(&mut self.named[idx], scopes, reason, key, report);
                    return;
                }
                ScopeEntry::Rootref => {
                    Self::expect_check3(&mut self.root, scopes, reason, key, report);
                    return;
                }
                ScopeEntry::Named(i, _) => idx = i,
                ScopeEntry::Backref(_) => unreachable!(),
            }
        }
    }

    #[doc(hidden)]
    fn expect_internal(&mut self, scopes: Scopes, reason: &Reason, mut back: usize) {
        // go N steps back and check/modify that scope. If the scope is itself
        // a back reference, go that much further back.

        let mut ptr = &mut self.prev;
        loop {
            if let Some(ref mut entry) = *ptr {
                if back == 0 {
                    match entry.this {
                        ScopeEntry::Scope(_, _) => {
                            Self::expect_check(&mut entry.this, scopes, reason);
                            return;
                        }
                        ScopeEntry::Backref(r) => back = r + 1,
                        ScopeEntry::Rootref => {
                            Self::expect_check(&mut self.root, scopes, reason);
                            return;
                        }
                        ScopeEntry::Named(idx, _) => {
                            self.expect_named(idx, scopes, reason);
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

    #[doc(hidden)]
    fn expect3_internal(
        &mut self,
        scopes: Scopes,
        reason: &Reason,
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
                            Self::expect_check3(&mut entry.this, scopes, reason, key, report);
                            return;
                        }
                        ScopeEntry::Backref(r) => back = r + 1,
                        ScopeEntry::Rootref => {
                            Self::expect_check3(&mut self.root, scopes, reason, key, report);
                            return;
                        }
                        ScopeEntry::Named(idx, ref _t) => {
                            self.expect_named3(idx, scopes, reason, key, report);
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

    /// Record that the `this` in the current scope level is one of the scope types `scopes`, and
    /// if this is new information, record `token` as the reason we think that.
    /// Emit an error if what we already know about `this` is incompatible with `scopes`.
    pub fn expect(&mut self, scopes: Scopes, reason: &Reason) {
        // The None scope is special, it means the scope isn't used or inspected
        if self.no_warn || scopes == Scopes::None {
            return;
        }
        match self.this {
            ScopeEntry::Scope(_, _) => Self::expect_check(&mut self.this, scopes, reason),
            ScopeEntry::Backref(r) => self.expect_internal(scopes, reason, r),
            ScopeEntry::Rootref => Self::expect_check(&mut self.root, scopes, reason),
            ScopeEntry::Named(idx, ref _t) => self.expect_named(idx, scopes, reason),
        }
    }

    /// Like [`Self::expect()`], but the error emitted will be located at token `key`.
    ///
    /// This function is used when the expectation of scope compatibility comes from `key`, for
    /// example when matching up a caller's scope context with a scripted effect's scope context.
    fn expect3(&mut self, scopes: Scopes, reason: &Reason, key: &Token) {
        // The None scope is special, it means the scope isn't used or inspected
        if scopes == Scopes::None {
            return;
        }
        match self.this {
            ScopeEntry::Scope(_, _) => {
                Self::expect_check3(&mut self.this, scopes, reason, key, "scope");
            }
            ScopeEntry::Backref(r) => self.expect3_internal(scopes, reason, r, key, "scope"),
            ScopeEntry::Rootref => {
                Self::expect_check3(&mut self.root, scopes, reason, key, "scope");
            }
            ScopeEntry::Named(idx, ref _t) => {
                self.expect_named3(idx, scopes, reason, key, "scope");
            }
        }
    }

    /// Compare this scope context to `other`, with `key` as the token that identifies `other`.
    ///
    /// This function examines the `root`, `this`, `prev`, and named scopes of the two scope
    /// contexts and warns about any contradictions it finds.
    ///
    /// It expects `self` to be the caller and `other` to be the callee.
    pub fn expect_compatibility(&mut self, other: &ScopeContext, key: &Token) {
        if self.no_warn {
            return;
        }
        // Compare restrictions on `root`
        match other.root {
            ScopeEntry::Scope(scopes, ref token) => {
                Self::expect_check3(&mut self.root, scopes, token, key, "root");
            }
            _ => unreachable!(),
        }

        // Compare restrictions on `this`
        let (scopes, reason) = other.scopes_reason();
        self.expect3(scopes, reason, key);

        // Compare restrictions on `prev`
        // In practice, we don't need to go further than one `prev` back, because of how expect_compatibility is used.
        let (scopes, reason) = other.scopes_reason_internal(0);
        self.expect3_internal(scopes, reason, usize::from(self.is_builder), key, "prev");

        // Compare restrictions on named scopes
        for (name, &oidx) in &other.names {
            if self.names.contains_key(name) {
                let (s, reason) = other.resolve_named(oidx);
                if other.is_input[oidx].is_some() {
                    let idx = self.named_index(name, key);
                    let report = format!("scope:{name}");
                    self.expect_named3(idx, s, reason, key, &report);
                } else {
                    // Their scopes now become our scopes.
                    self.define_name_internal(name, s, reason.clone());
                }
            } else if self.strict_scopes && other.is_input[oidx].is_some() {
                let token = other.is_input[oidx].as_ref().unwrap();
                let msg = format!("`{key}` expects scope:{name} to be set");
                let msg2 = "here";
                warn(ErrorKey::StrictScopes).msg(msg).loc(key).loc_msg(token, msg2).push();
            } else {
                // Their scopes now become our scopes.
                let (s, reason) = other.resolve_named(oidx);
                self.names.insert(name.to_string(), self.named.len());
                self.named.push(ScopeEntry::Scope(s, reason.clone()));
                self.is_input.push(other.is_input[oidx].clone());
            }
        }

        // Compare restrictions on lists
        for (name, &oidx) in &other.list_names {
            if self.list_names.contains_key(name) {
                let (s, reason) = other.resolve_named(oidx);
                if other.is_input[oidx].is_some() {
                    let idx = self.named_list_index(name, key);
                    let report = format!("list {name}");
                    self.expect_named3(idx, s, reason, key, &report);
                } else {
                    // Their lists now become our lists.
                    self.define_list_internal(name, s, reason.clone());
                }
            } else if self.strict_scopes && other.is_input[oidx].is_some() {
                let token = other.is_input[oidx].as_ref().unwrap();
                let msg = format!("`{key}` expects list {name} to exist");
                let msg2 = "here";
                warn(ErrorKey::StrictScopes).msg(msg).loc(key).loc_msg(token, msg2).push();
            } else {
                // Their lists now become our lists.
                let (s, reason) = other.resolve_named(oidx);
                self.list_names.insert(name.to_string(), self.named.len());
                self.named.push(ScopeEntry::Scope(s, reason.clone()));
                self.is_input.push(other.is_input[oidx].clone());
            }
        }
    }
}

impl Drop for ScopeContext {
    /// This `drop` function checks that every opened scope level was also closed.
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

/// Deduce a scope type from a scope's name. This leads to better error messages.
///
/// It should be limited to names that are so obvious that it's extremely unlikely that anyone
/// would use them for a different type.
fn scope_type_from_name(mut name: &str) -> Option<Scopes> {
    if let Some(real_name) = name.strip_prefix("scope:") {
        name = real_name;
    } else {
        return None;
    }

    #[cfg(feature = "ck3")]
    if Game::is_ck3() {
        return match name {
            "accolade" => Some(Scopes::Accolade),
            "accolade_type" => Some(Scopes::AccoladeType),
            "activity" => Some(Scopes::Activity),
            "actor"
            | "recipient"
            | "secondary_actor"
            | "secondary_recipient"
            | "mother"
            | "father"
            | "real_father"
            | "child"
            | "councillor"
            | "liege"
            | "courtier"
            | "guest"
            | "host" => Some(Scopes::Character),
            "army" => Some(Scopes::Army),
            "artifact" => Some(Scopes::Artifact),
            "barony" | "county" | "title" | "landed_title" => Some(Scopes::LandedTitle),
            "combat_side" => Some(Scopes::CombatSide),
            "council_task" => Some(Scopes::CouncilTask),
            "culture" => Some(Scopes::Culture),
            "faction" => Some(Scopes::Faction),
            "faith" => Some(Scopes::Faith),
            "province" => Some(Scopes::Province),
            "scheme" => Some(Scopes::Scheme),
            "struggle" => Some(Scopes::Struggle),
            "story" => Some(Scopes::StoryCycle),
            "travel_plan" => Some(Scopes::TravelPlan),
            "war" => Some(Scopes::War),
            _ => None,
        };
    }

    #[cfg(feature = "vic3")]
    if Game::is_vic3() {
        // Due to differences in state vs state_region, law vs law_type, etc, less can be deduced
        // with certainty for vic3.
        return match name {
            "admiral" | "general" | "character" => Some(Scopes::Character),
            "actor" | "country" | "enemy_country" | "initiator" | "target_country" => {
                Some(Scopes::Country)
            }
            "battle" => Some(Scopes::Battle),
            "interest_group" => Some(Scopes::InterestGroup),
            "journal_entry" => Some(Scopes::JournalEntry),
            "market" => Some(Scopes::Market),
            _ => None,
        };
    }

    #[cfg(feature = "imperator")]
    if Game::is_imperator() {
        return match name {
            "party" | "character_party" => Some(Scopes::Party),
            "employer" | "party_country" | "country" | "overlord" | "unit_owner"
            | "attacker_warleader" | "defender_warleader" | "former_overlord"
            | "target_subject" | "future_overlord" | "old_country" | "controller" | "owner"
            | "family_country" | "losing_side" | "home_country" => Some(Scopes::Country),
            "fam" | "family" => Some(Scopes::Family),
            "preferred_heir" | "deified_ruler" | "personal_loyalty" | "character"
            | "siege_controller" | "party_leader" | "next_in_family" | "ruler" | "governor"
            | "governor_or_ruler" | "commander" | "former_ruler" | "newborn" | "spouse"
            | "job_holder" | "consort" | "current_heir" | "current_ruler" | "primary_heir"
            | "secondary_heir" | "current_co_ruler" | "head_of_family" | "holding_owner"
            | "char" | "mother" | "father" => Some(Scopes::Character),
            "job" => Some(Scopes::Job),
            "legion" => Some(Scopes::Legion),
            "dominant_province_religion" | "religion" => Some(Scopes::Religion),
            "area" => Some(Scopes::Area),
            "region" => Some(Scopes::Region),
            "governorship" => Some(Scopes::Governorship),
            "country_culture" => Some(Scopes::CountryCulture),
            "location"
            | "unit_destination"
            | "unit_objective_destination"
            | "unit_location"
            | "unit_next_location"
            | "capital_scope"
            | "holy_site" => Some(Scopes::Province),
            "dominant_province_culture_group" | "culture_group" => Some(Scopes::CultureGroup),
            "dominant_province_culture" | "culture" => Some(Scopes::Culture),
            "owning_unit" => Some(Scopes::Unit),
            "deity" | "province_deity" => Some(Scopes::Deity),
            "state" => Some(Scopes::State),
            "treasure" => Some(Scopes::Treasure),
            "siege" => Some(Scopes::Siege),
            _ => None,
        };
    }

    None
}
