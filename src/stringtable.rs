use std::cell::RefCell;
use std::mem::{forget, ManuallyDrop};
use std::str::from_utf8_unchecked;

/// A global table for strings that are never deallocated, which allows &'static str references to them.
/// Using this makes `Token` cheaper to clone because it doesn't have to clone the allocated strings.
/// It also reduces memory usage by making `Token` slightly smaller and not cloning the string
/// along with the `Token`.
pub struct StringTable {}

// A round number of kilobytes minus some overhead
const CHUNK_SIZE: usize = 8 * 1024 - 48;
// The longest string that will be packed into a chunk. Longer strings get their own memory.
const MAX_PACK_SIZE: usize = 1024;

struct Chunk {
    allocated: usize,
    space: [u8; CHUNK_SIZE],
}

thread_local!(static CHUNK: RefCell<ManuallyDrop<Box<Chunk>>> = RefCell::new(ManuallyDrop::new(Box::new(Chunk::new()))));

impl StringTable {
    /// Stores a string in the string table and returns a static reference to it.
    pub fn store(s: &str) -> &'static str {
        if s.len() > MAX_PACK_SIZE {
            Self::store_leak(s)
        } else {
            Self::store_pack(s)
        }
    }

    fn store_leak(s: &str) -> &'static str {
        let leak_s = s.to_owned();

        // Go through a raw pointer in order to confuse the borrow checker.
        // This allows us to return a &'static str.
        let ptr: *const str = &*leak_s;
        // Leak the String. The String object itself is probably on the stack and will be deallocated,
        // but its u8 array with the string contents will stay on the heap.
        forget(leak_s);
        unsafe { &*ptr }
    }

    fn store_pack(s: &str) -> &'static str {
        CHUNK.with(|cell| {
            if cell.borrow().space_left() < s.len() {
                // Because the Box is ManuallyDrop and we never manually drop it, its memory will stay allocated
                // even when we replace it with a new one.
                cell.replace(ManuallyDrop::new(Box::new(Chunk::new())));
            }
            cell.borrow_mut().allocate(s)
        })
    }
}

impl Chunk {
    // TODO: use MaybeUninit to avoid having to zero the `space` array.
    fn new() -> Self {
        Self { allocated: 0, space: [0; CHUNK_SIZE] }
    }

    fn space_left(&self) -> usize {
        CHUNK_SIZE - self.allocated
    }

    fn allocate(&mut self, s: &str) -> &'static str {
        let slice = &mut self.space[self.allocated..self.allocated + s.len()];
        (*slice).copy_from_slice(s.as_bytes());
        self.allocated += s.len();
        assert!(self.allocated <= CHUNK_SIZE);
        // Safe because we just copied these bytes from a str, so they must still be utf8
        let packed_s = unsafe { from_utf8_unchecked(slice) };
        // Go through a raw pointer in order to confuse the borrow checker.
        // This allows us to return a &'static str.
        let ptr: *const str = packed_s;
        unsafe { &*ptr }
    }
}
