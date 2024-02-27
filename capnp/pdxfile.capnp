@0xddaed8cb121ed121;

# Store the parse results of a pdxscript file as a combination of string table
# (for the tokens) and a recursive Block.

using Rust = import "rust.capnp";
$Rust.parentModule("capnp");

struct PdxFile {
    # The parse result of a pdxscript file. It is stored as a combination of
    # string table (for the tokens) and a recursive Block.

    fileKind @2 :FileKind;
    pathName @3 :Text; # The pathname relative to the root of the fileKind

    stringTable @0 :Text;
    # Tokens in the following Block are (offset, len) pointers into this table.

    block @1 :Block;
}

struct FileKind {
    union {
        internal @0 :Void;
        clausewitz @1 :Void;
        jomini @2 :Void;
        vanilla @3 :Void;
        loadedMod @4 :UInt8;
        mod @5 :Void;
    }
}

struct Block {
    items @0 :List(BlockItem);
    tag @1 :Token; # optional
    line @2 :UInt32;
    column @3 :UInt16;
    source @4 :List(MacroComponent); # optional
}

struct BlockItem {
    union {
        value @0 :Token;
        block @1 :Block;
        field @2 :Field;
    }
}

struct Field {
    token @0 :Token;
    comparator @1 :Comparator;
    bv :union {
        value @2: Token;
        block @3: Block;
    }
}

enum Comparator {
    equalsSingle @0; # =
    equalsDouble @1; # ==
    equalsQuestion @2; # ?=
    notEquals @3; # !=
    lessThan @4; # <
    greaterThan @5; # >
    atMost @6; # <=
    atLeast @7; # >=
}

struct Token {
    offset @0 :Int32; # offset into PdxFile.stringTable
    length @1 :Int32;
    line @2 :UInt32;
    column @3 :UInt16;
}

struct MacroComponent {
    union {
        source @0 :Token;
        localValue @1 :Token;
        macro @2 :Token;
    }
}
