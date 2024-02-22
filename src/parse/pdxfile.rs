//! Parses a Pdx script file into a [`Block`].
//!
//! The main entry points are [`parse_pdx`] and [`parse_pdx_macro`].

use std::fmt::Display;
use std::mem::{swap, take};
use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::block::Eq::Single;
use crate::block::{Block, Comparator, BV};
use crate::fileset::{FileEntry, FileKind};
use crate::report::{err, fatal, untidy, warn, ErrorKey};
use crate::token::{bump, leak, Loc, Token};

/// ^Z is by convention an end-of-text marker, and the game engine treats it as such.
const CONTROL_Z: char = '\u{001A}';

#[derive(Debug, Copy, Clone)]
struct IndexLoc(usize, Loc);

impl IndexLoc {
    // ASSUME: current char is single-byte
    #[inline]
    fn next(mut self) -> Self {
        self.0 += 1;
        self.1.column += 1;
        self
    }
}

/// Internal states of the parsing state machine.
#[derive(Copy, Clone, Debug)]
enum State {
    /// Between tokens.
    Neutral,
    /// Parsing a quoted string.
    QString,
    /// Parsing an unquoted token.
    Id,
    /// Parsing a comparator like `=` or `<=`.
    Comparator,
    /// Parsing a macro surrounded by `$...$`.
    Macro,
    /// Parsing a local value `@...`
    LocalValue,
    /// Parsing a `@[ ... ]` local value calculation.
    Calculation(Option<(usize, Loc)>),
    /// Parsing a comment till end of line.
    Comment,
}

/// A type to record the operations done in a `@[ ... ]` local value calculation.
///
/// These calculations are evaluated at parse time, and don't depend on anything outside the current file.
/// Because the game engine respects the conventional order of operations (multiplication and
/// division before addition and subtraction), the operations are stored by the parser and
/// evaluated once the full formula has been parsed.
///
/// The grouping operators (`(` and `)`) are not represented here because they are evaluated inline
/// by the parser using a stack. Each grouped sub-formula is reduced to a single
/// [`Value`](Calculation::Value) before being inserted in the main calculation.
#[derive(Clone, Debug)]
enum Calculation {
    /// Either a literal value, or a lookup of a named @-value, or the result of a previous calculation.
    Value(f64),
    Add,
    /// -, either binary (a - b) or unary negation (-a).
    Subtract,
    Multiply,
    /// Division carries a [`Loc`] in order to report errors about division by zero when appropriate.
    Divide(Loc),
}

impl Display for Calculation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Calculation::Value(v) => write!(f, "{v:.5}"),
            Calculation::Add => write!(f, "+"),
            Calculation::Subtract => write!(f, "-"),
            Calculation::Multiply => write!(f, "*"),
            Calculation::Divide(_) => write!(f, "/"),
        }
    }
}

impl Calculation {
    /// Convenience function. Returns true iff the calculation is a [`Calculation::Value`]
    fn is_value(&self) -> bool {
        match self {
            Calculation::Value(_) => true,
            Calculation::Add
            | Calculation::Subtract
            | Calculation::Multiply
            | Calculation::Divide(_) => false,
        }
    }
}

/// Keeps the stack of calculations and current, top of the stack for processing.
#[derive(Debug)]
struct Calculator {
    stack: Vec<Vec<Calculation>>,
    current: Vec<Calculation>,
}

impl Calculator {
    fn new() -> Self {
        Self { stack: Vec::new(), current: Vec::new() }
    }

    fn start(&mut self) {
        self.stack.clear();
        self.current.clear();
    }

    /// Register an operator.
    fn op(&mut self, op: Calculation, loc: Loc) {
        if let Some(Calculation::Value(_)) = self.current.last() {
            self.current.push(op);
        } else if let Calculation::Subtract = op {
            // accept negation
            self.current.push(op);
        } else {
            let msg = format!("operator `{op}` without left-hand value");
            err(ErrorKey::LocalValues).msg(msg).loc(loc).push();
        }
    }

    /// Register a named local value being used in a `@[ ... ]` calculation.
    ///
    /// The numeric value of this local value will be looked up and inserted in the calculation.
    /// If there's no such value, log an error message.
    fn next(&mut self, local_value: &str, local_values: &LocalValues, loc: Loc) {
        if let Some(value) = local_values.get_value(local_value) {
            self.current.push(Calculation::Value(value));
        } else {
            let msg = format!("local value {local_value} not defined");
            err(ErrorKey::LocalValues).msg(msg).loc(loc).push();
        }
    }

    /// Register an opening `(` in a local value calculation.
    fn push(&mut self, loc: Loc) {
        if let Some(Calculation::Value(_)) = self.current.last() {
            let msg = "calculation has two values with no operator in between";
            err(ErrorKey::LocalValues).msg(msg).loc(loc).push();
        }
        self.stack.push(take(&mut self.current));
    }

    /// Register a closing `)` in a local value calculation.
    fn pop(&mut self, loc: Loc) {
        if let Some(mut calc) = self.stack.pop() {
            calc.push(Calculation::Value(self.result()));
            self.current = calc;
        } else {
            let msg = "found `)` without corresponding `(`";
            warn(ErrorKey::LocalValues).msg(msg).loc(loc).push();
        }
    }

    /// Register the end of a `@[ ... ]` calculation, and return the resulting numerical value.
    fn result(&mut self) -> f64 {
        Self::calculate(take(&mut self.current))
    }

    /// Evaluate a completely parsed formula, hopefully resulting in a single [`Calculation::Value`].
    ///
    /// If the formula is malformed, it returns 0.0.
    fn calculate(mut calc: Vec<Calculation>) -> f64 {
        // Handle unary negation
        for i in 0..calc.len().saturating_sub(1) {
            if let Calculation::Subtract = calc[i] {
                if let Calculation::Value(value) = calc[i + 1] {
                    // Negation is unary if it occurs at the start of a calculation, or after another operator.
                    if i == 0 || !calc[i - 1].is_value() {
                        calc.splice(i..=i + 1, vec![Calculation::Value(-value)]);
                    }
                }
            }
        }

        // Handle multiply and divide.
        // Loop from 1 to len-1 (exclusive) in order to only catch operators that are between other indices
        let mut i = 1;
        while i < calc.len().saturating_sub(1) {
            if let Calculation::Value(value1) = calc[i - 1] {
                if let Calculation::Value(value2) = calc[i + 1] {
                    #[allow(clippy::match_on_vec_items)] // guaranteed by while condition
                    match calc[i] {
                        Calculation::Multiply => {
                            calc.splice(i - 1..=i + 1, vec![Calculation::Value(value1 * value2)]);
                            i -= 1;
                        }
                        Calculation::Divide(loc) => {
                            if value2 == 0.0 {
                                let msg = "dividing by zero";
                                err(ErrorKey::LocalValues).msg(msg).loc(loc).push();
                            } else {
                                calc.splice(
                                    i - 1..=i + 1,
                                    vec![Calculation::Value(value1 / value2)],
                                );
                                i -= 1;
                            }
                        }
                        _ => (),
                    }
                }
            }
            i += 1;
        }

        // Handle add and subtract.
        // Loop from 1 to len-1 (exclusive) in order to only catch operators that are between other indices
        let mut i = 1;
        while i < calc.len().saturating_sub(1) {
            if let Calculation::Value(value1) = calc[i - 1] {
                if let Calculation::Value(value2) = calc[i + 1] {
                    #[allow(clippy::match_on_vec_items)] // guaranteed by while condition
                    match calc[i] {
                        Calculation::Add => {
                            calc.splice(i - 1..=i + 1, vec![Calculation::Value(value1 + value2)]);
                            i -= 1;
                        }
                        Calculation::Subtract => {
                            calc.splice(i - 1..=i + 1, vec![Calculation::Value(value1 - value2)]);
                            i -= 1;
                        }
                        _ => (),
                    }
                }
            }
            i += 1;
        }

        if calc.len() == 1 {
            if let Calculation::Value(value) = calc[0] {
                return value;
            }
        }
        // Whatever went wrong, we've already logged an error about it
        0.0
    }
}

/// A convenience trait to add some methods to [`char`]
#[allow(clippy::wrong_self_convention)]
trait CharExt {
    /// Can the char be part of an unquoted [`Id`](State::Id)?
    fn is_id_char(self) -> bool;
    /// Can the char be part of a local value?
    fn is_local_value_char(self) -> bool;
    /// Can the char be part of a [`Comparator`](State::Comparator)?
    fn is_comparator_char(self) -> bool;
}

impl CharExt for char {
    fn is_id_char(self) -> bool {
        self.is_alphabetic()
            || self.is_ascii_digit()
            // %, [, ] added for parsing .gui files
            || matches!(self, '.' | ':' | '_' | '-' | '&' | '/' | '|' | '\'' | '%' | '[' | ']')
    }

    fn is_local_value_char(self) -> bool {
        self.is_ascii_alphanumeric() || self == '_'
    }

    fn is_comparator_char(self) -> bool {
        matches!(self, '<' | '>' | '!' | '=' | '?')
    }
}

/// Tracks the @-values defined in this file.
/// Values starting with `@` are local to a file, and are evaluated at parse time.
#[derive(Clone, Debug, Default)]
pub struct LocalValues {
    /// @-values defined as numbers. Calculations can be done with these in `@[ ... ]` blocks.
    values: FnvHashMap<String, (f64, &'static str)>,
    /// @-values defined as text. These can be substituted at other locations in the script.
    text: FnvHashMap<String, &'static str>,
}

impl LocalValues {
    /// Get the value of a numeric @-value or numeric literal.
    /// This is used in the [`State::Calculation`] state.
    ///
    /// The [`f64`] representation is lossy compared to the fixed-point numbers used in the script,
    /// but that hasn't been a problem so far.
    // TODO: the interface here is a bit confusing, the way it mixes number parsing with an actual
    // value lookup.
    fn get_value(&self, key: &str) -> Option<f64> {
        // key can be a local macro or a literal numeric value
        self.values.get(key).map(|(v, _)| v).copied().or_else(|| key.parse().ok())
    }

    /// Get the text form of a numeric or text @-value.
    fn get_as_str(&self, key: &str) -> Option<&'static str> {
        if let Some(value) = self.values.get(key) {
            Some(value.1)
        } else {
            self.text.get(key).copied()
        }
    }

    /// Insert a local @-value definition.
    fn insert(&mut self, key: &str, value: &str) {
        let key = key.to_string();
        let value = bump(value);
        if let Ok(num) = value.parse::<f64>() {
            self.values.insert(key, (num, value));
        } else {
            self.text.insert(key, value);
        }
    }
}

/// Bookkeeping for parsing one block.
#[derive(Debug)]
struct ParseLevel {
    /// The [`Block`] under construction.
    block: Block,
    /// The offset of this block's opening `{`
    start: usize,
    /// The current candidate for the key of an upcoming [`Field`](crate::block::Field).
    key: Option<Token>,
    /// The [`Comparator`] that came after the key, if any.
    cmp: Option<(Comparator, Loc)>,
    /// The "tag" for the upcoming sub-block. See [`Block::tag`].
    tag: Option<Token>,
    /// True iff this block (or any of its sub-blocks) contains `$PARAM$` type parameters.
    ///
    /// This triggers special macro processing when the block is complete.
    contains_macro_parms: bool,
}

/// Bookkeeping for the current file being parsed.
///
/// Use [`Parser`] by creating one for the current file (or macro) you want to parse, then lex the
/// file and call a method in the parser for everything you find.
struct Parser {
    /// Bookkeeping for the deepest block being currently parsed.
    current: ParseLevel,
    /// The parent blocks of `current`.
    stack: Vec<ParseLevel>,
    /// A store of local @-values.
    /// Identifiers that start with `@` are local per-file definitions and are processed at parse time.
    local_values: LocalValues,
    /// calculator used to store and calculate local variable calculations.
    calculator: Calculator,
}

impl Parser {
    /// Construct a parser for a block or file starting at `loc`.
    fn new(loc: Loc) -> Self {
        Self {
            current: ParseLevel {
                block: Block::new(loc),
                start: 0,
                key: None,
                cmp: None,
                tag: None,
                contains_macro_parms: false,
            },
            stack: Vec::new(),
            local_values: LocalValues::default(),
            calculator: Calculator::new(),
        }
    }

    /// Register a single [`Token`]. Can be the result of a quoted or unquoted string; no distinction
    /// between them is made after lexing.
    ///
    /// The token may be a local value id (starting with `@`), in which case it is looked up or
    /// inserted in the [`Parser::local_values`] field as appropriate.
    ///
    /// The parser will take care of deciding whether this token is a loose value or part of a [`Field`](crate::block::Field).
    fn token(&mut self, token: Token) {
        // Special case parsing of color = hsv { ... } and camera positions
        if token.is("hsv")
            || token.is("rgb")
            || token.is("hsv360")
            || token.is("cylindrical")
            || token.is("cartesian")
        {
            self.current.tag = Some(token);
            return;
        }

        if self.stack.is_empty() && self.current.contains_macro_parms {
            let msg = "$-substitutions only work inside blocks, not at top level";
            err(ErrorKey::ParseError).msg(msg).loc(&token).push();
            self.current.contains_macro_parms = false;
        }

        if let Some(key) = self.current.key.take() {
            if let Some((cmp, _)) = self.current.cmp.take() {
                if let Some(local_value_key) = key.as_str().strip_prefix('@') {
                    // @local_value_key = ...
                    if local_value_key.is_empty() {
                        let msg = "empty local value key";
                        err(ErrorKey::LocalValues).msg(msg).loc(key).push();
                        return;
                    }

                    if !local_value_key.starts_with(|c: char| c.is_ascii_alphabetic()) {
                        let msg = "local value names must start with an ascii letter";
                        err(ErrorKey::LocalValues).msg(msg).loc(key).push();
                        return;
                    }

                    if !local_value_key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        let msg = "local value names must only contain ascii letters, numbers or underscores";
                        err(ErrorKey::LocalValues).msg(msg).loc(key).push();
                        return;
                    }

                    if let Some(local_value) = token.as_str().strip_prefix('@') {
                        // @local_value_key = @local_value
                        if let Some(value) = self.local_values.get_as_str(local_value) {
                            self.local_values.insert(local_value_key, value);
                        } else {
                            err(ErrorKey::LocalValues)
                                .msg("local value not defined")
                                .loc(&token)
                                .push();
                        }
                    } else {
                        // @localvalue_key = value
                        self.local_values.insert(local_value_key, token.as_str());
                    }
                } else if let Some(local_value) = token.as_str().strip_prefix('@') {
                    // key = @local_value
                    if token.as_str().contains('!') {
                        // Check for a '!' to avoid looking up macros in gui code that uses @icon! syntax
                        self.current.block.add_key_bv(key, cmp, BV::Value(token));
                    } else if let Some(value) = self.local_values.get_as_str(local_value) {
                        let token = Token::from_static_str(value, token.loc);
                        self.current.block.add_key_bv(key, cmp, BV::Value(token));
                    } else {
                        err(ErrorKey::LocalValues)
                            .msg("local value not defined")
                            .loc(&token)
                            .push();
                        self.current.block.add_key_bv(key, cmp, BV::Value(token));
                    }
                } else {
                    self.current.block.add_key_bv(key, cmp, BV::Value(token));
                }
            } else {
                if let Some(local_value) = key.as_str().strip_prefix('@') {
                    // value1 value2 ... @local_value ...
                    if let Some(value) = self.local_values.get_as_str(local_value) {
                        let token = Token::from_static_str(value, key.loc);
                        self.current.block.add_value(token);
                    } else {
                        err(ErrorKey::LocalValues).msg("local value not defined").loc(&key).push();
                        self.current.block.add_value(key);
                    }
                } else {
                    self.current.block.add_value(key);
                }
                self.current.key = Some(token);
            }
        } else {
            self.current.key = Some(token);
        }
    }

    /// Register a sub-block.
    /// The parser will take care of deciding whether it is a loose block or part of a [`Field`](crate::block::Field).
    fn block_value(&mut self, mut block: Block) {
        // Like token(), but block values cannot become keys
        if let Some(tag) = self.current.tag.take() {
            block.tag = Some(Box::new(tag));
        }
        if let Some(key) = self.current.key.take() {
            if let Some((cmp, _)) = self.current.cmp.take() {
                self.current.block.add_key_bv(key, cmp, BV::Block(block));
            } else {
                self.current.block.add_value(key);
                self.current.block.add_block(block);
            }
        } else {
            self.current.block.add_block(block);
        }
    }

    /// Register a comparator, which here is defined as any string consisting of comparator
    /// characters such as `=` or `<`.
    ///
    /// The parser will look up whether it's a valid comparator and log an error if not.
    fn comparator(&mut self, s: &'static str, loc: Loc) {
        let cmp = Comparator::from_str(s).unwrap_or_else(|| {
            let msg = format!("Unrecognized comparator '{s}'");
            err(ErrorKey::ParseError).msg(msg).loc(loc).push();
            Comparator::Equals(Single)
        });

        if self.current.key.is_none() {
            let msg = format!("Unexpected comparator '{s}'");
            err(ErrorKey::ParseError).msg(msg).loc(loc).push();
        } else if let Some((cmp, _)) = self.current.cmp {
            // Double comparator is valid in macro parameters, such as `OPERATOR = >=`.
            if cmp == Comparator::Equals(Single) {
                let token = Token::from_static_str(s, loc);
                self.token(token);
            } else {
                let msg = &format!("Double comparator '{s}'");
                err(ErrorKey::ParseError).msg(msg).loc(loc).push();
            }
        } else {
            self.current.cmp = Some((cmp, loc));
        }
    }

    /// Internal function to register that a possible [`Field`](crate::block::Field) is no longer possible, and the
    /// stored key, if any, should be registered as a loose value.
    fn end_assign(&mut self) {
        if let Some(key) = self.current.key.take() {
            if let Some((_, cmp_loc)) = self.current.cmp.take() {
                err(ErrorKey::ParseError).msg("comparator without value").loc(cmp_loc).push();
            }
            if let Some(local_value) = key.as_str().strip_prefix('@') {
                if let Some(value) = self.local_values.get_as_str(local_value) {
                    let token = Token::from_static_str(value, key.loc);
                    self.current.block.add_value(token);
                } else {
                    err(ErrorKey::LocalValues).msg("local value not defined").loc(&key).push();
                    self.current.block.add_value(key);
                }
            } else {
                self.current.block.add_value(key);
            }
        }
    }

    /// Register the start of a new block.
    fn open_brace(&mut self, loc: Loc, offset: usize) {
        let mut new_level = ParseLevel {
            block: Block::new(loc),
            start: offset,
            key: None,
            cmp: None,
            tag: None,
            contains_macro_parms: false,
        };
        swap(&mut new_level, &mut self.current);
        self.stack.push(new_level);
    }

    /// Register the end of a block.
    ///
    /// `content` and `offset` are needed to construct the special macro storage in the [`Block`],
    /// if appropriate.
    // TODO: maybe two versions, one for macro parsing and one for file parsing, so that the file
    // parsing version can take a &'static str and construct a token from that.
    fn close_brace(&mut self, loc: Loc, content: &'static str, offset: usize) {
        self.end_assign();
        if let Some(mut prev_level) = self.stack.pop() {
            swap(&mut self.current, &mut prev_level);
            if self.stack.is_empty() && prev_level.contains_macro_parms {
                // skip the { } in constructing s
                let s = &content[prev_level.start + 1..offset - 1];
                let mut loc = prev_level.block.loc;
                loc.column += 1;
                let token = Token::from_static_str(s, loc);
                prev_level.block.source = Some(split_macros(&token, &self.local_values));
            } else {
                self.current.contains_macro_parms |= prev_level.contains_macro_parms;
            }
            self.block_value(prev_level.block);
            if loc.column == 1 && !self.stack.is_empty() {
                let info = "This closing brace is at the start of a line but does not end a top-level item.";
                warn(ErrorKey::BracePlacement)
                    .msg("possible brace error")
                    .info(info)
                    .loc(loc)
                    .push();
            }
        } else {
            err(ErrorKey::BraceError).msg("unexpected }").loc(loc).push();
        }
    }

    /// Register the end of file (or end of parsing, when re-parsing macros).
    /// Return the resulting [`Block`].
    ///
    /// The game engine parser is very forgiving, and this parser tries to emulate it, so parsing
    /// never fails and always returns a block.
    fn eof(mut self) -> Block {
        self.end_assign();
        while let Some(mut prev_level) = self.stack.pop() {
            err(ErrorKey::BraceError)
                .msg("opening { was never closed")
                .loc(self.current.block.loc)
                .push();
            swap(&mut self.current, &mut prev_level);
            self.block_value(prev_level.block);
        }
        self.current.block
    }
}

/// Found a character that doesn't fit anywhere. Log an error message about it and then ignore it.
fn unknown_char(c: char, loc: Loc) {
    let msg = format!("Unrecognized character {c}");
    err(ErrorKey::ParseError).msg(msg).loc(loc).push();
}

/// Found a ^Z character. Log an error message about it and then ignore it. The lexer should
/// stop after calling this method.
fn control_z(loc: Loc, at_end: bool) {
    let msg = "^Z in file";
    if at_end {
        let info = "This control code means stop reading the file here, which will cause trouble if you add more code later.";
        untidy(ErrorKey::ParseError).msg(msg).info(info).loc(loc).push();
    } else {
        let info = "This control code means stop reading the file here. Nothing that follows will be read.";
        err(ErrorKey::ParseError).msg(msg).info(info).loc(loc).push();
    }
}

enum Id {
    Uninit,
    Borrowed(&'static str, usize, usize, Loc),
    Owned(String, Loc),
}

impl Default for Id {
    fn default() -> Self {
        Self::Uninit
    }
}

impl Id {
    fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn set(&mut self, str: &'static str, index: usize, loc: Loc) {
        *self = Self::Borrowed(str, index, index, loc);
    }

    /// **ASSERT**: the char must match the char starting at the end index of the borrowed string (if applicable).
    fn add_char(&mut self, c: char) {
        match *self {
            Self::Uninit => unreachable!(),
            Self::Borrowed(str, start, end, loc) if end == str.len() => {
                let mut string = str[start..].to_owned();
                string.push(c);
                *self = Self::Owned(string, loc);
            }
            Self::Borrowed(_str, _, ref mut end, _) => {
                // ASSERT: _str[*end..].starts_with(c)
                *end += c.len_utf8();
            }
            Self::Owned(ref mut string, _) => string.push(c),
        }
    }

    #[inline]
    fn push(&mut self, c: char, str: &'static str, index: usize, loc: Loc) {
        if matches!(self, Self::Uninit) {
            self.set(str, index, loc);
        }
        self.add_char(c);
    }

    fn take_to_token(&mut self) -> Token {
        match take(self) {
            Id::Uninit => unreachable!(),
            Id::Borrowed(str, start, end, loc) => Token::from_static_str(&str[start..end], loc),
            Id::Owned(string, loc) => Token::new(&string, loc),
        }
    }
}

/// Re-parse a macro (which is a scripted effect, trigger, or modifier that uses $ parameters)
/// after argument substitution. A full re-parse is needed because the game engine allows tricks
/// such as passing `#` as a macro argument in order to comment out the rest of a line.
pub fn parse_pdx_macro(inputs: &[Token]) -> Block {
    let blockloc = inputs[0].loc;
    let mut parser = Parser::new(blockloc);
    let mut state = State::Neutral;
    let mut current_id = Id::new();

    for token in inputs {
        let content = token.as_str();
        let mut loc = token.loc;

        for (i, c) in content.char_indices() {
            match state {
                State::Neutral => match c {
                    _ if c.is_ascii_whitespace() => (),
                    _ if c.is_id_char() => {
                        current_id.push(c, content, i, loc);
                        state = State::Id;
                    }
                    _ if c.is_comparator_char() => {
                        current_id.push(c, content, i, loc);
                        state = State::Comparator;
                    }
                    '{' => parser.open_brace(loc, i),
                    '}' => parser.close_brace(loc, content, i),
                    '#' => state = State::Comment,
                    '"' => state = State::QString,
                    _ => unknown_char(c, loc),
                },
                State::Comment => {
                    if c == '\n' {
                        state = State::Neutral;
                    }
                }
                State::QString => match c {
                    '"' => {
                        let token = if matches!(current_id, Id::Uninit) {
                            // empty quoted string
                            Token::from_static_str("", loc)
                        } else {
                            current_id.take_to_token()
                        };
                        parser.token(token);
                        state = State::Neutral;
                    }
                    '\n' => {
                        warn(ErrorKey::ParseError).msg("quoted string not closed").loc(loc).push();
                        let token = if matches!(current_id, Id::Uninit) {
                            // empty quoted string
                            Token::from_static_str("", loc)
                        } else {
                            current_id.take_to_token()
                        };
                        parser.token(token);
                        state = State::Neutral;
                    }
                    _ => current_id.push(c, content, i, loc),
                },
                State::Id => {
                    if c.is_id_char() {
                        current_id.push(c, content, i, loc);
                    } else {
                        parser.token(current_id.take_to_token());

                        match c {
                            _ if c.is_ascii_whitespace() => state = State::Neutral,
                            _ if c.is_comparator_char() => {
                                current_id.push(c, content, i, loc);
                                state = State::Comparator;
                            }
                            '{' => {
                                parser.open_brace(loc, i);
                                state = State::Neutral;
                            }
                            '}' => {
                                parser.close_brace(loc, content, i);
                                state = State::Neutral;
                            }
                            '#' => state = State::Comment,
                            '"' => state = State::QString,
                            ';' => state = State::Neutral,
                            _ => {
                                unknown_char(c, loc);
                                state = State::Neutral;
                            }
                        }
                    }
                }
                State::Comparator => {
                    if c.is_comparator_char() {
                        current_id.push(c, content, i, loc);
                    } else {
                        let token = current_id.take_to_token();
                        parser.comparator(token.as_str(), token.loc);

                        match c {
                            _ if c.is_ascii_whitespace() => {
                                state = State::Neutral;
                            }
                            _ if c.is_id_char() => {
                                current_id.push(c, content, i, loc);
                                state = State::Id;
                            }
                            '{' => {
                                parser.open_brace(loc, i);
                                state = State::Neutral;
                            }
                            '}' => {
                                parser.close_brace(loc, content, i);
                                state = State::Neutral;
                            }
                            '#' => state = State::Comment,
                            '"' => state = State::QString,
                            _ => {
                                unknown_char(c, loc);
                                state = State::Neutral;
                            }
                        }
                    }
                }
                // All of these should have been processed in `split_macros`
                State::Macro | State::LocalValue | State::Calculation(_) => unreachable!(),
            }

            match c {
                CONTROL_Z => {
                    control_z(loc, content[i + 1..].trim().is_empty());
                    break;
                }
                '\n' => {
                    loc.column = 1;
                    loc.line += 1;
                }
                _ => loc.column += 1,
            }
        }
    }

    // Deal with state at end of file
    if !matches!(current_id, Id::Uninit) {
        let token = current_id.take_to_token();
        match state {
            State::QString => {
                err(ErrorKey::ParseError).msg("Quoted string not closed").loc(&token).push();
                parser.token(token);
            }
            State::Id => {
                parser.token(token);
            }
            State::Comparator => {
                parser.comparator(token.as_str(), token.loc);
            }
            _ => (),
        }
    }
    parser.eof()
}

/// Parse a whole file into a `Block`.
///
/// There is a lot of code duplication between this function and [`parse_pdx_macro`], but it's for a
/// good cause: this function uses the fact that all the input is in one big string to construct
/// [`Token`] objects that are just references into that string. It's much faster that way and uses
/// less memory.
#[allow(clippy::module_name_repetitions)]
fn parse_pdx(entry: &FileEntry, content: &'static str) -> Block {
    let mut loc = Loc::from(entry);
    let mut parser = Parser::new(loc);
    loc.line = 1;
    loc.column = 1;
    let mut state = State::Neutral;
    let mut index_loc = IndexLoc(0, loc);

    for (i, c) in content.char_indices() {
        match state {
            State::Neutral => match c {
                _ if c.is_ascii_whitespace() => (),
                _ if c.is_id_char() => {
                    index_loc = IndexLoc(i, loc);
                    state = State::Id;
                }
                _ if c.is_comparator_char() => {
                    index_loc = IndexLoc(i, loc);
                    state = State::Comparator;
                }
                '{' => parser.open_brace(loc, i),
                '}' => parser.close_brace(loc, content, i),
                '#' => state = State::Comment,
                '"' => {
                    index_loc = IndexLoc(i, loc).next();
                    state = State::QString;
                }
                '$' => {
                    parser.current.contains_macro_parms = true;
                    index_loc = IndexLoc(i, loc).next();
                    state = State::Macro;
                }
                '@' => {
                    index_loc = IndexLoc(i, loc);
                    state = State::LocalValue;
                }
                _ => unknown_char(c, loc),
            },
            State::Comment => {
                if c == '\n' {
                    state = State::Neutral;
                }
            }
            State::QString => match c {
                '"' => {
                    let token = Token::from_static_str(&content[index_loc.0..i], index_loc.1);
                    parser.token(token);
                    state = State::Neutral;
                }
                '\n' => {
                    warn(ErrorKey::ParseError).msg("quoted string not closed").loc(loc).push();
                    let token = Token::from_static_str(&content[index_loc.0..i], index_loc.1);
                    parser.token(token);
                    state = State::Neutral;
                }
                _ => (),
            },
            State::Id => {
                if c.is_id_char() {
                } else if c == '$' {
                    parser.current.contains_macro_parms = true;
                } else {
                    let token = Token::from_static_str(&content[index_loc.0..i], index_loc.1);
                    parser.token(token);

                    match c {
                        _ if c.is_ascii_whitespace() => state = State::Neutral,
                        _ if c.is_comparator_char() => {
                            index_loc = IndexLoc(i, loc);
                            state = State::Comparator;
                        }
                        '{' => {
                            parser.open_brace(loc, i);
                            state = State::Neutral;
                        }
                        '}' => {
                            parser.close_brace(loc, content, i);
                            state = State::Neutral;
                        }
                        '#' => state = State::Comment,
                        '"' => {
                            index_loc = IndexLoc(i, loc).next();
                            state = State::QString;
                        }
                        ';' => state = State::Neutral,
                        _ => {
                            unknown_char(c, loc);
                            state = State::Neutral;
                        }
                    }
                }
            }
            State::Macro => match c {
                _ if c.is_id_char() => (),
                '$' => {
                    let token = Token::from_static_str(&content[index_loc.0..i], index_loc.1);
                    parser.token(token);
                    index_loc = IndexLoc(i, loc).next();
                    state = State::Neutral;
                }
                _ => {
                    unknown_char(c, loc);
                    state = State::Neutral;
                }
            },
            State::LocalValue => match c {
                _ if c.is_local_value_char() => (),
                '[' => {
                    if index_loc.0 + 1 != i {
                        let msg = "not in @[...] form for local value calculation";
                        err(ErrorKey::LocalValues).msg(msg).loc(loc).push();
                    }
                    state = State::Calculation(None);
                }
                _ => {
                    let token = Token::from_static_str(&content[index_loc.0..i], index_loc.1);
                    parser.token(token);

                    match c {
                        _ if c.is_ascii_whitespace() => state = State::Neutral,
                        _ if c.is_comparator_char() => {
                            index_loc = IndexLoc(i, loc);
                            state = State::Comparator;
                        }
                        '{' => {
                            parser.open_brace(loc, i);
                            state = State::Neutral;
                        }
                        '}' => {
                            parser.close_brace(loc, content, i);
                            state = State::Neutral;
                        }
                        '#' => state = State::Comment,
                        ';' => state = State::Neutral,
                        '$' => {
                            parser.current.contains_macro_parms = true;
                            index_loc = IndexLoc(i, loc).next();
                            state = State::Macro;
                        }
                        '@' => {
                            index_loc = IndexLoc(i, loc).next();
                            state = State::LocalValue;
                        }
                        _ => {
                            unknown_char(c, loc);
                            state = State::Neutral;
                        }
                    }
                }
            },
            State::Calculation(current_value) => {
                let calculator = &mut parser.calculator;
                if c.is_ascii_whitespace() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | ']') {
                    if let Some((start_offset, start_loc)) = current_value {
                        calculator.next(&content[start_offset..i], &parser.local_values, start_loc);
                        state = State::Calculation(None);
                    }
                }

                match c {
                    _ if c.is_ascii_whitespace() => (),
                    ']' => {
                        let token = Token::new(&calculator.result().to_string(), index_loc.1);
                        parser.token(token);
                        state = State::Neutral;
                    }
                    _ if c.is_local_value_char() => {
                        if current_value.is_none() {
                            state = State::Calculation(Some((i, loc)));
                        }
                    }
                    // `@[.5 + local_value]` is verified to work
                    '.' => {
                        if current_value.is_none() {
                            state = State::Calculation(Some((i, loc)));
                        }
                    }
                    '+' => calculator.op(Calculation::Add, loc),
                    '-' => calculator.op(Calculation::Subtract, loc),
                    '*' => calculator.op(Calculation::Multiply, loc),
                    '/' => calculator.op(Calculation::Divide(loc), loc),
                    '(' => calculator.push(loc),
                    ')' => calculator.pop(loc),
                    _ => {
                        unknown_char(c, loc);
                        state = State::Neutral;
                    }
                }
            }
            State::Comparator => {
                if c.is_comparator_char() {
                } else {
                    parser.comparator(&content[index_loc.0..i], index_loc.1);

                    match c {
                        _ if c.is_ascii_whitespace() => {
                            state = State::Neutral;
                        }
                        _ if c.is_id_char() => {
                            index_loc = IndexLoc(i, loc);
                            state = State::Id;
                        }
                        '{' => {
                            parser.open_brace(loc, i);
                            state = State::Neutral;
                        }
                        '}' => {
                            parser.close_brace(loc, content, i);
                            state = State::Neutral;
                        }
                        '#' => state = State::Comment,
                        '"' => {
                            index_loc = IndexLoc(i, loc).next();
                            state = State::QString;
                        }
                        '$' => {
                            parser.current.contains_macro_parms = true;
                            index_loc = IndexLoc(i, loc).next();
                            state = State::Macro;
                        }
                        '@' => {
                            index_loc = IndexLoc(i, loc).next();
                            state = State::LocalValue;
                        }
                        _ => {
                            unknown_char(c, loc);
                            state = State::Neutral;
                        }
                    }
                }
            }
        }

        match c {
            CONTROL_Z => {
                control_z(loc, content[i + 1..].trim().is_empty());
                break;
            }
            '\n' => {
                loc.column = 1;
                loc.line += 1;
            }
            _ => loc.column += 1,
        }
    }

    // Deal with state at end of file
    match state {
        State::QString | State::Id | State::Macro | State::LocalValue => {
            let token = Token::from_static_str(&content[index_loc.0..], index_loc.1);
            match state {
                State::QString => {
                    err(ErrorKey::ParseError).msg("quoted string not closed").loc(&token).push();
                }
                State::Macro => {
                    err(ErrorKey::ParseError).msg("macro not closed by `$`").loc(&token).push();
                }
                _ => (),
            }
            parser.token(token);
        }
        State::Calculation(current_value) => {
            let calculator = &mut parser.calculator;
            if let Some((start_offset, start_loc)) = current_value {
                calculator.next(&content[start_offset..], &parser.local_values, start_loc);
            }
            let token = Token::new(&calculator.result().to_string(), index_loc.1);
            err(ErrorKey::ParseError)
                .msg("local value calculation not closed by `]`")
                .loc(&token)
                .push();
            parser.token(token);
        }
        State::Comparator => {
            parser.comparator(&content[index_loc.0..], index_loc.1);
        }
        _ => (),
    }

    parser.eof()
}

/// Parse the content associated with the [`FileEntry`].
pub fn parse_pdx_file(entry: &FileEntry, content: String) -> Block {
    let content = leak(content);
    parse_pdx(entry, content)
}

/// Parse a string into a [`Block`]. This function is meant for use by the validator itself, to
/// allow it to load game description data from internal strings that are in pdx script format.
pub fn parse_pdx_internal(input: &'static str, desc: &str) -> Block {
    let entry = FileEntry::new(PathBuf::from(desc), FileKind::Internal, PathBuf::from(desc));
    parse_pdx(&entry, input)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Kinds of [`MacroComponent`].
pub enum MacroComponentKind {
    Source,
    LocalValue,
    Macro,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Macro components output from [`split_macros`].
pub struct MacroComponent {
    kind: MacroComponentKind,
    token: Token,
}

impl MacroComponent {
    pub fn kind(&self) -> MacroComponentKind {
        self.kind
    }

    pub fn token(&self) -> &Token {
        &self.token
    }
}

/// Split a block that contains macro parameters (represented here as a [`Token`] containing its
/// source script) into [`MacroComponent`].
///
/// Having this available will speed up macro re-parsing later.
///
/// The function is aware of comments and quoted strings and will avoid detecting macro parameters
/// inside those.
fn split_macros(token: &Token, local_values: &LocalValues) -> Vec<MacroComponent> {
    #[derive(Debug, Clone, Copy)]
    enum State {
        Neutral,
        QString,
        Comment,
        LocalValue,
        Calculation(Option<IndexLoc>),
        Macro,
    }
    let content = token.as_str();
    let mut loc = token.loc;

    let mut calculator = Calculator::new();
    let mut vec = Vec::new();
    let mut index_loc = IndexLoc(0, loc);
    let mut state = State::Neutral;

    for (i, c) in content.char_indices() {
        match state {
            State::Neutral => match c {
                '#' => state = State::Comment,
                '"' => state = State::QString,
                '$' => {
                    vec.push(MacroComponent {
                        kind: MacroComponentKind::Source,
                        token: token.subtoken(index_loc.0..i, index_loc.1),
                    });
                    index_loc = IndexLoc(i, loc).next();
                    state = State::Macro;
                }
                '@' => {
                    vec.push(MacroComponent {
                        kind: MacroComponentKind::Source,
                        token: token.subtoken(index_loc.0..i, index_loc.1),
                    });
                    index_loc = IndexLoc(i, loc);
                    state = State::LocalValue;
                }
                _ => (),
            },
            State::Comment => {
                if c == '\n' {
                    state = State::Neutral;
                }
            }
            State::QString => match c {
                '$' => {
                    let msg = "use of `$` inside quotes may cause undefined behaviours";
                    warn(ErrorKey::ParseError).msg(msg).loc(loc).push();
                }
                '\n' | '"' => state = State::Neutral,
                _ => (),
            },
            State::LocalValue => match c {
                _ if c.is_local_value_char() => (),
                '[' => {
                    if index_loc.0 + 1 == i {
                        calculator.start();
                        state = State::Calculation(None);
                    } else {
                        let msg = "not in @[...] form for local value calculation";
                        err(ErrorKey::LocalValues).msg(msg).loc(loc).push();
                    }
                }
                _ => {
                    let str = &content[index_loc.0 + 1..i];
                    if let Some(value) = local_values.get_as_str(str) {
                        let token = Token::from_static_str(value, index_loc.1);
                        vec.push(MacroComponent { kind: MacroComponentKind::LocalValue, token });
                    } else {
                        let err_token = Token::new(str, index_loc.1);
                        err(ErrorKey::LocalValues)
                            .msg("local value not defined")
                            .loc(err_token)
                            .push();
                    }
                    index_loc = IndexLoc(i, loc);
                    match c {
                        _ if c.is_ascii_whitespace() => state = State::Neutral,
                        '#' => state = State::Comment,
                        '"' => state = State::QString,
                        '$' => {
                            index_loc = index_loc.next();
                            state = State::Macro;
                        }
                        '@' => state = State::LocalValue,
                        _ => {
                            unknown_char(c, loc);
                            state = State::Neutral;
                        }
                    }
                }
            },
            State::Calculation(current_value) => {
                if c.is_ascii_whitespace() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | ']') {
                    if let Some(IndexLoc(start_offset, start_loc)) = current_value {
                        // end of current value
                        calculator.next(&content[start_offset..i], local_values, start_loc);
                        state = State::Calculation(None);
                    }
                }

                match c {
                    _ if c.is_ascii_whitespace() => (),
                    ']' => {
                        let token = Token::new(&calculator.result().to_string(), index_loc.1);
                        vec.push(MacroComponent { kind: MacroComponentKind::LocalValue, token });
                        index_loc = IndexLoc(i, loc).next();
                        state = State::Neutral;
                    }
                    _ if c.is_local_value_char() => {
                        if current_value.is_none() {
                            state = State::Calculation(Some(IndexLoc(i, loc)));
                        }
                    }
                    '.' => {
                        // `@[.5 + local_value]` is verified to work
                        if current_value.is_none() {
                            state = State::Calculation(Some(IndexLoc(i, loc)));
                        }
                    }
                    '+' => calculator.op(Calculation::Add, loc),
                    '-' => calculator.op(Calculation::Subtract, loc),
                    '*' => calculator.op(Calculation::Multiply, loc),
                    '/' => calculator.op(Calculation::Divide(loc), loc),
                    '(' => calculator.push(loc),
                    ')' => calculator.pop(loc),
                    _ => unknown_char(c, loc),
                }
            }
            State::Macro => match c {
                _ if c.is_id_char() => (),
                '$' => {
                    if index_loc.0 == i {
                        err(ErrorKey::ParseError).msg("empty macro").loc(index_loc.1).push();
                    } else {
                        vec.push(MacroComponent {
                            kind: MacroComponentKind::Macro,
                            token: token.subtoken(index_loc.0..i, index_loc.1),
                        });
                    }
                    index_loc = IndexLoc(i, loc).next();
                    state = State::Neutral;
                }
                _ => unknown_char(c, loc),
            },
        }

        match c {
            CONTROL_Z => {
                control_z(loc, content[i + 1..].trim().is_empty());
                break;
            }
            '\n' => {
                loc.column = 1;
                loc.line += 1;
            }
            _ => loc.column += 1,
        }
    }

    match state {
        State::Macro => {
            let mut err_loc = index_loc.1;
            // point to the opening '$'
            err_loc.column -= 1;
            fatal(ErrorKey::ParseError).msg("macro not closed by '$'").loc(err_loc).push();
        }
        State::Calculation(_) => {
            // point to the '['
            fatal(ErrorKey::ParseError)
                .msg("local value calculation not closed by ']'")
                .loc(index_loc.1)
                .push();
        }
        State::LocalValue => {
            let str = &content[index_loc.0..];
            if let Some(value) = local_values.get_as_str(str) {
                let token = Token::from_static_str(value, index_loc.1);
                vec.push(MacroComponent { kind: MacroComponentKind::LocalValue, token });
            } else {
                let err_token = Token::new(str, index_loc.1);
                err(ErrorKey::LocalValues).msg("local value not defined").loc(err_token).push();
            }
        }
        _ => {
            vec.push(MacroComponent {
                kind: MacroComponentKind::Source,
                token: token.subtoken(index_loc.0.., index_loc.1),
            });
        }
    }
    vec
}
