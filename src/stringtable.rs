use std::mem::forget;

/// A global table for strings that are never deallocated, which allows &'static str references to them.
/// Using this makes `Token` cheaper to clone because it doesn't have to clone the allocated strings.
/// It also reduces memory usage by making `Token` slightly smaller and not cloning the string
/// along with the `Token`.
pub struct StringTable {}

impl StringTable {
    /// Stores a string in the string table and returns a static reference to it.
    pub fn store(s: &str) -> &'static str {
        // This first version just creates a String and leaks it.
        // TODO: pack the strings into chunks.
        let leak_s = s.to_owned();

        // Go through a raw pointer in order to confuse the borrow checker.
        // The borrow checker complains about returning a str with lifetime 'static, but we promise not
        // to move or deallocate the chunks. So it's safe.
        let ptr: *const str = &*leak_s;
        forget(leak_s);
        unsafe { &*ptr }
    }
}
