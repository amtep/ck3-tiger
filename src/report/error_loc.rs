use crate::block::{Block, BV};
use crate::fileset::FileEntry;
use crate::token::{Loc, Token};

// This trait lets the error functions accept a variety of things as the error locator.
pub trait ErrorLoc {
    fn into_loc(self) -> Loc;
}

impl ErrorLoc for BV {
    fn into_loc(self) -> Loc {
        match self {
            BV::Value(t) => t.into_loc(),
            BV::Block(s) => s.into_loc(),
        }
    }
}

impl ErrorLoc for &BV {
    fn into_loc(self) -> Loc {
        match self {
            BV::Value(t) => t.into_loc(),
            BV::Block(s) => s.into_loc(),
        }
    }
}

impl ErrorLoc for FileEntry {
    fn into_loc(self) -> Loc {
        Loc::for_entry(&self)
    }
}

impl ErrorLoc for &FileEntry {
    fn into_loc(self) -> Loc {
        Loc::for_entry(self)
    }
}

impl ErrorLoc for Loc {
    fn into_loc(self) -> Loc {
        self
    }
}

impl ErrorLoc for &Loc {
    fn into_loc(self) -> Loc {
        self.clone()
    }
}

impl ErrorLoc for Token {
    fn into_loc(self) -> Loc {
        self.loc
    }
}

impl ErrorLoc for &Token {
    fn into_loc(self) -> Loc {
        self.loc.clone()
    }
}

impl ErrorLoc for Block {
    fn into_loc(self) -> Loc {
        self.loc
    }
}

impl ErrorLoc for &Block {
    fn into_loc(self) -> Loc {
        self.loc.clone()
    }
}
