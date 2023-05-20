use std::borrow::Cow;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

pub mod validator;

use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info};
use crate::parse::pdxfile::{parse_pdx_macro, split_macros, LocalMacros};
use crate::token::{Loc, Token};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub enum BlockOrValue {
    Token(Token),
    Block(Block),
}

impl BlockOrValue {
    pub fn get_block(&self) -> Option<&Block> {
        match self {
            BlockOrValue::Token(_) => None,
            BlockOrValue::Block(b) => Some(b),
        }
    }

    pub fn get_value(&self) -> Option<&Token> {
        match self {
            BlockOrValue::Token(t) => Some(t),
            BlockOrValue::Block(_) => None,
        }
    }

    pub fn expect_block(&self) -> Option<&Block> {
        match self {
            BlockOrValue::Token(_) => {
                error(self, ErrorKey::Validation, "expected block, found value");
                None
            }
            BlockOrValue::Block(b) => Some(b),
        }
    }

    pub fn expect_value(&self) -> Option<&Token> {
        match self {
            BlockOrValue::Token(t) => Some(t),
            BlockOrValue::Block(_) => {
                error(self, ErrorKey::Validation, "expected value, found block");
                None
            }
        }
    }

    pub fn into_value(self) -> Option<Token> {
        match self {
            BlockOrValue::Token(t) => Some(t),
            BlockOrValue::Block(_) => None,
        }
    }
}

type BlockItem = (Option<Token>, Comparator, BlockOrValue);

#[derive(Clone, Debug)]
pub struct Block {
    // v can contain key = value pairs as well as unadorned values.
    // The latter are inserted as None tokens and Comparator::None
    v: Vec<BlockItem>,
    pub tag: Option<Token>,
    pub loc: Loc,
    /// If the block is a top-level block and contains macro substitutions,
    /// this field will hold the original source for re-parsing.
    pub source: Option<(Token, LocalMacros)>,
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

    pub fn add_value(&mut self, value: BlockOrValue) {
        self.v.push((None, Comparator::None, value));
    }

    pub fn add_key_value(&mut self, key: Token, cmp: Comparator, value: BlockOrValue) {
        self.v.push((Some(key), cmp, value));
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
                        BlockOrValue::Token(t) => return Some(t),
                        BlockOrValue::Block(_) => (),
                    }
                }
            }
        }
        None
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
                        BlockOrValue::Token(t) => vec.push(t.clone()),
                        BlockOrValue::Block(_) => (),
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
                        BlockOrValue::Token(_) => (),
                        BlockOrValue::Block(s) => return Some(s),
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
                        BlockOrValue::Token(_) => (),
                        BlockOrValue::Block(s) => vec.push(s),
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
                        BlockOrValue::Token(_) => (),
                        BlockOrValue::Block(s) => {
                            return Some(s.get_values());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_field(&self, name: &str) -> Option<&BlockOrValue> {
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
                    BlockOrValue::Token(t) => vec.push(t.clone()),
                    BlockOrValue::Block(_) => (),
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
                    BlockOrValue::Token(_) => (),
                    BlockOrValue::Block(b) => vec.push(b.clone()),
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
                        BlockOrValue::Token(t) => vec.push((key, t)),
                        BlockOrValue::Block(_) => (),
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

    pub fn iter_items(&self) -> std::slice::Iter<BlockItem> {
        self.v.iter()
    }

    pub fn iter_definitions(&self) -> IterDefinitions {
        IterDefinitions {
            iter: self.v.iter(),
            warn: false,
        }
    }

    pub fn iter_definitions_warn(&self) -> IterDefinitions {
        IterDefinitions {
            iter: self.v.iter(),
            warn: true,
        }
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

    pub fn iter_pure_definitions_warn(&self) -> IterPureDefinitions {
        IterPureDefinitions {
            iter: self.iter_definitions_warn(),
            warn: true,
        }
    }

    pub fn iter_pure_definitions(&self) -> IterPureDefinitions {
        IterPureDefinitions {
            iter: self.iter_definitions(),
            warn: false,
        }
    }

    pub fn get_field_at_date(&self, name: &str, date: Date) -> Option<BlockOrValue> {
        let mut found_date: Option<Date> = None;
        let mut found: Option<&BlockOrValue> = None;

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

    pub fn macro_parms(&self) -> Vec<String> {
        let mut vec = Vec::new();
        if let Some((source, _)) = &self.source {
            let mut odd = false;
            for part in split_macros(source) {
                odd = !odd;
                if !odd {
                    vec.push(part.into_string());
                }
            }
            vec.sort();
            vec.dedup();
        }
        vec
    }

    pub fn expand_macro(&self, args: &[(String, Token)]) -> Option<Block> {
        if let Some((source, local_macros)) = &self.source {
            let mut content = Vec::new();
            let mut odd = false;
            for part in split_macros(source) {
                odd = !odd;
                if odd {
                    content.push(part);
                } else {
                    for (arg, val) in args {
                        if part.is(arg) {
                            content.push(val.clone());
                        }
                    }
                }
            }
            parse_pdx_macro(&content, local_macros.clone())
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Comparator {
    None,
    Eq,  // Eq is also Assign
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

/// A type for callers who are only interested in "definition"-style blocks, where there
/// are no comparisons and no loose blocks.
#[derive(Clone, Debug)]
pub enum DefinitionItem<'a> {
    Assignment(&'a Token, &'a Token), // key = value
    Definition(&'a Token, &'a Block), // key = { definition }
    Keyword(&'a Token),
}

#[derive(Clone, Debug)]
pub struct IterDefinitions<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterDefinitions<'a> {
    type Item = DefinitionItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for (k, cmp, v) in self.iter.by_ref() {
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
                return match v {
                    BlockOrValue::Token(t) => Some(DefinitionItem::Assignment(key, t)),
                    BlockOrValue::Block(b) => Some(DefinitionItem::Definition(key, b)),
                };
            }
            match v {
                BlockOrValue::Token(t) => return Some(DefinitionItem::Keyword(t)),
                BlockOrValue::Block(b) => {
                    if self.warn {
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
pub struct IterAssignments<'a> {
    iter: std::slice::Iter<'a, BlockItem>,
    warn: bool,
}

impl<'a> Iterator for IterAssignments<'a> {
    type Item = (&'a Token, &'a Token);

    fn next(&mut self) -> Option<Self::Item> {
        for (k, cmp, v) in self.iter.by_ref() {
            if let Some(key) = k {
                if !matches!(cmp, Comparator::Eq) {
                    if self.warn {
                        let msg = format!("expected `{key} =`, found `{cmp}`");
                        error(key, ErrorKey::Validation, &msg);
                    }
                    continue;
                }
                return match v {
                    BlockOrValue::Token(t) => Some((key, t)),
                    BlockOrValue::Block(b) => {
                        if self.warn {
                            error(b, ErrorKey::Validation, "expected value, found block");
                        }
                        None
                    }
                };
            }
            match v {
                BlockOrValue::Token(t) => {
                    if self.warn {
                        error_info(
                            t,
                            ErrorKey::Validation,
                            "unexpected value",
                            "Did you forget an = ?",
                        );
                    }
                }
                BlockOrValue::Block(b) => {
                    if self.warn {
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
pub struct IterPureDefinitions<'a> {
    iter: IterDefinitions<'a>,
    warn: bool,
}

impl<'a> Iterator for IterPureDefinitions<'a> {
    type Item = (&'a Token, &'a Block);

    fn next(&mut self) -> Option<Self::Item> {
        for def in self.iter.by_ref() {
            match def {
                DefinitionItem::Keyword(t) => {
                    if self.warn {
                        error_info(
                            t,
                            ErrorKey::Validation,
                            "unexpected token",
                            "Did you forget an = ?",
                        );
                    }
                }
                DefinitionItem::Assignment(key, _) => {
                    if self.warn {
                        error(key, ErrorKey::Validation, "unexpected assignment");
                    }
                }
                DefinitionItem::Definition(key, block) => {
                    return Some((key, block));
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
    type Item = (&'a Token, &'a BlockOrValue);

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
            match bv {
                BlockOrValue::Token(t) => {
                    if self.warn {
                        error_info(
                            t,
                            ErrorKey::Validation,
                            "unexpected token",
                            "Did you forget an = ?",
                        );
                    }
                }
                BlockOrValue::Block(b) => {
                    if self.warn {
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
