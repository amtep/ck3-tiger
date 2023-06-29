use std::borrow::Cow;
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;
use std::str::FromStr;

pub mod validator;

use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info};
use crate::parse::pdxfile::{parse_pdx_macro, LocalMacros};
use crate::token::{Loc, Token};

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

type BlockItem = (Option<Token>, Comparator, BV);

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
        Block {
            v: Vec::new(),
            tag: None,
            loc,
            source: None,
        }
    }

    pub fn add_value(&mut self, value: BV) {
        self.v.push((None, Comparator::None, value));
    }

    pub fn add_key_value(&mut self, key: Token, cmp: Comparator, value: BV) {
        self.v.push((Some(key), cmp, value));
    }

    pub fn add_to_field_block(&mut self, name: &str, block: &mut Block) -> bool {
        for (k, _, bv) in self.v.iter_mut().rev() {
            if let Some(key) = k {
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
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
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
        self.get_field_value(name)
            .and_then(|t| t.as_str().parse().ok())
    }

    /// Get all the values of `name = value` assignments in this block
    pub fn get_field_values(&self, name: &str) -> Vec<Token> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BV::Value(t) => vec.push(t.clone()),
                        BV::Block(_) => (),
                    }
                }
            }
        }
        vec
    }

    /// Get the block of a `name = { ... }` assignment
    pub fn get_field_block(&self, name: &str) -> Option<&Block> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BV::Value(_) => (),
                        BV::Block(s) => return Some(s),
                    }
                }
            }
        }
        None
    }

    /// Get all the blocks of `name = { ... }` assignments in this block
    pub fn get_field_blocks(&self, name: &str) -> Vec<&Block> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BV::Value(_) => (),
                        BV::Block(s) => vec.push(s),
                    }
                }
            }
        }
        vec
    }

    /// Get the values of a single `name = { value ... }` assignment
    pub fn get_field_list(&self, name: &str) -> Option<Vec<Token>> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    match v {
                        BV::Value(_) => (),
                        BV::Block(s) => {
                            return Some(s.get_values());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_field(&self, name: &str) -> Option<&BV> {
        for (k, _, v) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    return Some(v);
                }
            }
        }
        None
    }

    /// Get all the unkeyed values in this block
    pub fn get_values(&self) -> Vec<Token> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if k.is_none() {
                match v {
                    BV::Value(t) => vec.push(t.clone()),
                    BV::Block(_) => (),
                }
            }
        }
        vec
    }

    /// Get all the unkeyed blocks in this block
    pub fn get_sub_blocks(&self) -> Vec<Block> {
        let mut vec = Vec::new();
        for (k, _, v) in &self.v {
            if k.is_none() {
                match v {
                    BV::Value(_) => (),
                    BV::Block(b) => vec.push(b.clone()),
                }
            }
        }
        vec
    }

    /// Get all the token = token items in this block
    pub fn get_assignments(&self) -> Vec<(&Token, &Token)> {
        let mut vec = Vec::new();
        for (k, cmp, v) in &self.v {
            if let Some(key) = k {
                if matches!(cmp, Comparator::Eq) {
                    match v {
                        BV::Value(t) => vec.push((key, t)),
                        BV::Block(_) => (),
                    }
                }
            }
        }
        vec
    }
    pub fn dbg_keys(&self) {
        for (k, _, _) in &self.v {
            if let Some(k) = k {
                let key = k.as_str();
                dbg!(key);
            }
        }
    }

    pub fn get_key(&self, name: &str) -> Option<&Token> {
        for (k, _, _) in self.v.iter().rev() {
            if let Some(key) = k {
                if key.is(name) {
                    return Some(key);
                }
            }
        }
        None
    }

    pub fn has_key(&self, name: &str) -> bool {
        self.get_key(name).is_some()
    }

    pub fn count_keys(&self, name: &str) -> usize {
        let mut count = 0;
        for (k, _, _) in self.v.iter() {
            if let Some(key) = k {
                if key.is(name) {
                    count += 1
                }
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

    pub fn iter_assignments(&self) -> IterAssignments {
        IterAssignments {
            iter: self.v.iter(),
            warn: false,
        }
    }

    pub fn iter_assignments_warn(&self) -> IterAssignments {
        IterAssignments {
            iter: self.v.iter(),
            warn: true,
        }
    }

    pub fn iter_bv_definitions_warn(&self) -> IterBlockValueDefinitions {
        IterBlockValueDefinitions {
            iter: self.v.iter(),
            warn: true,
        }
    }

    pub fn iter_definitions_warn(&self) -> IterDefinitions {
        IterDefinitions {
            iter: self.v.iter(),
            warn: true,
        }
    }

    pub fn iter_definitions(&self) -> IterDefinitions {
        IterDefinitions {
            iter: self.v.iter(),
            warn: false,
        }
    }

    pub fn drain_definitions_warn<'a>(&'a mut self) -> DrainDefinitions<'a> {
        DrainDefinitions {
            iter: self.v.drain(..),
        }
    }

    pub fn get_field_at_date(&self, name: &str, date: Date) -> Option<BV> {
        let mut found_date: Option<Date> = None;
        let mut found: Option<&BV> = None;

        for (k, _, v) in &self.v {
            if let Some(k) = k {
                if k.is(name) && found_date.is_none() {
                    found = Some(v);
                } else if let Ok(isdate) = Date::try_from(k) {
                    if isdate <= date && (found_date.is_none() || found_date.unwrap() < isdate) {
                        if let Some(value) = v.get_block().and_then(|b| b.get_field(name)) {
                            found_date = Some(isdate);
                            found = Some(value);
                        }
                    }
                }
            }
        }
        found.cloned()
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
        let link = Rc::new(link.loc.clone());
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
                            val.loc.link = Some(Rc::new(orig_loc));
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
            if let Some(ref key1) = self.v[i].0 {
                if let Some(ref key2) = other.v[i].0 {
                    if !key1.is(key2.as_str()) {
                        return false;
                    }
                } else {
                    return false;
                }
            } else if other.v[i].0.is_some() {
                return false;
            }

            if self.v[i].1 != other.v[i].1 || !self.v[i].2.equivalent(&other.v[i].2) {
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
        let mut reserve: Option<(Token, Comparator, BV)> = None;
        for (k, cmp, bv) in self.v {
            if let Some((rkey, rcmp, rbv)) = reserve {
                if k.is_none() {
                    if let BV::Value(token) = bv {
                        if let BV::Value(mut rtoken) = rbv {
                            // Combine current value with reserved assignment
                            rtoken.combine(&token, '"');
                            other.add_key_value(rkey, rcmp, BV::Value(rtoken));
                        }
                    } else {
                        // Can't use current bv, so send the reserve and then this bv separately
                        other.add_key_value(rkey, rcmp, rbv);
                        other.add_value(bv);
                    }
                    reserve = None;
                    continue;
                }
                other.add_key_value(rkey, rcmp, rbv);
                reserve = None;
            }
            if let Some(key) = k {
                match bv {
                    BV::Value(token) => {
                        if token.is(tag) {
                            reserve = Some((key, cmp, BV::Value(token)));
                            continue;
                        }
                        other.add_key_value(key, cmp, BV::Value(token));
                    }
                    BV::Block(block) => {
                        other.add_key_value(key, cmp, BV::Block(block.condense_tag(tag)));
                    }
                }
            } else {
                other.add_value(bv);
            }
        }
        other
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Comparator {
    None,
    Eq,  // Eq is also Assign
    EEq, // The == operator, which means Eq but cannot be used to assign
    QEq, // The ?= operator
    Lt,
    Gt,
    Le,
    Ge,
    Ne,
}

impl Comparator {
    pub fn from_str(s: &str) -> Option<Self> {
        if s == "=" {
            Some(Comparator::Eq)
        } else if s == "?=" {
            Some(Comparator::QEq)
        } else if s == "==" {
            Some(Comparator::EEq)
        } else if s == "<" {
            Some(Comparator::Lt)
        } else if s == ">" {
            Some(Comparator::Gt)
        } else if s == "<=" {
            Some(Comparator::Le)
        } else if s == ">=" {
            Some(Comparator::Ge)
        } else if s == "!=" {
            Some(Comparator::Ne)
        } else {
            None
        }
    }

    pub fn from_token(token: &Token) -> Option<Self> {
        Self::from_str(token.as_str())
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Comparator::Eq => write!(f, "="),
            Comparator::EEq => write!(f, "=="),
            Comparator::QEq => write!(f, "?="),
            Comparator::Lt => write!(f, "<"),
            Comparator::Gt => write!(f, ">"),
            Comparator::Le => write!(f, "<="),
            Comparator::Ge => write!(f, ">="),
            Comparator::Ne => write!(f, "!="),
            Comparator::None => Ok(()),
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
        for (k, cmp, bv) in self.iter.by_ref() {
            if let Some(key) = k {
                if !matches!(cmp, Comparator::Eq) {
                    if self.warn {
                        let msg = format!("expected `{key} =`, found `{cmp}`");
                        error(key, ErrorKey::Validation, &msg);
                    }
                    continue;
                }
                return match bv {
                    BV::Value(t) => Some((key, t)),
                    BV::Block(b) => {
                        if self.warn {
                            error(b, ErrorKey::Validation, "expected value, found block");
                        }
                        None
                    }
                };
            }
            if self.warn {
                match bv {
                    BV::Value(t) => {
                        error_info(
                            t,
                            ErrorKey::Validation,
                            "unexpected value",
                            "Did you forget an = ?",
                        );
                    }
                    BV::Block(b) => {
                        error_info(
                            b,
                            ErrorKey::Validation,
                            "unexpected block",
                            "Did you forget an = ?",
                        );
                    }
                }
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
        for (k, _, bv) in self.iter.by_ref() {
            if let Some(key) = k {
                if let Some(block) = bv.get_block() {
                    return Some((key, block));
                } else if self.warn {
                    error(key, ErrorKey::Validation, "unexpected assignment");
                }
            } else if self.warn {
                match bv {
                    BV::Value(t) => {
                        error_info(
                            t,
                            ErrorKey::Validation,
                            "unexpected value",
                            "Did you forget an = ?",
                        );
                    }
                    BV::Block(b) => {
                        error_info(
                            b,
                            ErrorKey::Validation,
                            "unexpected block",
                            "Did you forget an = ?",
                        );
                    }
                }
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
        for (k, _, bv) in self.iter.by_ref() {
            if let Some(key) = k {
                if let Some(block) = bv.into_block() {
                    return Some((key, block));
                } else {
                    error(key, ErrorKey::Validation, "unexpected assignment");
                }
            } else {
                match bv {
                    BV::Value(t) => {
                        error_info(
                            t,
                            ErrorKey::Validation,
                            "unexpected value",
                            "Did you forget an = ?",
                        );
                    }
                    BV::Block(b) => {
                        error_info(
                            b,
                            ErrorKey::Validation,
                            "unexpected block",
                            "Did you forget an = ?",
                        );
                    }
                }
            }
        }
        None
    }
}
#[derive(Clone, Debug)]
pub struct IterBlockValueDefinitions<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterBlockValueDefinitions<'a> {
    type Item = (&'a Token, &'a BV);

    fn next(&mut self) -> Option<Self::Item> {
        for (k, cmp, bv) in self.iter.by_ref() {
            if let Some(key) = k {
                if !matches!(cmp, Comparator::Eq) {
                    if self.warn {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{key} =`, found `{cmp}`"),
                        );
                    }
                    continue;
                }
                return Some((key, bv));
            }
            if self.warn {
                match bv {
                    BV::Value(t) => {
                        error_info(
                            t,
                            ErrorKey::Validation,
                            "unexpected token",
                            "Did you forget an = ?",
                        );
                    }
                    BV::Block(b) => {
                        error_info(
                            b,
                            ErrorKey::Validation,
                            "unexpected block",
                            "Did you forget an = ?",
                        );
                    }
                }
            }
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
