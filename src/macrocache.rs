use fnv::FnvHashMap;
use std::cell::RefCell;

use crate::token::{Loc, Token};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct MacroKey {
    loc: Loc,                    // the loc of the call site
    args: Vec<(String, String)>, // lexically sorted macro arguments
}

impl MacroKey {
    pub fn new(mut loc: Loc, args: &[(String, Token)]) -> Self {
        loc.link = None;
        let mut args: Vec<(String, String)> = args
            .iter()
            .map(|(parm, arg)| (parm.clone(), arg.to_string()))
            .collect();
        args.sort();
        Self { loc, args }
    }
}

#[derive(Clone, Debug)]
pub struct MacroCache<T> {
    cache: RefCell<FnvHashMap<MacroKey, T>>,
}

impl<T> MacroCache<T> {
    pub fn perform(&self, key: &Token, args: &[(String, Token)], mut f: impl FnMut(&T)) -> bool {
        let key = MacroKey::new(key.loc.clone(), args);
        if let Some(x) = self.cache.borrow().get(&key) {
            f(x);
            true
        } else {
            false
        }
    }

    pub fn insert(&self, key: &Token, args: &[(String, Token)], value: T) {
        let key = MacroKey::new(key.loc.clone(), args);
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
