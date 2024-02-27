//! This is the interface to the parse cache.
//!
//! Serialized parse results (represented as byte strings) can be stored and loaded here,
//! with cache validity checking based on properties of the source file.
//!
//! The cache does not have an index. The easiest way to remove old files is to just delete the
//! whole cache and let it repopulate.

use std::fs::{create_dir_all, metadata, read, write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use capnp::message::{Builder, DEFAULT_READER_OPTIONS};
use capnp::serialize::{read_message_from_flat_slice, write_message_to_words};
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use xxhash_rust::xxh3::xxh3_128;

use crate::capnp::fileheader_capnp::file_header;
pub use crate::capnp::fileheader_capnp::ParserType;
use crate::fileset::FileEntry;

const MAGIC: &[u8] = b"TIGER\n\x1a\x00";

static PROJECT_DIRS: Lazy<ProjectDirs> = Lazy::new(|| {
    ProjectDirs::from("io.github", "amtep", "tiger-lib").expect("No valid home directory found")
});

impl ParserType {
    fn to_filename(self) -> &'static str {
        match self {
            ParserType::PdxCk3 => "pdxck3",
            ParserType::PdxVic3 => "pdxvic3",
            ParserType::PdxImperator => "pdximperator",
        }
    }
}

/// Return the full path for a cache file based on `entry`.
/// The subdirectories leading up to it do not necessarily exist.
fn cache_pathname(entry: &FileEntry, parser: ParserType) -> Option<PathBuf> {
    let hash = format!("{:0x}", xxh3_128(entry.fullpath().to_string_lossy().as_bytes()));
    // Put the cache files into subdirectories based on the start of the hash,
    // in order to distribute them over smaller directories instead of one giant one.
    // This helps with filesystem performance.
    let subdir = format!("tiger.{}", &hash[0..2]);
    let filename = format!("tiger.{}.{hash}", parser.to_filename());
    Some(PROJECT_DIRS.cache_dir().join(subdir).join(filename))
}

/// Return the cached parse result for this `entry`, if it's found in the cache and is still valid.
/// For efficiency, the result is returned as a `Vec` and the offset into the `Vec` where the encoded parse result starts.
pub fn cache_lookup(
    entry: &FileEntry,
    parser: ParserType,
    version: u32,
) -> Option<(Vec<u8>, usize)> {
    let pathname = cache_pathname(entry, parser)?;
    let bytes = read(pathname).ok()?;
    if !bytes.starts_with(MAGIC) {
        return None;
    }

    let attr = metadata(entry.fullpath()).ok()?;
    let size = attr.len();
    let modtime = attr.modified().ok()?.duration_since(SystemTime::UNIX_EPOCH).ok()?;

    let mut slice = &bytes[MAGIC.len()..];
    let reader = read_message_from_flat_slice(&mut slice, DEFAULT_READER_OPTIONS).ok()?;
    let file_header = reader.get_root::<file_header::Reader>().ok()?;
    let stored_pathname = file_header.get_pathname().ok()?.to_str().ok()?;
    let stored_modtime = Duration::new(
        file_header.get_last_modified_seconds(),
        file_header.get_last_modified_nanoseconds(),
    );
    let stored_parser = file_header.get_parser().ok()?;
    if stored_pathname != entry.fullpath().to_string_lossy()
        || file_header.get_size() != size
        || stored_modtime != modtime
        || stored_parser != parser
        || file_header.get_parser_version() != version
    {
        return None;
    }

    // The capnp reader will have updated the slice to point just beyond the decoded file header.
    let offset = slice.as_ptr() as usize - bytes.as_ptr() as usize;

    Some((bytes, offset))
}

// An version of cache_put that returns an Option, just to enable `?` usage
fn cache_put_inner(
    entry: &FileEntry,
    parser: ParserType,
    version: u32,
    parse_result: &[u8],
) -> Option<()> {
    let attr = metadata(entry.fullpath()).ok()?;
    let size = attr.len();
    let modtime = attr.modified().ok()?.duration_since(SystemTime::UNIX_EPOCH).ok()?;

    let pathname = cache_pathname(entry, parser)?;

    let mut message = Builder::new_default();
    let mut file_header = message.init_root::<file_header::Builder>();
    file_header.set_pathname(entry.fullpath().to_string_lossy());
    file_header.set_size(size);
    file_header.set_last_modified_seconds(modtime.as_secs());
    file_header.set_last_modified_nanoseconds(modtime.subsec_nanos());
    file_header.set_parser(parser);
    file_header.set_parser_version(version);

    // The unwrap is safe because we generated the pathname with a filename.
    create_dir_all(pathname.parent().unwrap()).ok()?;

    // Add the bytes in memory instead of doing three separate write() calls.
    // It means extra copying but it's still faster than multiple syscalls.
    let encoded_file_header = write_message_to_words(&message);
    let mut bytes =
        Vec::with_capacity(MAGIC.len() + encoded_file_header.len() + parse_result.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&encoded_file_header);
    bytes.extend_from_slice(parse_result);
    write(pathname, bytes).ok()
}

/// Store the parse result for `entry` in the cache.
pub fn cache_put(entry: &FileEntry, parser: ParserType, version: u32, parse_result: &[u8]) {
    _ = cache_put_inner(entry, parser, version, parse_result);
}
