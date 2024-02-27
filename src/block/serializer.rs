//! Support code for serializing and deserializing [`Block`] recursively.

use capnp::message::{Builder, DEFAULT_READER_OPTIONS};
use capnp::serialize_packed::{read_message, write_message};
use fnv::FnvHashMap;

use crate::block::Block;
use crate::capnp::pdxfile_capnp::{pdx_file, token};
use crate::fileset::{FileEntry, FileKind};
use crate::pathtable::PathTableIndex;
use crate::token::{leak, Loc, Token};

pub struct Serializer {
    /// A stringtable holding all unique `Token` strings, so that the `Token`s can be encoded as
    /// just an offset and length into this table.
    table: String,

    /// All unique `Token` strings that have been put in the table, paired with their offsets.
    // TODO: check if directly searching the table for substrings might be faster than maintaining
    // this hashmap.
    tokens_seen: FnvHashMap<&'static str, usize>,
}

impl Serializer {
    pub fn new() -> Self {
        Self { table: String::new(), tokens_seen: FnvHashMap::default() }
    }

    pub fn serialize(&mut self, topblock: &Block) -> Vec<u8> {
        let mut message = Builder::new_default();
        let mut top = message.init_root::<pdx_file::Builder>();
        let mut block = top.reborrow().init_block();
        topblock.serialize(self, &mut block);

        top.set_string_table(&self.table);
        // TODO: figure out how to get the total message size from a Builder,
        // and preallocate a suitable Vec capacity.
        let mut output = Vec::<u8>::default();
        // `write_message` cannot fail here, because the output is to a non-failling stream
        _ = write_message(&mut output, &message);
        output
    }

    pub fn add_token(&mut self, builder: &mut token::Builder, token: &Token) {
        let offset;
        // TODO: use `get_or_insert` once that API is stable in Rust
        if let Some(&stored_offset) = self.tokens_seen.get(token.as_str()) {
            offset = stored_offset;
        } else {
            offset = self.table.len();
            self.table.push_str(token.as_str());
            self.tokens_seen.insert(token.as_str(), offset);
        }

        // This will panic if offsets or lengths grow beyond 2GB.
        // Panicking is better than silently putting the wrong value.
        builder.set_offset(u32::try_from(offset).unwrap());
        builder.set_length(u32::try_from(token.as_str().len()).unwrap());
        // From the loc only line and column are needed, because the file information is kept
        // centralized for the whole file, and the link index is not used in raw file parse
        // results (which are the only blocks we serialize).
        builder.set_line(token.loc.line);
        builder.set_column(token.loc.column);
    }
}

pub struct Deserializer {
    /// The string table that was created during serialization.
    table: &'static str,

    /// The index to use for creating [`Loc`]s in the deserialized [`Block`].
    index: PathTableIndex,

    /// The filekind to use for creating [`Loc`]s in the deserialized [`Block`].
    kind: FileKind,
}

impl Deserializer {
    pub fn deserialize(entry: &FileEntry, serialized: &[u8]) -> Option<Block> {
        let message = read_message(serialized, DEFAULT_READER_OPTIONS).ok()?;
        let reader = message.get_root::<pdx_file::Reader>().ok()?;
        let index = entry.path_idx()?;
        let kind = entry.kind();
        let table = leak(reader.get_string_table().ok()?.to_str().ok()?.to_string());
        let deserializer = Self { table, index, kind };
        Block::deserialize(&deserializer, reader.get_block().ok()?)
    }

    pub fn get_token(&self, reader: token::Reader) -> Token {
        let offset = reader.get_offset() as usize;
        let length = reader.get_length() as usize;
        let slice = &self.table[offset..offset + length];
        Token::new(slice, self.loc_at(reader.get_line(), reader.get_column()))
    }

    pub fn loc(&self) -> Loc {
        (self.index, self.kind).into()
    }

    pub fn loc_at(&self, line: u32, column: u16) -> Loc {
        let mut loc = self.loc();
        loc.line = line;
        loc.column = column;
        loc
    }
}
