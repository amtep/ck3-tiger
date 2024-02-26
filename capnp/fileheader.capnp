@0xb4bfcb7a504c79fa;

# A cache file is named after a hash of the full pathname plus the parser type.
# Then it starts with the bytes "TIGER" followed by 0x0a 0x1a 0x00
# which together form the 8-byte "magic number" for this file format.
# The rest of the file consists of two Cap'n'proto "messages", where
# the first is the struct FileHeader below and the second depends on
# the parser.

using Rust = import "rust.capnp";
$Rust.parentModule("capnp");

struct FileHeader {
    # Identifying information for a parse result cache file.
    # This part contains only the information necessary to determine whether
    # a cache file is still valid.

    pathname @0 :Text;  # Full path to the file that was parsed
    size @1 :UInt64;    # Size of the source file at time of parsing
    lastModifiedSeconds @2 :UInt64;  # Whole seconds part of mtime
    lastModifiedNanoseconds @3 :UInt32;  # Nanoseconds part of mtime
    parser @5 :ParserType;  # Which parser made this cached result
    parserVersion @4 :UInt32;
    # The parser version field is incremented every time the parser changes,
    # in order to invalidate cache files made with the previous parser.
}

enum ParserType {
    # Enumeration of the different parser types that can cache their results.
    pdxCk3 @0;
    pdxVic3 @1;
    pdxImperator @2;
}
