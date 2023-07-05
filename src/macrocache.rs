use std::cell::RefCell;

use fnv::FnvHashMap;

use crate::token::{Loc, Token};
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct MacroKey {
    loc: Loc,
    // the loc of the call site
    args: Vec<(String, String)>,
    // lexically sorted macro arguments
    tooltipped: Tooltipped,
    negated: bool, // only for triggers
}

impl MacroKey {
    pub fn new(
        mut loc: Loc,
        args: &[(&str, Token)],
        tooltipped: Tooltipped,
        negated: bool,
    ) -> Self {
        loc.link = None;
        let mut args: Vec<(String, String)> = args
            .iter()
            .map(|(parm, arg)| ((*parm).to_string(), arg.to_string()))
            .collect();
        args.sort();
        Self {
            loc,
            args,
            tooltipped,
            negated,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MacroCache<T> {
    cache: RefCell<FnvHashMap<MacroKey, T>>,
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
        if let Some(x) = self.cache.borrow().get(&key) {
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
        self.cache.borrow_mut().insert(key, value);
    }
}

impl<T> Default for MacroCache<T> {
    fn default() -> Self {
        MacroCache {
            cache: RefCell::new(FnvHashMap::default()),
        }
    }
}
