use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::CharIndices;

use crate::block::Comparator;
use crate::block::Eq::Single;
use crate::game::Game;
use crate::parse::pdxfile::{CharExt, Cob};
use crate::report::{err, untidy, warn, ErrorKey};
use crate::token::{Loc, Token};

/// ^Z is by convention an end-of-text marker, and the game engine treats it as such.
const CONTROL_Z: char = '\u{001A}';

#[derive(Debug, Clone)]
pub enum Lexeme {
    General(Token),                // id or "quoted string"
    Comparator(Comparator, Token), // =, ?=, <=, <, etc
    VariableReference(Token),      // @varname
    MacroParam(Token),             // $PARAM$
    BlockStart(Token),             // {
    BlockEnd(Token),               // }
    CalcStart(Token),              // @[
    CalcEnd(Token),                // ]
    OpenParen(Token),              // (
    CloseParen(Token),             // )
    Add(Token),                    // +
    Subtract(Token),               // -
    Multiply(Token),               // *
    Divide(Token),                 // /
    Directive(Directive, Token),   // @:insert etc
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Lexeme::General(token) => write!(f, "value `{token}`"),
            Lexeme::Comparator(_, token) => write!(f, "comparator `{token}`"),
            Lexeme::VariableReference(token) => write!(f, "variable `{token}`"),
            Lexeme::MacroParam(token) => write!(f, "parameter `${token}$`"),
            Lexeme::BlockStart(_) => write!(f, "`{{`"),
            Lexeme::BlockEnd(_) => write!(f, "`}}`"),
            Lexeme::CalcStart(_) => write!(f, "`@[`"),
            Lexeme::CalcEnd(_) => write!(f, "`]`"),
            Lexeme::OpenParen(_) => write!(f, "`(`"),
            Lexeme::CloseParen(_) => write!(f, "`)`"),
            Lexeme::Add(_) => write!(f, "`+`"),
            Lexeme::Subtract(_) => write!(f, "`-`"),
            Lexeme::Multiply(_) => write!(f, "`*`"),
            Lexeme::Divide(_) => write!(f, "`/`"),
            Lexeme::Directive(_, token) => write!(f, "directive `{token}`"),
        }
    }
}

impl Lexeme {
    /// Return the [`Token`] contained in this lexeme.
    pub fn into_token(self) -> Token {
        match self {
            Lexeme::General(token)
            | Lexeme::Comparator(_, token)
            | Lexeme::VariableReference(token)
            | Lexeme::MacroParam(token)
            | Lexeme::BlockStart(token)
            | Lexeme::BlockEnd(token)
            | Lexeme::CalcStart(token)
            | Lexeme::CalcEnd(token)
            | Lexeme::OpenParen(token)
            | Lexeme::CloseParen(token)
            | Lexeme::Add(token)
            | Lexeme::Subtract(token)
            | Lexeme::Multiply(token)
            | Lexeme::Divide(token)
            | Lexeme::Directive(_, token) => token,
        }
    }

    /// Return the [`Loc`] of this lexeme.
    pub fn get_loc(&self) -> Loc {
        match self {
            Lexeme::General(token)
            | Lexeme::Comparator(_, token)
            | Lexeme::VariableReference(token)
            | Lexeme::MacroParam(token)
            | Lexeme::BlockStart(token)
            | Lexeme::BlockEnd(token)
            | Lexeme::CalcStart(token)
            | Lexeme::CalcEnd(token)
            | Lexeme::OpenParen(token)
            | Lexeme::CloseParen(token)
            | Lexeme::Add(token)
            | Lexeme::Subtract(token)
            | Lexeme::Multiply(token)
            | Lexeme::Divide(token)
            | Lexeme::Directive(_, token) => token.loc,
        }
    }

    /// Return the [`Comparator`] of this lexeme.
    /// The parser will only call this for the Comparator lexeme.
    pub fn get_cmp(&self) -> Comparator {
        match self {
            Lexeme::Comparator(cmp, _) => *cmp,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Directive {
    RegisterVariable,
    LoadVariable,
    Define,
    Insert,
    Log,
    // `Assert` is left out because it is never passed to the parser.
}

/// An error type is required by lalrpop, but it will not be used.
/// All errors are reported via the report module and then swallowed.
pub enum LexError {}

impl Display for LexError {
    fn fmt(&self, _: &mut Formatter) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

/// An iterator that produces [`Lexeme`] values on demand.
pub struct Lexer<'input> {
    /// The input is in most cases a single token (a whole file), but when processing macros it can
    /// be a sequence of tokens from different locations.
    /// A specialized lexer for the whole-file case may be worth it for speed.
    inputs: &'input [Token],
    /// The current index into the `inputs` array.
    inputs_index: usize,
    /// Tracking file, line, and column of the current char.
    loc: Loc,
    /// Iterator over the current `inputs` token.
    iter: Peekable<CharIndices<'input>>,
    /// How many nested braces are around the current char.
    /// This is only used to warn about misaligned braces.
    brace_depth: usize,
    /// Is the lexer inside a `@[` calculation?
    /// This restricts the chars allowed in identifiers.
    in_calc: bool,
}

impl<'input> Lexer<'input> {
    pub fn new(inputs: &'input [Token]) -> Self {
        assert!(!inputs.is_empty());

        Lexer {
            inputs,
            inputs_index: 0,
            loc: inputs[0].loc,
            iter: inputs[0].as_str().char_indices().peekable(),
            brace_depth: 0,
            in_calc: false,
        }
    }

    /// Return the current char and its offset in the current input.
    fn peek(&mut self) -> Option<(usize, char)> {
        let p = self.iter.peek();
        if p.is_none() {
            if self.inputs_index + 1 == self.inputs.len() {
                None
            } else {
                self.inputs_index += 1;
                self.iter = self.inputs[self.inputs_index].as_str().char_indices().peekable();
                self.loc = self.inputs[self.inputs_index].loc;
                self.peek()
            }
        } else {
            p.copied()
        }
    }

    /// Advance to the next char.
    fn consume(&mut self) {
        // self.peek advances the inputs_index if needed
        if self.peek().is_some() {
            let (_, c) = self.iter.next().unwrap();
            if c == '\n' {
                self.loc.line += 1;
                self.loc.column = 1;
            } else {
                self.loc.column += 1;
            }
        }
    }

    /// Initialize a [`Cob`] starting at the current char.
    fn start_cob(&mut self) -> Cob {
        let mut cob = Cob::new();
        if let Some((i, _)) = self.peek() {
            cob.set(self.inputs[self.inputs_index].as_str(), i, self.loc);
        }
        cob
    }

    /// Return the offset just beyond the final character in the input.
    fn eof_offset(&self) -> usize {
        self.inputs[self.inputs_index].as_str().len()
    }

    /// Destructively check if there are any non-whitespace characters between here and the end of
    /// the input.
    fn only_whitespace_left(&mut self) -> bool {
        while let Some((_, c)) = self.peek() {
            if !c.is_whitespace() {
                return false;
            }
            self.consume();
        }
        true
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<(usize, Lexeme, usize), LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.peek() {
            match c {
                _ if c.is_ascii_whitespace() => self.consume(),
                _ if !self.in_calc && c.is_id_char() => {
                    // An unquoted token
                    let mut id = self.start_cob();
                    id.add_char(c);
                    let start_i = i;
                    self.consume();
                    while let Some((i, c)) = self.peek() {
                        if c.is_id_char() {
                            id.add_char(c);
                            self.consume();
                        } else {
                            let token = id.take_to_token();
                            return Some(Ok((start_i, Lexeme::General(token), i)));
                        }
                    }
                    let token = id.take_to_token();
                    return Some(Ok((start_i, Lexeme::General(token), self.eof_offset())));
                }
                _ if c.is_comparator_char() => {
                    let mut id = self.start_cob();
                    id.add_char(c);
                    let start_i = i;
                    self.consume();
                    while let Some((i, c)) = self.peek() {
                        if c.is_comparator_char() {
                            id.add_char(c);
                            self.consume();
                        } else {
                            let token = id.take_to_token();
                            let cmp = parse_comparator(&token);
                            return Some(Ok((start_i, Lexeme::Comparator(cmp, token), i)));
                        }
                    }
                    let token = id.take_to_token();
                    let cmp = parse_comparator(&token);
                    return Some(Ok((start_i, Lexeme::Comparator(cmp, token), self.eof_offset())));
                }
                _ if self.in_calc && (c.is_local_value_char() || c == '.') => {
                    // A number or the name of a reader variable, inside a `@[` calculation
                    let mut id = self.start_cob();
                    id.add_char(c);
                    let start_i = i;
                    self.consume();
                    while let Some((i, c)) = self.peek() {
                        if c.is_local_value_char() || c == '.' {
                            id.add_char(c);
                            self.consume();
                        } else {
                            return Some(Ok((start_i, Lexeme::General(id.take_to_token()), i)));
                        }
                    }
                    return Some(Ok((
                        start_i,
                        Lexeme::General(id.take_to_token()),
                        self.eof_offset(),
                    )));
                }
                // The ; is silently accepted because putting it after a number is a common mistake
                // and doesn't seem to cause any harm.
                ';' => self.consume(),
                '"' => {
                    // A quoted token
                    let start_i = i;
                    let start_loc = self.loc;
                    self.consume();
                    let mut id = self.start_cob();
                    while let Some((i, c)) = self.peek() {
                        if c == '\n' {
                            // Warn, but continue parsing the string.
                            id.add_char(c);
                            let msg = "quoted string not closed";
                            warn(ErrorKey::ParseError).weak().msg(msg).loc(self.loc).push();
                            self.consume();
                        } else if c == '"' {
                            let token = id.take_to_token();
                            self.consume();
                            return Some(Ok((start_i, Lexeme::General(token), i + 1)));
                        } else {
                            id.add_char(c);
                            self.consume();
                        }
                    }
                    let msg = "quoted string not closed";
                    err(ErrorKey::ParseError).msg(msg).loc(start_loc).push();
                    let token = if matches!(id, Cob::Uninit) {
                        Token::from_static_str("", self.loc)
                    } else {
                        id.take_to_token()
                    };
                    return Some(Ok((start_i, Lexeme::General(token), self.eof_offset())));
                }
                '#' => {
                    // A comment
                    self.consume();
                    while let Some((_, c)) = self.peek() {
                        self.consume();
                        if c == '\n' {
                            break;
                        }
                    }
                }
                '$' => {
                    // A macro parameter
                    let start_i = i;
                    let start_loc = self.loc;
                    self.consume();
                    let mut id = self.start_cob();
                    while let Some((i, c)) = self.peek() {
                        if c.is_id_char() {
                            id.add_char(c);
                            self.consume();
                        } else if c == '$' {
                            let token = id.take_to_token();
                            self.consume();
                            return Some(Ok((start_i, Lexeme::MacroParam(token), i + 1)));
                        } else {
                            let msg = "macro parameter not closed";
                            err(ErrorKey::ParseError).msg(msg).loc(self.loc).push();
                            // Return it as a Lexeme::General because a stray $ is not treated
                            // as a macro parameter by the game.
                            let token = id.take_to_token();
                            return Some(Ok((start_i, Lexeme::General(token), i)));
                        }
                    }
                    let msg = "macro parameter not closed";
                    err(ErrorKey::ParseError).msg(msg).loc(start_loc).push();
                    let token = if matches!(id, Cob::Uninit) {
                        Token::from_static_str("", self.loc)
                    } else {
                        id.take_to_token()
                    };
                    return Some(Ok((start_i, Lexeme::General(token), self.eof_offset())));
                }
                '@' => {
                    // A variable reference @name
                    let mut id = self.start_cob();
                    id.add_char(c);
                    let start_i = i;
                    let loc = self.loc;
                    self.consume();
                    if let Some((_, '[')) = self.peek() {
                        // @[ ... ] calculation
                        self.consume();
                        self.in_calc = true;
                        let token = Token::from_static_str("@[", loc);
                        return Some(Ok((start_i, Lexeme::CalcStart(token), start_i + 2)));
                    }
                    if let Some((_, ':')) = self.peek() {
                        // reader directive, such as @:insert
                        id.add_char(':');
                        self.consume();
                        let mut end_i = self.eof_offset();
                        while let Some((i, c)) = self.peek() {
                            // Match c == '-' too, to be able to warn when it's used in place of _
                            if c.is_alphanumeric() || c == '_' || c == '-' {
                                id.add_char(c);
                                self.consume();
                            } else {
                                end_i = i;
                                break;
                            }
                        }
                        let token = id.take_to_token();
                        if !Game::is_ck3() {
                            let msg = "reader directives are only for CK3 so far";
                            err(ErrorKey::WrongGame).msg(msg).loc(&token).push();
                        }
                        let lexeme = match token.as_str() {
                            "@:register_variable" => {
                                let msg =
                                    "`@:register_variable` is (as of CK3 1.13) not yet supported";
                                let info = "prefer just @name = value";
                                err(ErrorKey::Bugs).msg(msg).info(info).loc(&token).push();
                                Some(Lexeme::Directive(Directive::RegisterVariable, token))
                            }
                            "@:register-variable" => {
                                let msg = format!("unknown reader directive `{token}`");
                                let info = "did you mean `@:register_variable`?";
                                err(ErrorKey::ParseError).msg(msg).info(info).loc(&token).push();
                                None
                            }
                            "@:load_variable" => {
                                let msg = "`@:load_variable` is (as of CK3 1.13) not yet supported";
                                let info = "prefer just @name";
                                err(ErrorKey::Bugs).msg(msg).info(info).loc(&token).push();
                                Some(Lexeme::Directive(Directive::LoadVariable, token))
                            }
                            "@:load-variable" => {
                                let msg = format!("unknown reader directive `{token}`");
                                let info = "did you mean `@:load_variable`?";
                                err(ErrorKey::ParseError).msg(msg).info(info).loc(&token).push();
                                None
                            }
                            "@:define" => Some(Lexeme::Directive(Directive::Define, token)),
                            "@:insert" => Some(Lexeme::Directive(Directive::Insert, token)),
                            "@:assert" => {
                                let msg = "`@:assert` should not be left in the script";
                                err(ErrorKey::Crash).msg(msg).loc(&token).push();
                                // Swallow @:assert because it would just complicate the parser.
                                None
                            }
                            "@:log" => Some(Lexeme::Directive(Directive::Log, token)),
                            _ => {
                                let msg = format!("unknown reader directive `{token}`");
                                err(ErrorKey::ParseError).msg(msg).loc(&token).push();
                                None
                            }
                        };
                        if let Some(lexeme) = lexeme {
                            return Some(Ok((start_i, lexeme, end_i)));
                        }
                    } else {
                        while let Some((i, c)) = self.peek() {
                            if c.is_local_value_char() {
                                id.add_char(c);
                                self.consume();
                            } else {
                                return Some(Ok((
                                    start_i,
                                    Lexeme::VariableReference(id.take_to_token()),
                                    i,
                                )));
                            }
                        }
                        return Some(Ok((
                            start_i,
                            Lexeme::VariableReference(id.take_to_token()),
                            self.eof_offset(),
                        )));
                    }
                }
                '{' => {
                    let token = Token::from_static_str("{", self.loc);
                    self.consume();
                    self.brace_depth += 1;
                    return Some(Ok((i, Lexeme::BlockStart(token), i + 1)));
                }
                '}' => {
                    if self.brace_depth > 0 {
                        self.brace_depth -= 1;
                    }
                    if self.loc.column == 1 && self.brace_depth > 0 {
                        let msg = "possible brace error";
                        let info = "This closing brace is at the start of the line but does not close a top-level block.";
                        warn(ErrorKey::BracePlacement)
                            .weak()
                            .msg(msg)
                            .info(info)
                            .loc(self.loc)
                            .push();
                    }
                    let token = Token::from_static_str("}", self.loc);
                    self.consume();
                    self.in_calc = false; // synchronization point
                    return Some(Ok((i, Lexeme::BlockEnd(token), i + 1)));
                }
                ']' => {
                    let token = Token::from_static_str("]", self.loc);
                    self.consume();
                    self.in_calc = false;
                    return Some(Ok((i, Lexeme::CalcEnd(token), i + 1)));
                }
                '(' => {
                    let token = Token::from_static_str("(", self.loc);
                    self.consume();
                    return Some(Ok((i, Lexeme::OpenParen(token), i + 1)));
                }
                ')' => {
                    let token = Token::from_static_str(")", self.loc);
                    self.consume();
                    return Some(Ok((i, Lexeme::CloseParen(token), i + 1)));
                }
                '+' => {
                    let token = Token::from_static_str("+", self.loc);
                    self.consume();
                    return Some(Ok((i, Lexeme::Add(token), i + 1)));
                }
                '-' => {
                    let token = Token::from_static_str("-", self.loc);
                    self.consume();
                    return Some(Ok((i, Lexeme::Subtract(token), i + 1)));
                }
                '*' => {
                    let token = Token::from_static_str("*", self.loc);
                    self.consume();
                    return Some(Ok((i, Lexeme::Multiply(token), i + 1)));
                }
                '/' => {
                    let token = Token::from_static_str("/", self.loc);
                    self.consume();
                    return Some(Ok((i, Lexeme::Divide(token), i + 1)));
                }
                // TODO: should really detect ^Z anywhere in the input.
                // Move this to consume() ?
                CONTROL_Z => {
                    let loc = self.loc;
                    self.consume();
                    let msg = "^Z in file";
                    if self.only_whitespace_left() {
                        let info = "This control code means stop reading the file here, which will cause trouble if you add more code later.";
                        untidy(ErrorKey::ParseError).msg(msg).info(info).loc(loc).push();
                    } else {
                        let info = "This control code means stop reading the file here. Nothing that follows will be read.";
                        err(ErrorKey::ParseError).msg(msg).info(info).loc(loc).push();
                    }
                    return None;
                }
                _ => {
                    let msg = format!("unrecognized character `{c}`");
                    err(ErrorKey::ParseError).msg(msg).loc(self.loc).push();
                    self.consume();
                }
            }
        }
        None
    }
}

fn parse_comparator(token: &Token) -> Comparator {
    let s = token.as_str();
    s.parse::<Comparator>().unwrap_or_else(|_| {
        let msg = format!("unrecognized comparator `{s}`");
        err(ErrorKey::ParseError).msg(msg).loc(token).push();
        Comparator::Equals(Single) // fallback
    })
}
