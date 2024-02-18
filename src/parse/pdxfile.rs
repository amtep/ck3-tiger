//! Parses a Pdx script file into a [`Block`].
//!
//! The main entry points are [`parse_pdx`] and [`parse_pdx_macro`].

use std::mem::{swap, take};
use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::block::Eq::Single;
use crate::block::{Block, Comparator, BV};
use crate::fileset::{FileEntry, FileKind};
use crate::report::{err, error, fatal, old_warn, untidy, warn, warn_info, ErrorKey};
use crate::token::{leak, Loc, Token};

/// ^Z is by convention an end-of-text marker, and the game engine treats it as such.
const CONTROL_Z: char = '\u{001A}';

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
    /// Parsing a `@[ ... ]` local value calculation.
    Calculation(Option<usize>),
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

    /// Register a part of a calculation, either an operator or a [`Value`](`Calculation::Value`).
    fn op(&mut self, op: Calculation, loc: Loc) {
        if let Some(Calculation::Value(_)) = self.current.last() {
            self.current.push(op);
        } else if let Calculation::Subtract = op {
            // accept negation
            self.current.push(op);
        } else {
            let msg = "operator `{op}` without left-hand value";
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
                                error(loc, ErrorKey::LocalValues, msg);
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
            || self == '.'
            || self == ':'
            || self == '_'
            || self == '-'
            || self == '&'
            || self == '/'
            || self == '|'
            || self == '\''
            || self == '%' // added for parsing .gui files
            || self == '[' // added for parsing .gui files
            || self == ']' // added for parsing .gui files
    }

    fn is_local_value_char(self) -> bool {
        self.is_ascii_alphanumeric() || self == '_' || self == '.'
    }

    fn is_comparator_char(self) -> bool {
        self == '<' || self == '>' || self == '!' || self == '=' || self == '?'
    }
}

/// Tracks the @-values defined in this file.
/// Values starting with `@` are local to a file, and are evaluated at parse time.
#[derive(Clone, Debug, Default)]
pub struct LocalValues {
    /// @-values defined as numbers. Calculations can be done with these in `@[ ... ]` blocks.
    values: FnvHashMap<String, f64>,
    /// @-values defined as text. These can be substituted at other locations in the script.
    text: FnvHashMap<String, String>,
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
        self.values.get(key).copied().or_else(|| key.parse().ok())
    }

    /// Get the text form of a numeric or text @-value.
    fn get_as_string(&self, key: &str) -> Option<String> {
        if let Some(value) = self.values.get(key) {
            Some(value.to_string())
        } else {
            self.text.get(key).map(ToString::to_string)
        }
    }

    /// Insert a local @-value definition.
    fn insert(&mut self, key: &str, value: &str) {
        if let Ok(value) = value.parse::<f64>() {
            self.values.insert(key.to_string(), value);
        } else {
            self.text.insert(key.to_string(), value.to_string());
        }
    }
}

/// Bookkeeping for parsing one block.
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
            error(&token, ErrorKey::ParseError, msg);
            self.current.contains_macro_parms = false;
        }

        if let Some(key) = self.current.key.take() {
            if let Some((cmp, _)) = self.current.cmp.take() {
                if let Some(local_value_key) = key.as_str().strip_prefix('@') {
                    // @local_value_key = ...
                    if key.as_str().starts_with(|c: char| c.is_ascii_alphabetic()) {
                        if key.as_str().chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            if let Some(local_value) = token.as_str().strip_prefix('@') {
                                // @local_value_key = @local_value
                                if let Some(value) = self.local_values.get_as_string(local_value) {
                                    self.local_values.insert(local_value_key, &value);
                                } else {
                                    error(token, ErrorKey::LocalValues, "local value not defined");
                                }
                            } else {
                                // @localvalue_key = value
                                self.local_values.insert(local_value_key, token.as_str());
                            }
                        } else {
                            let msg = "local value names must only contain ascii letters, numbers or underscores";
                            err(ErrorKey::LocalValues).msg(msg).loc(key).push();
                        }
                    } else {
                        let msg = "local value names must start with an ascii letter";
                        err(ErrorKey::LocalValues).msg(msg).loc(key).push();
                    }
                } else if let Some(local_value) = token.as_str().strip_prefix('@') {
                    // key = @local_value
                    if token.as_str().contains('!') {
                        // Check for a '!' to avoid looking up macros in gui code that uses @icon! syntax
                        self.current.block.add_key_bv(key, cmp, BV::Value(token));
                    } else if let Some(value) = self.local_values.get_as_string(local_value) {
                        let token = Token::new(&value, token.loc);
                        self.current.block.add_key_bv(key, cmp, BV::Value(token));
                    } else {
                        error(&token, ErrorKey::LocalValues, "local value not defined");
                        self.current.block.add_key_bv(key, cmp, BV::Value(token));
                    }
                } else {
                    self.current.block.add_key_bv(key, cmp, BV::Value(token));
                }
            } else {
                if let Some(local_value) = key.as_str().strip_prefix('@') {
                    // value1 value2 ... @local_value ...
                    if let Some(value) = self.local_values.get_as_string(local_value) {
                        let token = Token::new(&value, key.loc);
                        self.current.block.add_value(token);
                    } else {
                        error(&key, ErrorKey::LocalValues, "local value not defined");
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
    fn comparator(&mut self, s: &str, loc: Loc) {
        let cmp = Comparator::from_str(s).unwrap_or_else(|| {
            let msg = format!("Unrecognized comparator '{s}'");
            error(loc, ErrorKey::ParseError, &msg);
            Comparator::Equals(Single)
        });

        if self.current.key.is_none() {
            let msg = format!("Unexpected comparator '{s}'");
            error(loc, ErrorKey::ParseError, &msg);
        } else if let Some((cmp, _)) = self.current.cmp {
            // Double comparator is valid in macro parameters, such as `OPERATOR = >=`.
            if cmp == Comparator::Equals(Single) {
                let token = Token::new(s, loc);
                self.token(token);
            } else {
                let msg = &format!("Double comparator '{s}'");
                error(loc, ErrorKey::ParseError, msg);
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
                error(cmp_loc, ErrorKey::ParseError, "Comparator without value");
            }
            if let Some(local_value) = key.as_str().strip_prefix('@') {
                if let Some(value) = self.local_values.get_as_string(local_value) {
                    let token = Token::new(&value, key.loc);
                    self.current.block.add_value(token);
                } else {
                    error(&key, ErrorKey::LocalValues, "local value not defined");
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
    fn close_brace(&mut self, loc: Loc, content: &str, offset: usize) {
        self.end_assign();
        if let Some(mut prev_level) = self.stack.pop() {
            swap(&mut self.current, &mut prev_level);
            if self.stack.is_empty() && prev_level.contains_macro_parms {
                // skip the { } in constructing s
                let s = &content[prev_level.start + 1..offset - 1];
                let mut loc = prev_level.block.loc;
                loc.column += 1;
                let token = Token::new(s, prev_level.block.loc);
                prev_level.block.source = Some(split_macros(&token, &self.local_values));
            } else {
                self.current.contains_macro_parms |= prev_level.contains_macro_parms;
            }
            self.block_value(prev_level.block);
            if loc.column == 1 && !self.stack.is_empty() {
                warn_info(loc,
                          ErrorKey::BracePlacement,
                          "possible brace error",
                          "This closing brace is at the start of a line but does not end a top-level item.",
                );
            }
        } else {
            error(loc, ErrorKey::BraceError, "Unexpected }");
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
            error(self.current.block.loc, ErrorKey::BraceError, "Opening { was never closed");
            swap(&mut self.current, &mut prev_level);
            self.block_value(prev_level.block);
        }
        self.current.block
    }
}

/// Found a character that doesn't fit anywhere. Log an error message about it and then ignore it.
fn unknown_char(c: char, loc: Loc) {
    let msg = format!("Unrecognized character {c}");
    error(loc, ErrorKey::ParseError, &msg);
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

/// Re-parse a macro (which is a scripted effect, trigger, or modifier that uses $ parameters)
/// after argument substitution. A full re-parse is needed because the game engine allows tricks
/// such as passing `#` as a macro argument in order to comment out the rest of a line.
// TODO: efficiency could be improved by constructing subtokens if a token is contained completely
// within one of the input tokens.
pub fn parse_pdx_macro(inputs: &[Token]) -> Block {
    let blockloc = inputs[0].loc;
    let mut parser = Parser::new(blockloc);
    let mut state = State::Neutral;
    let mut token_start = blockloc;
    let mut current_id = String::new();

    for token in inputs {
        let content = token.as_str();
        let mut loc = token.loc;

        for (i, c) in content.char_indices() {
            match state {
                State::Neutral => {
                    if c.is_ascii_whitespace() {
                    } else if c == '"' {
                        token_start = loc;
                        token_start.column += 1;
                        state = State::QString;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c.is_comparator_char() {
                        token_start = loc;
                        state = State::Comparator;
                        current_id.push(c);
                    } else if c.is_id_char() || c == '$' {
                        // c == '$' can only happen with extreme shenanigans in the input.
                        // Treat it as just another character.
                        token_start = loc;
                        state = State::Id;
                        current_id.push(c);
                    } else if c == '{' {
                        parser.open_brace(loc, i);
                    } else if c == '}' {
                        parser.close_brace(loc, content, i);
                    } else {
                        unknown_char(c, loc);
                    }
                }
                State::Comment => {
                    if c == '\n' {
                        state = State::Neutral;
                    }
                }
                State::QString => {
                    if c == '"' {
                        let token = Token::new(&take(&mut current_id), token_start);
                        parser.token(token);
                        state = State::Neutral;
                    } else if c == '\n' {
                        old_warn(loc, ErrorKey::ParseError, "Quoted string not closed");
                    } else {
                        current_id.push(c);
                    }
                }
                State::Id => {
                    if c == '$' || c.is_id_char() {
                        current_id.push(c);
                    } else {
                        let token = Token::new(&take(&mut current_id), token_start);
                        parser.token(token);

                        if c.is_comparator_char() {
                            current_id.push(c);
                            token_start = loc;
                            state = State::Comparator;
                        } else if c.is_ascii_whitespace() || c == ';' {
                            // An id followed by ; is silently accepted because it's a common mistake,
                            // and doesn't seem to cause any harm.
                            state = State::Neutral;
                        } else if c == '#' {
                            state = State::Comment;
                        } else if c == '{' {
                            parser.open_brace(loc, i);
                            state = State::Neutral;
                        } else if c == '}' {
                            parser.close_brace(loc, content, i);
                            state = State::Neutral;
                        } else if c == '"' {
                            state = State::QString;
                            token_start = loc;
                            token_start.column += 1;
                        } else {
                            unknown_char(c, loc);
                            state = State::Neutral;
                        }
                    }
                }
                State::Comparator => {
                    if c.is_comparator_char() {
                        current_id.push(c);
                    } else {
                        parser.comparator(&take(&mut current_id), token_start);

                        if c == '"' {
                            token_start = loc;
                            token_start.column += 1;
                            state = State::QString;
                        } else if c.is_id_char() || c == '$' {
                            token_start = loc;
                            state = State::Id;
                            current_id.push(c);
                        } else if c.is_ascii_whitespace() {
                            state = State::Neutral;
                        } else if c == '#' {
                            state = State::Comment;
                        } else if c == '{' {
                            parser.open_brace(loc, i);
                            state = State::Neutral;
                        } else if c == '}' {
                            parser.close_brace(loc, content, i);
                            state = State::Neutral;
                        } else {
                            unknown_char(c, loc);
                            state = State::Neutral;
                        }
                    }
                }
                // No @local_value is possible since they have been all substituted in `split_macros`
                State::Calculation(_) => unreachable!(),
            }

            if c == CONTROL_Z {
                control_z(loc, content[i + 1..].trim().is_empty());
                break;
            } else if c == '\n' {
                loc.line += 1;
                loc.column = 1;
            } else {
                loc.column += 1;
            }
        }
    }

    // Deal with state at end of file
    match state {
        State::QString => {
            let token = Token::new(&current_id, token_start);
            error(&token, ErrorKey::ParseError, "Quoted string not closed");
            parser.token(token);
        }
        State::Id => {
            let token = Token::new(&current_id, token_start);
            parser.token(token);
        }
        State::Comparator => {
            parser.comparator(&current_id, token_start);
        }
        _ => (),
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
    let mut token_start = loc;
    let mut token_start_offset = 0;

    for (i, c) in content.char_indices() {
        match state {
            State::Neutral => {
                if c.is_ascii_whitespace() {
                } else if c == '"' {
                    token_start = loc;
                    token_start.column += 1;
                    token_start_offset = i + 1;
                    state = State::QString;
                } else if c == '#' {
                    state = State::Comment;
                } else if c.is_comparator_char() {
                    token_start = loc;
                    token_start_offset = i;
                    state = State::Comparator;
                } else if c == '@' {
                    // @ can start tokens but is special
                    token_start = loc;
                    token_start_offset = i;
                    state = State::Id;
                } else if c == '$' {
                    parser.current.contains_macro_parms = true;
                    token_start = loc;
                    token_start_offset = i;
                    state = State::Id;
                } else if c.is_id_char() {
                    token_start = loc;
                    token_start_offset = i;
                    state = State::Id;
                } else if c == '{' {
                    parser.open_brace(loc, i);
                } else if c == '}' {
                    parser.close_brace(loc, content, i);
                } else {
                    unknown_char(c, loc);
                }
            }
            State::Comment => {
                if c == '\n' {
                    state = State::Neutral;
                }
            }
            State::QString => {
                if c == '"' {
                    let token =
                        Token::from_static_str(&content[token_start_offset..i], token_start);
                    parser.token(token);
                    state = State::Neutral;
                } else if c == '\n' {
                    old_warn(loc, ErrorKey::ParseError, "Quoted string not closed");
                }
            }
            State::Id => {
                if c == '$' {
                    parser.current.contains_macro_parms = true;
                } else if c == '[' && &content[token_start_offset..i] == "@" {
                    parser.calculator.start();
                    state = State::Calculation(None);
                } else if c.is_id_char() {
                } else {
                    let token =
                        Token::from_static_str(&content[token_start_offset..i], token_start);
                    parser.token(token);

                    if c.is_comparator_char() {
                        token_start = loc;
                        token_start_offset = i;
                        state = State::Comparator;
                    } else if c.is_ascii_whitespace() || c == ';' {
                        // An id followed by ; is silently accepted because it's a common mistake,
                        // and doesn't seem to cause any harm.
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c == '{' {
                        parser.open_brace(loc, i);
                        state = State::Neutral;
                    } else if c == '}' {
                        parser.close_brace(loc, content, i);
                        state = State::Neutral;
                    } else if c == '"' {
                        token_start = loc;
                        token_start.column += 1;
                        token_start_offset = i + 1;
                        state = State::QString;
                    } else {
                        unknown_char(c, loc);
                        state = State::Neutral;
                    }
                }
            }
            State::Calculation(current_value) => {
                let calculator = &mut parser.calculator;
                if matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | ']' | _ if c.is_ascii_whitespace())
                {
                    if let Some(offset) = current_value {
                        calculator.next(&content[offset..i], &parser.local_values, loc);
                        state = State::Calculation(None);
                    }
                }

                match c {
                    _ if c.is_ascii_whitespace() => (),
                    '+' => calculator.op(Calculation::Add, loc),
                    '-' => calculator.op(Calculation::Subtract, loc),
                    '*' => calculator.op(Calculation::Multiply, loc),
                    '/' => calculator.op(Calculation::Divide(loc), loc),
                    '(' => calculator.push(loc),
                    ')' => calculator.pop(loc),
                    ']' => {
                        let token = Token::new(&calculator.result().to_string(), token_start);
                        parser.token(token);
                        state = State::Neutral;
                    }
                    _ if c.is_local_value_char() => {
                        state = State::Calculation(Some(i));
                    }
                    _ => {
                        unknown_char(c, loc);
                        state = State::Neutral;
                    }
                }
            }
            State::Comparator => {
                if c.is_comparator_char() {
                } else {
                    parser.comparator(&content[token_start_offset..i], token_start);

                    if c == '"' {
                        token_start = loc;
                        token_start.column += 1;
                        token_start_offset = i + 1;
                        state = State::QString;
                    } else if c == '@' {
                        // @ can start tokens but is special
                        token_start = loc;
                        token_start_offset = i;
                        state = State::Id;
                    } else if c == '$' {
                        parser.current.contains_macro_parms = true;
                        token_start = loc;
                        token_start_offset = i;
                        state = State::Id;
                    } else if c.is_id_char() {
                        token_start = loc;
                        token_start_offset = i;
                        state = State::Id;
                    } else if c.is_ascii_whitespace() {
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c == '{' {
                        parser.open_brace(loc, i);
                        state = State::Neutral;
                    } else if c == '}' {
                        parser.close_brace(loc, content, i);
                        state = State::Neutral;
                    } else {
                        unknown_char(c, loc);
                        state = State::Neutral;
                    }
                }
            }
        }

        if c == CONTROL_Z {
            control_z(loc, content[i + 1..].trim().is_empty());
            break;
        } else if c == '\n' {
            loc.line += 1;
            loc.column = 1;
        } else {
            loc.column += 1;
        }
    }

    // Deal with state at end of file
    match state {
        State::QString => {
            let token = Token::from_static_str(&content[token_start_offset..], token_start);
            error(&token, ErrorKey::ParseError, "Quoted string not closed");
            parser.token(token);
        }
        State::Id => {
            let token = Token::from_static_str(&content[token_start_offset..], token_start);
            parser.token(token);
        }
        State::Comparator => {
            parser.comparator(&content[token_start_offset..], token_start);
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
/// Macro components outputted from [`split_macros`].
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
// TODO: is it actually correct to ignore macro params in comments and quoted strings? Verify.
fn split_macros(content: &Token, local_values: &LocalValues) -> Vec<MacroComponent> {
    #[derive(Eq, PartialEq)]
    enum State {
        Neutral,
        QString,
        Comment,
        LocalValue,
        Calculation(Option<usize>),
        Macro,
    }
    let mut state = State::Neutral;
    let mut calculator = Calculator::new();
    let mut vec = Vec::new();
    let mut loc = content.loc;
    let mut last_loc = loc;
    let mut last_pos = 0;
    for (i, c) in content.as_str().char_indices() {
        match state {
            State::Comment => {
                if c == '\n' {
                    state = State::Neutral;
                }
            }
            State::QString => {
                if c == '\n' || c == '"' {
                    state = State::Neutral;
                }
            }
            State::Neutral => {
                if c == '#' {
                    state = State::Comment;
                } else if c == '"' {
                    state = State::QString;
                } else if c == '$' || c == '@' {
                    vec.push(MacroComponent {
                        kind: MacroComponentKind::Source,
                        token: content.subtoken(last_pos..i, last_loc),
                    });
                    last_loc = loc;
                    // Skip the current '$' or '@'
                    last_loc.column += 1;
                    last_pos = i + 1;
                    state = if c == '$' { State::Macro } else { State::LocalValue };
                }
            }
            State::LocalValue => {
                if c == '[' {
                    calculator.start();
                    state = State::Calculation(None);
                } else if c.is_ascii_whitespace() {
                    if last_pos != i {
                        let str = &content.as_str()[last_pos..i];
                        if let Some(value) = local_values.get_as_string(str) {
                            let token = Token::new(&value, last_loc);
                            vec.push(MacroComponent {
                                kind: MacroComponentKind::LocalValue,
                                token,
                            });
                        } else {
                            let err_token = Token::new(str, last_loc);
                            err(ErrorKey::LocalValues)
                                .msg("local value not defined")
                                .loc(err_token)
                                .push();
                        }
                    } else {
                        err(ErrorKey::ParseError).msg("empty local value").loc(last_loc).push();
                    }
                    state = State::Neutral;
                } else if !c.is_local_value_char() {
                    unknown_char(c, loc);
                }
            }
            State::Calculation(current_value) => {
                if matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | ']' | _ if c.is_ascii_whitespace())
                {
                    if let Some(offset) = current_value {
                        calculator.next(&content.as_str()[offset..i], local_values, loc);
                        state = State::Calculation(None);
                    }
                }

                match c {
                    _ if c.is_ascii_whitespace() => (),
                    '+' => calculator.op(Calculation::Add, loc),
                    '-' => calculator.op(Calculation::Subtract, loc),
                    '*' => calculator.op(Calculation::Multiply, loc),
                    '/' => calculator.op(Calculation::Divide(loc), loc),
                    '(' => calculator.push(loc),
                    ')' => calculator.pop(loc),
                    ']' => {
                        let token = Token::new(&calculator.result().to_string(), last_loc);
                        vec.push(MacroComponent { kind: MacroComponentKind::LocalValue, token });
                        state = State::Neutral;
                    }
                    _ if c.is_local_value_char() => {
                        state = State::Calculation(Some(i));
                    }
                    _ => {
                        unknown_char(c, loc);
                        state = State::Neutral;
                    }
                }
            }
            State::Macro => {
                if c == '$' {
                    if last_pos != i {
                        vec.push(MacroComponent {
                            kind: MacroComponentKind::Macro,
                            token: content.subtoken(last_pos..i, last_loc),
                        });
                    } else {
                        err(ErrorKey::ParseError).msg("empty macro").loc(last_loc).push();
                    }

                    last_loc = loc;
                    last_loc.column += 1;
                    last_pos = i + 1;
                    state = State::Neutral;
                } else if !c.is_id_char() {
                    unknown_char(c, loc);
                }
            }
        }
        if c == '\n' {
            loc.column = 1;
            loc.line += 1;
        } else {
            loc.column += 1;
        }
    }

    match state {
        State::Macro => {
            let mut err_loc = last_loc;
            // point to the opening '$'
            err_loc.column -= 1;
            fatal(ErrorKey::ParseError).msg("macro not closed by '$'").loc(err_loc).push();
        }
        State::Calculation(_) => {
            // point to the '['
            fatal(ErrorKey::ParseError)
                .msg("local value calculation not closed by ']'")
                .loc(last_loc)
                .push();
        }
        _ => (),
    }

    vec.push(MacroComponent {
        kind: MacroComponentKind::Source,
        token: content.subtoken(last_pos.., last_loc),
    });
    vec
}
