use std::borrow::Cow;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;
use std::sync::Arc;

use crate::parse::pdxfile::{parse_pdx_macro, LocalMacros};
use crate::token::{Loc, Token};

mod blockitem;
mod bv;
mod comparator;
mod field;
pub mod validator;

pub use crate::block::blockitem::BlockItem;
pub use crate::block::bv::BV;
pub use crate::block::comparator::{Comparator, Eq};
pub use crate::block::field::Field;

#[derive(Clone, Debug)]
pub struct Block {
    // v can contain key = value pairs as well as unadorned values.
    // The latter are inserted as None tokens and Comparator::None
    v: Vec<BlockItem>,
    pub tag: Option<Token>,
    pub loc: Loc,
    /// If the block is a top-level block and contains macro substitutions,
    /// this field will hold the original source for re-parsing.
    /// The source has already been split into a vec that alternates content
    /// with macro parameters.
    pub source: Option<(Vec<Token>, LocalMacros)>,
}

impl Block {
    pub fn new(loc: Loc) -> Self {
        Block { v: Vec::new(), tag: None, loc, source: None }
    }

    pub fn add_value(&mut self, value: BV) {
        match value {
            BV::Value(token) => self.v.push(BlockItem::Value(token)),
            BV::Block(block) => self.v.push(BlockItem::Block(block)),
        }
    }

    pub fn add_key_value(&mut self, key: Token, cmp: Comparator, value: BV) {
        self.v.push(BlockItem::Field(Field(key, cmp, value)));
    }

    pub fn add_item(&mut self, item: BlockItem) {
        self.v.push(item);
    }

    pub fn add_to_field_block(&mut self, name: &str, block: &mut Block) -> bool {
        for item in self.v.iter_mut().rev() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is(name) {
                    match bv {
                        BV::Value(_) => (),
                        BV::Block(b) => {
                            b.append(block);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn append(&mut self, other: &mut Block) {
        self.v.append(&mut other.v);
    }

    pub fn filename(&self) -> Cow<str> {
        self.loc.filename()
    }

    /// Get the value of a single `name = value` assignment
    pub fn get_field_value(&self, name: &str) -> Option<&Token> {
        for item in self.v.iter().rev() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is(name) {
                    match bv {
                        BV::Value(t) => return Some(t),
                        BV::Block(_) => (),
                    }
                }
            }
        }
        None
    }

    pub fn field_value_is(&self, name: &str, value: &str) -> bool {
        if let Some(token) = self.get_field_value(name) {
            token.is(value)
        } else {
            false
        }
    }

    pub fn get_field_bool(&self, name: &str) -> Option<bool> {
        self.get_field_value(name).map(|t| t.is("yes"))
    }

    pub fn get_field_integer(&self, name: &str) -> Option<i64> {
        self.get_field_value(name).and_then(|t| t.as_str().parse().ok())
    }

    /// Get all the values of `name = value` assignments in this block
    pub fn get_field_values(&self, name: &str) -> Vec<&Token> {
        let mut vec = Vec::new();
        for (key, token) in self.iter_assignments() {
            if key.is(name) {
                vec.push(token);
            }
        }
        vec
    }

    /// Get the block of a `name = { ... }` assignment
    pub fn get_field_block(&self, name: &str) -> Option<&Block> {
        for item in self.v.iter().rev() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is(name) {
                    match bv {
                        BV::Value(_) => (),
                        BV::Block(b) => return Some(b),
                    }
                }
            }
        }
        None
    }

    /// Get all the blocks of `name = { ... }` assignments in this block
    pub fn get_field_blocks(&self, name: &str) -> Vec<&Block> {
        let mut vec = Vec::new();
        for (key, block) in self.iter_definitions() {
            if key.is(name) {
                vec.push(block);
            }
        }
        vec
    }

    /// Get the values of a single `name = { value ... }` assignment
    pub fn get_field_list(&self, name: &str) -> Option<Vec<Token>> {
        for item in self.v.iter().rev() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is(name) {
                    match bv {
                        BV::Value(_) => (),
                        BV::Block(b) => {
                            return Some(b.iter_values().cloned().collect());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_field(&self, name: &str) -> Option<&BV> {
        for item in self.v.iter().rev() {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is(name) {
                    return Some(bv);
                }
            }
        }
        None
    }

    pub fn get_key(&self, name: &str) -> Option<&Token> {
        for item in self.v.iter().rev() {
            if let BlockItem::Field(Field(key, _, _)) = item {
                if key.is(name) {
                    return Some(key);
                }
            }
        }
        None
    }

    pub fn get_keys(&self, name: &str) -> Vec<&Token> {
        let mut vec = Vec::new();
        for Field(key, _, _) in self.iter_fields() {
            if key.is(name) {
                vec.push(key);
            }
        }
        vec
    }

    pub fn has_key(&self, name: &str) -> bool {
        self.get_key(name).is_some()
    }

    pub fn count_keys(&self, name: &str) -> usize {
        let mut count = 0;
        for Field(key, _, _) in self.iter_fields() {
            if key.is(name) {
                count += 1;
            }
        }
        count
    }

    pub fn iter_items(&self) -> std::slice::Iter<BlockItem> {
        self.v.iter()
    }

    pub fn drain(&mut self) -> std::vec::Drain<BlockItem> {
        self.v.drain(..)
    }

    pub fn iter_fields(&self) -> IterFields {
        IterFields { iter: self.v.iter(), warn: false }
    }

    pub fn iter_fields_warn(&self) -> IterFields {
        IterFields { iter: self.v.iter(), warn: true }
    }

    /// "Assignments" are fields that have `key = value`.
    pub fn iter_assignments(&self) -> IterAssignments {
        IterAssignments { iter: self.v.iter(), warn: false }
    }

    /// "Assignments" are fields that have `key = value`.
    /// `warn` means emit reports for every `BlockItem` that's not an assignment.
    pub fn iter_assignments_warn(&self) -> IterAssignments {
        IterAssignments { iter: self.v.iter(), warn: true }
    }

    /// "Definitions" are fields that have `key = { block }`.
    pub fn iter_definitions(&self) -> IterDefinitions {
        IterDefinitions { iter: self.v.iter(), warn: false }
    }

    /// "Definitions" are fields that have `key = { block }`.
    /// `warn` means emit reports for every `BlockItem` that's not a definition.
    pub fn iter_definitions_warn(&self) -> IterDefinitions {
        IterDefinitions { iter: self.v.iter(), warn: true }
    }

    /// "Definitions" are fields that have `key = { block }`.
    pub fn iter_assignments_and_definitions(&self) -> IterAssignmentsAndDefinitions {
        IterAssignmentsAndDefinitions { iter: self.v.iter(), warn: false }
    }

    /// "Assignments" are fields that have `key = value`.
    /// "Definitions" are fields that have `key = { block }`.
    /// `warn` means emit reports for every `BlockItem` that's not an assignment or definition
    pub fn iter_assignments_and_definitions_warn(&self) -> IterAssignmentsAndDefinitions {
        IterAssignmentsAndDefinitions { iter: self.v.iter(), warn: true }
    }

    /// "Assignments" are fields that have `key = value`.
    /// "Definitions" are fields that have key = { block }
    /// `warn` means emit reports for every `BlockItem` that's not an assignment or definition
    pub fn drain_definitions_warn(&mut self) -> DrainDefinitions {
        DrainDefinitions { iter: self.v.drain(..) }
    }

    pub fn iter_values(&self) -> IterValues {
        IterValues { iter: self.v.iter(), warn: false }
    }

    pub fn iter_values_warn(&self) -> IterValues {
        IterValues { iter: self.v.iter(), warn: true }
    }

    pub fn iter_blocks(&self) -> IterBlocks {
        IterBlocks { iter: self.v.iter(), warn: false }
    }

    pub fn iter_blocks_warn(&self) -> IterBlocks {
        IterBlocks { iter: self.v.iter(), warn: true }
    }

    pub fn get_field_at_date(&self, name: &str, date: Date) -> Option<&BV> {
        let mut found_date: Option<Date> = None;
        let mut found: Option<&BV> = None;

        for Field(key, _, bv) in self.iter_fields() {
            if key.is(name) && found_date.is_none() {
                found = Some(bv);
            } else if let Ok(isdate) = Date::try_from(key) {
                if isdate <= date && (found_date.is_none() || found_date.unwrap() < isdate) {
                    if let Some(value) = bv.get_block().and_then(|b| b.get_field(name)) {
                        found_date = Some(isdate);
                        found = Some(value);
                    }
                }
            }
        }
        found
    }

    pub fn macro_parms(&self) -> Vec<&str> {
        let mut vec = Vec::new();
        if let Some((source, _)) = &self.source {
            let mut odd = false;
            for part in source {
                odd = !odd;
                if !odd {
                    vec.push(part.as_str());
                }
            }
            vec.sort_unstable();
            vec.dedup();
        }
        vec
    }

    pub fn expand_macro(&self, args: &[(&str, Token)], link: &Token) -> Option<Block> {
        let link = Arc::new(link.loc.clone());
        if let Some((source, local_macros)) = &self.source {
            let mut content = Vec::new();
            let mut odd = false;
            for part in source {
                odd = !odd;
                if odd {
                    let mut part = part.clone();
                    part.loc.link = Some(link.clone());
                    content.push(part);
                } else {
                    for (arg, val) in args {
                        if part.is(arg) {
                            // Make the replacement be a token that has the substituted content, but the original's loc,
                            // and a loc.link back to the caller's parameter. This gives the best error messages.
                            let mut val = val.clone();
                            let orig_loc = val.loc.clone();
                            val.loc = part.loc.clone();
                            val.loc.link = Some(Arc::new(orig_loc));
                            content.push(val);
                            break;
                        }
                    }
                }
            }
            Some(parse_pdx_macro(&content, local_macros.clone()))
        } else {
            None
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        if self.v.len() != other.v.len() {
            return false;
        }
        for i in 0..self.v.len() {
            if !self.v[i].equivalent(&other.v[i]) {
                return false;
            }
        }
        true
    }

    /// Create a version of this block where the `tag` is combined with a token that follows it.
    /// Example: `color1 = list colorlist` becomes `color1 = list"colorlist` (where the `"` character
    /// is used as the separator because it can't show up in normal parsing).
    pub fn condense_tag(self, tag: &str) -> Self {
        let mut other = Block::new(self.loc);
        let mut reserve: Option<(Token, Comparator, Token)> = None;
        for item in self.v {
            if let Some((rkey, rcmp, mut rtoken)) = reserve {
                if let BlockItem::Value(token) = item {
                    // Combine current value with reserved assignment
                    rtoken.combine(&token, '"');
                    other.add_key_value(rkey, rcmp, BV::Value(rtoken));
                    reserve = None;
                    continue;
                }
                other.add_key_value(rkey, rcmp, BV::Value(rtoken));
                reserve = None;
            }
            if let BlockItem::Field(Field(key, cmp, bv)) = item {
                match bv {
                    BV::Value(token) => {
                        if token.is(tag) {
                            reserve = Some((key, cmp, token));
                            continue;
                        }
                        other.add_key_value(key, cmp, BV::Value(token));
                    }
                    BV::Block(block) => {
                        other.add_key_value(key, cmp, BV::Block(block.condense_tag(tag)));
                    }
                }
            } else {
                other.add_item(item);
            }
        }
        other
    }
}

#[derive(Clone, Debug)]
pub struct IterAssignments<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterAssignments<'a> {
    type Item = (&'a Token, &'a Token);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if self.warn {
                item.expect_assignment();
            }
            if let Some((key, token)) = item.get_assignment() {
                return Some((key, token));
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct IterDefinitions<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterDefinitions<'a> {
    type Item = (&'a Token, &'a Block);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if self.warn {
                item.expect_definition();
            }
            if let Some((key, block)) = item.get_definition() {
                return Some((key, block));
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct IterAssignmentsAndDefinitions<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterAssignmentsAndDefinitions<'a> {
    type Item = (&'a Token, &'a BV);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if self.warn {
                item.expect_field();
            }
            if let BlockItem::Field(field) = item {
                if !field.is_eq() {
                    if self.warn {
                        field.expect_eq();
                    }
                    continue;
                }
                return Some((field.key(), field.bv()));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct DrainDefinitions<'a> {
    iter: std::vec::Drain<'a, BlockItem>,
}

impl Iterator for DrainDefinitions<'_> {
    type Item = (Token, Block);

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if let Some((key, block)) = item.expect_into_definition() {
                return Some((key, block));
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct IterFields<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterFields<'a> {
    type Item = &'a Field;

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if self.warn {
                item.expect_field();
            }
            if let BlockItem::Field(field) = item {
                return Some(field);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct IterValues<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterValues<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if self.warn {
                item.expect_value();
            }
            return item.get_value();
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct IterBlocks<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterBlocks<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if self.warn {
                item.expect_block();
            }
            return item.get_block();
        }
        None
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Date {
    year: i16,
    month: i8,
    day: i8,
}

impl Date {
    pub fn new(year: i16, month: i8, day: i8) -> Self {
        Date { year, month, day }
    }
}

impl FromStr for Date {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split('.');
        let year = splits.next().ok_or(Error)?;
        let month = splits.next().unwrap_or("1");
        let mut day = splits.next().unwrap_or("1");
        if splits.next().is_some() {
            return Err(Error);
        }
        if day.is_empty() {
            day = "1";
        }
        Ok(Date {
            year: year.parse().map_err(|_| Error)?,
            month: month.parse().map_err(|_| Error)?,
            day: day.parse().map_err(|_| Error)?,
        })
    }
}

impl TryFrom<&Token> for Date {
    type Error = Error;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}.{}.{}", self.year, self.month, self.day)
    }
}
