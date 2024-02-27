//! [`Block`] is the core type to represent Pdx script code

use crate::capnp::pdxfile_capnp::block::Builder;
use crate::date::Date;
use crate::macros::MACRO_MAP;
use crate::parse::pdxfile::{parse_pdx_macro, MacroComponent, MacroComponentKind};
use crate::token::{Loc, Token};

mod blockitem;
mod bv;
mod comparator;
mod field;
mod serializer;

pub use crate::block::blockitem::BlockItem;
pub use crate::block::bv::BV;
pub use crate::block::comparator::{Comparator, Eq};
pub use crate::block::field::Field;
pub use crate::block::serializer::Serializer;

/// This type represents the most basic structural element of Pdx script code.
/// Blocks are delimited by `{` and `}`. An entire file is also a `Block`.
///
/// A `Block` can contain a mix of these kinds of items:
///
/// * Assignments: `key = value`
/// * Definitions: `key = { ... }`
/// * Loose sub-blocks: `{ ... } { ... } ...`
/// * Loose values: `value value ...`
/// * Comparisons: `key < value` for a variety of comparators, including `=` for equality
/// * `key < { ... }` is accepted by the parser but is not used anywhere
///
/// The same key can occur multiple times in a block. If a single field is requested and its key
/// occurs multiple times, the last instance is returned (which is how the game usually resolves
/// this).
#[derive(Clone, Debug)]
pub struct Block {
    /// The contents of this block.
    v: Vec<BlockItem>,
    /// The `tag` is a short string that precedes a block, as in `color = hsv { 0.5 0.5 1.0 }`.
    /// Only a small number of hardcoded tags are parsed this way.
    /// It is in a `Box` to save space in blocks that don't have a tag, which is most of them.
    pub tag: Option<Box<Token>>,
    /// The location of the start of the block. Used mostly for error reporting.
    pub loc: Loc,
    /// If the block is a top-level block and contains macro substitutions, this field will
    /// hold the original source for re-parsing.
    /// The source has already been split into a vec that alternates content with macro parameters.
    /// It is in a `Box` to save space (80 bytes) from blocks that don't contain macro substitutions,
    /// which is most of them.
    pub source: Option<Vec<MacroComponent>>,
}

impl Block {
    /// Open a new `Block` at the given location.
    pub fn new(loc: Loc) -> Self {
        Block { v: Vec::new(), tag: None, loc, source: None }
    }

    /// Add a loose value to this `Block`. Mostly used by the parser.
    pub fn add_value(&mut self, value: Token) {
        self.v.push(BlockItem::Value(value));
    }

    /// Add a loose sub-block to this `Block`. Mostly used by the parser.
    pub fn add_block(&mut self, block: Block) {
        self.v.push(BlockItem::Block(block));
    }

    /// Add a `key = value` or `key = { ... }` field to this `Block`.
    /// Mostly used by the parser.
    pub fn add_key_bv(&mut self, key: Token, cmp: Comparator, value: BV) {
        self.v.push(BlockItem::Field(Field(key, cmp, value)));
    }

    /// Add a `BlockItem` to this `Block`.
    /// It can contain any of the variations of things that a `Block` can hold.
    pub fn add_item(&mut self, item: BlockItem) {
        self.v.push(item);
    }

    /// If this block contains a field `name` which takes a block as argument, add the contents of
    /// `block` to that block. It's used for merging items that have special merging rules.
    /// Currently only used for `on_action`.
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

    /// Combine two blocks by adding the contents of `other` to this block.
    /// To avoid lots of cloning, `other` will be emptied in the process.
    pub fn append(&mut self, other: &mut Block) {
        self.v.append(&mut other.v);
    }

    /// Get the value of a single `name = value` assignment.
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

    /// Check if `name` is a field that has the literal string `value` as its value.
    pub fn field_value_is(&self, name: &str, value: &str) -> bool {
        if let Some(token) = self.get_field_value(name) {
            token.is(value)
        } else {
            false
        }
    }

    /// Get the value of a literal boolean field
    pub fn get_field_bool(&self, name: &str) -> Option<bool> {
        self.get_field_value(name).map(|t| t.is("yes"))
    }

    /// Get the value of a literal integer field
    #[allow(dead_code)] // Not used by all games
    pub fn get_field_integer(&self, name: &str) -> Option<i64> {
        self.get_field_value(name).and_then(Token::get_integer)
    }

    /// Get the value of a literal date field
    #[allow(dead_code)] // Not used by all games
    pub fn get_field_date(&self, name: &str) -> Option<Date> {
        self.get_field_value(name).and_then(Token::get_date)
    }

    /// Get all the values of `name = value` assignments in this block
    ///
    /// TODO: should be an iterator
    pub fn get_field_values(&self, name: &str) -> Vec<&Token> {
        let mut vec = Vec::new();
        for (key, token) in self.iter_assignments() {
            if key.is(name) {
                vec.push(token);
            }
        }
        vec
    }

    /// Get the block of a `name = { ... }` definition
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

    /// Get all the blocks of `name = { ... }` definitions in this block
    pub fn get_field_blocks(&self, name: &str) -> Vec<&Block> {
        let mut vec = Vec::new();
        for (key, block) in self.iter_definitions() {
            if key.is(name) {
                vec.push(block);
            }
        }
        vec
    }

    /// Get the values of a single `name = { value value ... }` list
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

    /// Get the combined values of any number of `name = { value value ... }` list
    #[allow(dead_code)] // not used by all games
    pub fn get_multi_field_list(&self, name: &str) -> Vec<Token> {
        let mut vec = Vec::new();
        for item in &self.v {
            if let BlockItem::Field(Field(key, _, bv)) = item {
                if key.is(name) {
                    match bv {
                        BV::Value(_) => (),
                        BV::Block(b) => {
                            vec.extend(b.iter_values().cloned());
                        }
                    }
                }
            }
        }
        vec
    }

    /// Get the value or block on the right-hand side of a field `name`.
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

    /// Get the key of a field `name` in the `Block`. The string value of the key will be equal to
    /// `name`, but it can be useful to get this key as a `Token` with its location.
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

    /// Get all the keys of fields with key `name`. The string values of these keys will be equal
    /// to `name`, but it can be useful to get these keys as `Token` with their locations.
    pub fn get_keys(&self, name: &str) -> Vec<&Token> {
        let mut vec = Vec::new();
        for Field(key, _, _) in self.iter_fields() {
            if key.is(name) {
                vec.push(key);
            }
        }
        vec
    }

    /// Return true iff the `name` occurs in this block at least once as a field key.
    pub fn has_key(&self, name: &str) -> bool {
        self.get_key(name).is_some()
    }

    #[cfg(feature = "vic3")]
    pub fn has_key_recursive(&self, name: &str) -> bool {
        for item in &self.v {
            match item {
                BlockItem::Field(Field(key, _, bv)) => {
                    if key.is(name) {
                        return true;
                    }
                    if let Some(block) = bv.get_block() {
                        if block.has_key_recursive(name) {
                            return true;
                        }
                    }
                }
                BlockItem::Block(block) => {
                    if block.has_key_recursive(name) {
                        return true;
                    }
                }
                BlockItem::Value(_) => (),
            }
        }
        false
    }

    /// Return the number of times `name` occurs in this block as a field key.
    #[allow(dead_code)] // Not used by all games
    pub fn count_keys(&self, name: &str) -> usize {
        let mut count = 0;
        for Field(key, _, _) in self.iter_fields() {
            if key.is(name) {
                count += 1;
            }
        }
        count
    }

    /// Return an iterator over the contents of this block.
    pub fn iter_items(&self) -> std::slice::Iter<BlockItem> {
        self.v.iter()
    }

    /// Return a destructive iterator over the contents of this block.
    /// It will give ownership of the returned `BlockItem` objects.
    pub fn drain(&mut self) -> std::vec::Drain<BlockItem> {
        self.v.drain(..)
    }

    /// Return an iterator over all the `key = ...` fields in this block, ignoring the loose values
    /// and loose blocks.
    pub fn iter_fields(&self) -> IterFields {
        IterFields { iter: self.v.iter(), warn: false }
    }

    /// Return an iterator over all the `key = ...` fields in this block, while warning about loose values
    /// and loose blocks.
    #[allow(dead_code)] // Not used by all games
    pub fn iter_fields_warn(&self) -> IterFields {
        IterFields { iter: self.v.iter(), warn: true }
    }

    /// Return an iterator over all the `key = value` fields in this block, ignoring other kinds of contents.
    pub fn iter_assignments(&self) -> IterAssignments {
        IterAssignments { iter: self.v.iter(), warn: false }
    }

    /// Return an iterator over all the `key = value` fields in this block, while warning about
    /// every other kind of content.
    #[allow(dead_code)] // It's here for symmetry
    pub fn iter_assignments_warn(&self) -> IterAssignments {
        IterAssignments { iter: self.v.iter(), warn: true }
    }

    /// Return an iterator over all the `key = { ... }` fields in this block, ignoring other kinds of contents.
    pub fn iter_definitions(&self) -> IterDefinitions {
        IterDefinitions { iter: self.v.iter(), warn: false }
    }

    /// Return an iterator over all the `key = { ... }` fields in this block, while warning about
    /// every other kind of content.
    pub fn iter_definitions_warn(&self) -> IterDefinitions {
        IterDefinitions { iter: self.v.iter(), warn: true }
    }

    /// Return an iterator over all the `key = value` and `key = { ... }` fields in this block,
    /// ignoring every other kind of content.
    /// It differs from [`Block::iter_fields`] in that it requires the comparator to be `=`.
    #[allow(dead_code)] // It's here for symmetry
    pub fn iter_assignments_and_definitions(&self) -> IterAssignmentsAndDefinitions {
        IterAssignmentsAndDefinitions { iter: self.v.iter(), warn: false }
    }

    /// Return an iterator over all the `key = value` and `key = { ... }` fields in this block,
    /// while warning about every other kind of content.
    /// It differs from [`Block::iter_fields_warn`] in that it requires the comparator to be `=`.
    pub fn iter_assignments_and_definitions_warn(&self) -> IterAssignmentsAndDefinitions {
        IterAssignmentsAndDefinitions { iter: self.v.iter(), warn: true }
    }

    /// Like [`Block::iter_definitions_warn`] but it's a destructive iterator that gives ownership
    /// over the returned definitions.
    pub fn drain_definitions_warn(&mut self) -> DrainDefinitions {
        DrainDefinitions { iter: self.v.drain(..) }
    }

    /// Iterate over the loose values in the block.
    pub fn iter_values(&self) -> IterValues {
        IterValues { iter: self.v.iter(), warn: false }
    }

    /// Iterate over the loose values in the block, while warning about everything else.
    pub fn iter_values_warn(&self) -> IterValues {
        IterValues { iter: self.v.iter(), warn: true }
    }

    /// Iterate over the loose sub-blocks in the block.
    pub fn iter_blocks(&self) -> IterBlocks {
        IterBlocks { iter: self.v.iter(), warn: false }
    }

    /// Iterate over the loose sub-blocks in the block, while warning about everything else.
    #[allow(dead_code)] // It's here for symmetry
    pub fn iter_blocks_warn(&self) -> IterBlocks {
        IterBlocks { iter: self.v.iter(), warn: true }
    }

    /// Search through the history fields in this block and return the block or value the
    /// field `name` would have at the given `date`. The field value that's directly in this block,
    /// not in any history block, is considered to be the field value at the beginning of time.
    /// History fields are ones that have a date as the key, like `900.1.1 = { ... }`.
    #[allow(dead_code)] // Not used by all games
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

    /// Just like [`Block::get_field_at_date`] but only for fields that have values (not blocks).
    #[allow(dead_code)] // Not used by all games
    pub fn get_field_value_at_date(&self, name: &str, date: Date) -> Option<&Token> {
        self.get_field_at_date(name, date).and_then(BV::get_value)
    }

    /// Return a sorted vector of macro parameters taken by this block.
    /// Macro parameters are between `$` like `$CHARACTER$`.
    pub fn macro_parms(&self) -> Vec<&str> {
        if let Some(source) = &self.source {
            let mut vec = source
                .iter()
                .filter(|mc| mc.kind() == MacroComponentKind::Macro)
                .map(|mc| mc.token().as_str())
                .collect::<Vec<_>>();
            vec.sort_unstable();
            vec.dedup();
            vec
        } else {
            Vec::new()
        }
    }

    /// Expand a block that has macro parameters by substituting arguments for those parameters,
    /// then re-parsing the script, that links the expanded content back to `loc`.
    pub fn expand_macro(&self, args: &[(&str, Token)], loc: Loc) -> Option<Block> {
        let link_index = MACRO_MAP.get_or_insert_loc(loc);
        if let Some(source) = &self.source {
            let mut content = Vec::new();
            for part in source {
                let token = part.token();
                match part.kind() {
                    MacroComponentKind::Source | MacroComponentKind::LocalValue => {
                        content.push(token.clone().linked(Some(link_index)));
                    }
                    MacroComponentKind::Macro => {
                        for (arg, val) in args {
                            if token.is(arg) {
                                // Make the replacement be a token that has the substituted content, but the original's loc,
                                // and a loc.link back to the caller's parameter. This gives the best error messages.
                                let mut val = val.clone();
                                let orig_loc = val.loc;
                                val.loc = token.loc;
                                val.loc.column -= 1; // point at the $, it looks better
                                val.loc.link_idx = Some(MACRO_MAP.get_or_insert_loc(orig_loc));
                                content.push(val);
                                break;
                            }
                        }
                    }
                }
            }
            Some(parse_pdx_macro(&content))
        } else {
            None
        }
    }

    /// Return true iff this block has the same block items in the same order as `other`,
    /// including equivalence of blocks inside them.
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
    ///
    /// This function is used as a last resort when validating awkward syntax.
    pub fn condense_tag(self, tag: &str) -> Self {
        let mut other = Block::new(self.loc);
        let mut reserve: Option<(Token, Comparator, Token)> = None;
        for item in self.v {
            if let Some((rkey, rcmp, mut rtoken)) = reserve {
                if let BlockItem::Value(token) = item {
                    // Combine current value with reserved assignment
                    rtoken.combine(&token, '"');
                    other.add_key_bv(rkey, rcmp, BV::Value(rtoken));
                    reserve = None;
                    // This consumed the current item
                    continue;
                }
                other.add_key_bv(rkey, rcmp, BV::Value(rtoken));
                reserve = None;
            }
            if let BlockItem::Field(Field(key, cmp, bv)) = item {
                match bv {
                    BV::Value(token) => {
                        if token.is(tag) {
                            reserve = Some((key, cmp, token));
                            continue;
                        }
                        other.add_key_bv(key, cmp, BV::Value(token));
                    }
                    BV::Block(block) => {
                        other.add_key_bv(key, cmp, BV::Block(block.condense_tag(tag)));
                    }
                }
            } else {
                other.add_item(item);
            }
        }
        other
    }

    pub fn serialize(&self, s: &mut Serializer, m: &mut Builder) {
        if let Some(tag) = self.tag.as_ref() {
            s.add_token(&mut m.reborrow().init_tag(), tag.as_ref());
        }
        // From the loc only line and column are needed, because the file information is kept
        // centralized for the whole file, and the link index is not used in raw file parse
        // results (which are the only blocks we serialize).
        m.set_line(self.loc.line);
        m.set_column(self.loc.column);

        if let Some(source) = self.source.as_ref() {
            #[allow(clippy::cast_possible_truncation)]
            let mut source_builder = m.reborrow().init_source(source.len() as u32);
            for (i, mc) in source.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let mc_builder = source_builder.reborrow().get(i as u32);
                let mut mc_token_builder = match mc.kind() {
                    MacroComponentKind::Source => mc_builder.init_source(),
                    MacroComponentKind::LocalValue => mc_builder.init_local_value(),
                    MacroComponentKind::Macro => mc_builder.init_macro(),
                };
                s.add_token(&mut mc_token_builder, mc.token());
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        let mut items_builder = m.reborrow().init_items(self.v.len() as u32);
        for (i, blockitem) in self.v.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let mut item_builder = items_builder.reborrow().get(i as u32);
            blockitem.serialize(s, &mut item_builder);
        }
    }
}

/// An iterator for (key, value) pairs. It is returned by [`Block::iter_assignments`].
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

/// An iterator for (key, block) pairs. It is returned by [`Block::iter_definitions`].
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

/// An iterator for (key, bv) pairs. It is returned by [`Block::iter_assignments_and_definitions`].
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

/// An iterator for (key, block) pairs that transfers ownership.
/// It is returned by [`Block::drain_definitions_warn`].
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

/// An iterator for [`Field`] structs, returning the fields of a block.
/// It is returned by [`Block::iter_fields`].
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

/// An iterator for values (tokens), returning the loose values of a block.
/// It is returned by [`Block::iter_values`].
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
            if let BlockItem::Value(value) = item {
                return Some(value);
            }
        }
        None
    }
}

/// An iterator returning the loose sub-blocks of a block.
/// It is returned by [`Block::iter_blocks`].
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
            if let BlockItem::Block(block) = item {
                return Some(block);
            }
        }
        None
    }
}
