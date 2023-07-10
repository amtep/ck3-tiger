use crate::block::Eq::{Double, Question, Single};
use std::borrow::Cow;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;
use std::sync::Arc;

use crate::parse::pdxfile::{parse_pdx_macro, LocalMacros};
use crate::report::{error, error_info, ErrorKey};
use crate::token::{Loc, Token};

pub mod validator;

/// BV is an item in a Block, either on its own or after a field key.
/// It is itself either a Block or a single-token Value.
#[derive(Clone, Debug)]
pub enum BV {
    Value(Token),
    Block(Block),
}

impl BV {
    pub fn get_block(&self) -> Option<&Block> {
        match self {
            BV::Value(_) => None,
            BV::Block(b) => Some(b),
        }
    }

    pub fn get_value(&self) -> Option<&Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => None,
        }
    }

    pub fn expect_block(&self) -> Option<&Block> {
        match self {
            BV::Value(_) => {
                error(self, ErrorKey::Validation, "expected block, found value");
                None
            }
            BV::Block(b) => Some(b),
        }
    }

    pub fn expect_value(&self) -> Option<&Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => {
                error(self, ErrorKey::Validation, "expected value, found block");
                None
            }
        }
    }

    pub fn is_value(&self) -> bool {
        match self {
            BV::Value(_) => true,
            BV::Block(_) => false,
        }
    }

    pub fn is_block(&self) -> bool {
        !self.is_value()
    }

    pub fn into_value(self) -> Option<Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => None,
        }
    }

    pub fn expect_into_value(self) -> Option<Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => {
                error(self, ErrorKey::Validation, "expected value, found block");
                None
            }
        }
    }

    pub fn into_block(self) -> Option<Block> {
        match self {
            BV::Value(_) => None,
            BV::Block(b) => Some(b),
        }
    }

    pub fn expect_into_block(self) -> Option<Block> {
        match self {
            BV::Value(_) => {
                error(self, ErrorKey::Validation, "expected block, found value");
                None
            }
            BV::Block(b) => Some(b),
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        match self {
            BV::Value(t1) => {
                if let Some(t2) = other.get_value() {
                    t1.is(t2.as_str())
                } else {
                    false
                }
            }
            BV::Block(b1) => {
                if let Some(b2) = other.get_block() {
                    b1.equivalent(b2)
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field(pub Token, pub Comparator, pub BV);

impl Field {
    pub fn into_key(self) -> Token {
        self.0
    }

    pub fn key(&self) -> &Token {
        &self.0
    }

    pub fn cmp(&self) -> Comparator {
        self.1
    }

    pub fn into_bv(self) -> BV {
        self.2
    }

    pub fn bv(&self) -> &BV {
        &self.2
    }

    pub fn is_eq(&self) -> bool {
        matches!(self.1, Comparator::Equals(Single))
    }

    pub fn is_eq_qeq(&self) -> bool {
        matches!(self.1, Comparator::Equals(Single | Question))
    }

    pub fn expect_eq(&self) -> bool {
        let Self(key, cmp, _) = self;
        if matches!(cmp, Comparator::Equals(Single)) {
            true
        } else {
            error(self, ErrorKey::Validation, &format!("expected `{key} =`, found `{cmp}`"));
            false
        }
    }

    pub fn describe(&self) -> &'static str {
        if self.is_eq_qeq() {
            match self.2 {
                BV::Value(_) => "assignment",
                BV::Block(_) => "definition",
            }
        } else {
            "comparison"
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2.equivalent(&other.2)
    }

    pub fn expect_definition(&self) -> Option<(&Token, &Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                return Some((key, block))
            }
            _ => {
                let msg = format!("expected definition, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
            }
        }
        None
    }

    pub fn expect_into_definition(self) -> Option<(Token, Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                return Some((key, block))
            }
            _ => {
                let msg = format!("expected definition, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
            }
        }
        None
    }

    pub fn get_definition(&self) -> Option<(&Token, &Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                Some((key, block))
            }
            _ => None,
        }
    }

    pub fn get_into_definition(self) -> Option<(Token, Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                Some((key, block))
            }
            _ => None,
        }
    }
    pub fn expect_assignment(&self) -> Option<(&Token, &Token)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Value(token)) => {
                return Some((key, token))
            }
            _ => {
                let msg = format!("expected assignment, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Value(Token),
    Block(Block),
    Field(Field),
}

impl BlockItem {
    pub fn expect_field(&self) -> Option<&Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => {
                let msg = format!("unexpected {}", self.describe());
                error_info(self, ErrorKey::Validation, &msg, "Did you forget an = ?");
                None
            }
        }
    }

    pub fn expect_into_field(self) -> Option<Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => {
                let msg = format!("unexpected {}", self.describe());
                error_info(self, ErrorKey::Validation, &msg, "Did you forget an = ?");
                None
            }
        }
    }

    pub fn get_field(&self) -> Option<&Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => None,
        }
    }

    pub fn get_into_field(self) -> Option<Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => None,
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self, BlockItem::Field(_))
    }

    pub fn get_value(&self) -> Option<&Token> {
        match self {
            BlockItem::Value(token) => Some(token),
            _ => None,
        }
    }

    pub fn expect_value(&self) -> Option<&Token> {
        match self {
            BlockItem::Value(token) => Some(token),
            _ => {
                let msg = format!("expected value, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn expect_into_value(self) -> Option<Token> {
        match self {
            BlockItem::Value(token) => Some(token),
            _ => {
                let msg = format!("expected value, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn get_block(&self) -> Option<&Block> {
        match self {
            BlockItem::Block(block) => Some(block),
            _ => None,
        }
    }

    pub fn expect_block(&self) -> Option<&Block> {
        match self {
            BlockItem::Block(block) => Some(block),
            _ => {
                let msg = format!("expected block, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn expect_into_block(self) -> Option<Block> {
        match self {
            BlockItem::Block(block) => Some(block),
            _ => {
                let msg = format!("expected block, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn get_definition(&self) -> Option<(&Token, &Block)> {
        if let Some(field) = self.get_field() {
            field.get_definition()
        } else {
            None
        }
    }

    pub fn expect_into_definition(self) -> Option<(Token, Block)> {
        if let Some(field) = self.expect_into_field() {
            field.expect_into_definition()
        } else {
            None
        }
    }

    pub fn expect_definition(&self) -> Option<(&Token, &Block)> {
        if let Some(field) = self.expect_field() {
            field.expect_definition()
        } else {
            None
        }
    }

    pub fn get_into_definition(self) -> Option<(Token, Block)> {
        if let Some(field) = self.get_into_field() {
            field.get_into_definition()
        } else {
            None
        }
    }

    pub fn expect_assignment(&self) -> Option<(&Token, &Token)> {
        if let Some(field) = self.expect_field() {
            match field {
                Field(key, Comparator::Equals(Single | Question), BV::Value(token)) => {
                    return Some((key, token))
                }
                _ => {
                    let msg = format!("expected assignment, found {}", field.describe());
                    error(self, ErrorKey::Validation, &msg);
                }
            }
        }
        None
    }

    pub fn get_assignment(&self) -> Option<(&Token, &Token)> {
        match self {
            BlockItem::Field(Field(
                key,
                Comparator::Equals(Single | Question),
                BV::Value(token),
            )) => Some((key, token)),
            _ => None,
        }
    }

    pub fn describe(&self) -> &'static str {
        match self {
            BlockItem::Value(_) => "value",
            BlockItem::Block(_) => "block",
            BlockItem::Field(field) => field.describe(),
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        match self {
            BlockItem::Value(token) => {
                if let BlockItem::Value(t) = other {
                    token == t
                } else {
                    false
                }
            }
            BlockItem::Block(block) => {
                if let BlockItem::Block(b) = other {
                    block.equivalent(b)
                } else {
                    false
                }
            }
            BlockItem::Field(field) => {
                if let BlockItem::Field(f) = other {
                    field.equivalent(f)
                } else {
                    false
                }
            }
        }
    }
}

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Comparator {
    /// =, ?=, ==,
    Equals(Eq),
    /// !=
    NotEquals,
    /// <
    LessThan,
    /// >
    GreaterThan,
    /// <=
    AtMost,
    /// >=
    AtLeast,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Eq {
    /// Notation: =
    /// Valid as an equality comparison operator, assignment operator and scope opener.
    Single,
    /// Notation: ==
    /// Only valid as an equality comparison operator.
    Double,
    /// Notation: ?=
    /// Valid as a conditional equality comparison operator and condition scope opener.
    Question,
}

impl Comparator {
    pub fn from_str(s: &str) -> Option<Self> {
        if s == "=" {
            Some(Comparator::Equals(Single))
        } else if s == "==" {
            Some(Comparator::Equals(Double))
        } else if s == "?=" {
            Some(Comparator::Equals(Question))
        } else if s == "<" {
            Some(Comparator::LessThan)
        } else if s == ">" {
            Some(Comparator::GreaterThan)
        } else if s == "<=" {
            Some(Comparator::AtMost)
        } else if s == ">=" {
            Some(Comparator::AtLeast)
        } else if s == "!=" {
            Some(Comparator::NotEquals)
        } else {
            None
        }
    }
    pub fn from_token(token: &Token) -> Option<Self> {
        Self::from_str(token.as_str())
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Comparator::Equals(Single) => write!(f, "="),
            Comparator::Equals(Double) => write!(f, "=="),
            Comparator::Equals(Question) => write!(f, "?="),
            Comparator::LessThan => write!(f, "<"),
            Comparator::GreaterThan => write!(f, ">"),
            Comparator::AtMost => write!(f, "<="),
            Comparator::AtLeast => write!(f, ">="),
            Comparator::NotEquals => write!(f, "!="),
        }
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
