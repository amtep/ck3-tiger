//! [`MacroCache`] is a helper for scripted effects, triggers, and modifiers, all of which can
//! accept macro arguments and which need to be expanded for every macro call.
//!
//! The cache helps avoid needless re-expansions for arguments that have already been validated.

use std::sync::RwLock;

use fnv::FnvHashMap;

use crate::token::{Loc, Token};
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct MacroKey {
    /// the loc of the call site
    loc: Loc,
    /// lexically sorted macro arguments
    args: Vec<(String, String)>,
    tooltipped: Tooltipped,
    /// only for triggers
    negated: bool,
}

impl MacroKey {
    pub fn new(
        mut loc: Loc,
        args: &[(&str, Token)],
        tooltipped: Tooltipped,
        negated: bool,
    ) -> Self {
        loc.link = None;
        let mut args: Vec<(String, String)> =
            args.iter().map(|(parm, arg)| ((*parm).to_string(), arg.to_string())).collect();
        args.sort();
        Self { loc, args, tooltipped, negated }
    }
}

#[derive(Debug)]
pub struct MacroCache<T> {
    cache: RwLock<FnvHashMap<MacroKey, T>>,
}

impl<T> MacroCache<T> {
    pub fn perform(
        &self,
        key: &Token,
        args: &[(&str, Token)],
        tooltipped: Tooltipped,
        negated: bool,
        mut f: impl FnMut(&T),
    ) -> bool {
        // TODO: find a way to avoid all the cloning for creating a MacroKey just to look it up in the cache
        let key = MacroKey::new(key.loc.clone(), args, tooltipped, negated);
        if let Some(x) = self.cache.read().unwrap().get(&key) {
            f(x);
            true
        } else {
            false
        }
    }

    pub fn insert(
        &self,
        key: &Token,
        args: &[(&str, Token)],
        tooltipped: Tooltipped,
        negated: bool,
        value: T,
    ) {
        let key = MacroKey::new(key.loc.clone(), args, tooltipped, negated);
        self.cache.write().unwrap().insert(key, value);
    }
}

impl<T> Default for MacroCache<T> {
    fn default() -> Self {
        MacroCache { cache: RwLock::new(FnvHashMap::default()) }
    }
}
