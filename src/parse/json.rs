/// Parse a JSON file into a `Block`.
/// `Block` is used, instead of a JSON-specific representation, for compatibility with the rest of the code.
/// Unfortunately can't use serde-json because we need the locations for error reporting.
use std::fs::read_to_string;
use std::mem::{swap, take};

use crate::block::Eq::Single;
use crate::block::{Block, Comparator, BV};
use crate::fileset::FileEntry;
use crate::report::{err, warn, ErrorKey};
use crate::token::{Loc, Token};

#[derive(Copy, Clone, Debug)]
enum State {
    Neutral,
    QString,
}

struct ParseLevel {
    block: Block,
    key: Option<Token>,
    expect_colon: bool,
    expect_comma: bool,
    opening_bracket: char,
    closing_bracket: char,
}

struct Parser {
    current: ParseLevel,
    stack: Vec<ParseLevel>,
}

impl Parser {
    fn unknown_char(c: char, loc: Loc) {
        let msg = format!("Unrecognized character {c}");
        err(ErrorKey::ParseError).msg(msg).loc(loc).push();
    }

    fn colon(&mut self, loc: Loc) {
        if !self.current.expect_colon {
            err(ErrorKey::ParseError).msg("unexpected `:`").loc(loc).push();
        }
        self.current.expect_colon = false;
    }

    fn check_colon(&mut self, loc: Loc) {
        if self.current.expect_colon {
            err(ErrorKey::ParseError).msg("expected `:`").loc(loc).push();
            self.current.expect_comma = false;
        }
    }

    fn comma(&mut self, loc: Loc) {
        if !self.current.expect_comma {
            err(ErrorKey::ParseError).msg("unexpected `,`").loc(loc).push();
        }
        self.current.expect_comma = false;
    }

    fn check_comma(&mut self, loc: Loc) {
        if self.current.expect_comma {
            err(ErrorKey::ParseError).msg("expected `,`").loc(loc).push();
            self.current.expect_comma = false;
        }
    }

    fn token(&mut self, token: Token) {
        self.check_comma(token.loc);
        self.check_colon(token.loc);
        if let Some(key) = self.current.key.take() {
            self.current.block.add_key_bv(key, Comparator::Equals(Single), BV::Value(token));
            self.current.expect_comma = true;
        } else if self.current.opening_bracket == '[' {
            self.current.block.add_value(token);
            self.current.expect_comma = true;
        } else {
            self.current.key = Some(token);
            self.current.expect_colon = true;
        }
    }

    fn block_value(&mut self, block: Block) {
        if let Some(key) = self.current.key.take() {
            self.current.block.add_key_bv(key, Comparator::Equals(Single), BV::Block(block));
        } else {
            self.current.block.add_block(block);
        }
        self.current.expect_comma = true;
    }

    fn end_assign(&mut self) {
        if let Some(key) = self.current.key.take() {
            let msg = "key without value";
            err(ErrorKey::ParseError).msg(msg).loc(&key).push();
            self.current.block.add_value(key);
        }
    }

    fn open_bracket(&mut self, loc: Loc, bracket: char) {
        self.check_colon(loc);
        self.check_comma(loc);
        if self.current.opening_bracket == '{' && self.current.key.is_none() {
            err(ErrorKey::ParseError).msg("expected key not block").loc(loc).push();
        }
        let mut new_level = ParseLevel {
            block: Block::new(loc),
            key: None,
            expect_colon: false,
            expect_comma: false,
            opening_bracket: bracket,
            closing_bracket: if bracket == '{' { '}' } else { ']' },
        };
        swap(&mut new_level, &mut self.current);
        self.stack.push(new_level);
    }

    fn close_bracket(&mut self, loc: Loc, bracket: char) {
        self.end_assign();
        if let Some(mut prev_level) = self.stack.pop() {
            swap(&mut self.current, &mut prev_level);
            if prev_level.closing_bracket != bracket {
                let msg = format!("this {bracket} closes a {}", self.current.opening_bracket);
                err(ErrorKey::ParseError)
                    .strong()
                    .msg(msg)
                    .loc(loc)
                    .loc(prev_level.block.loc, "here")
                    .push();
            }
            self.block_value(prev_level.block);
            if loc.column == 1 && !self.stack.is_empty() {
                let msg = "possible bracket error";
                let info = "This closing bracket is at the start of a line but does not end a top-level item.";
                warn(ErrorKey::BracePlacement).msg(msg).info(info).loc(loc).push();
            }
        } else {
            err(ErrorKey::ParseError).msg(format!("Unexpected {bracket}")).loc(loc).push();
        }
    }

    fn eof(mut self) -> Block {
        self.end_assign();
        while let Some(mut prev_level) = self.stack.pop() {
            let msg = format!("Opening {} was never closed", prev_level.opening_bracket);
            err(ErrorKey::ParseError).msg(msg).loc(prev_level.block.loc).push();
            swap(&mut self.current, &mut prev_level);
            self.block_value(prev_level.block);
        }
        self.current.block
    }
}

fn parse(blockloc: Loc, content: &str) -> Block {
    let mut parser = Parser {
        current: ParseLevel {
            block: Block::new(blockloc),
            key: None,
            expect_colon: false,
            expect_comma: false,
            opening_bracket: '[',
            closing_bracket: ']',
        },
        stack: Vec::new(),
    };
    let mut state = State::Neutral;
    let mut token_start = blockloc;
    let mut current_id = String::new();

    let mut loc = blockloc;
    for c in content.chars() {
        match state {
            State::Neutral => {
                if c.is_ascii_whitespace() {
                } else if c == '"' {
                    token_start = loc;
                    state = State::QString;
                } else if c == ':' {
                    parser.colon(loc);
                } else if c == ',' {
                    parser.comma(loc);
                } else if c == '{' {
                    parser.open_bracket(loc, '{');
                } else if c == '}' {
                    parser.close_bracket(loc, '}');
                } else if c == '[' {
                    parser.open_bracket(loc, '[');
                } else if c == ']' {
                    parser.close_bracket(loc, ']');
                } else {
                    Parser::unknown_char(c, loc);
                }
            }
            State::QString => {
                if c == '"' {
                    let token = Token::new(&take(&mut current_id), token_start);
                    parser.token(token);
                    state = State::Neutral;
                } else if c == '\n' {
                    let token = Token::new(&take(&mut current_id), token_start);
                    warn(ErrorKey::ParseError).msg("quoted string not closed").loc(token).push();
                    state = State::Neutral;
                } else {
                    current_id.push(c);
                }
            }
        }

        if c == '\n' {
            loc.line += 1;
            loc.column = 1;
        } else {
            loc.column += 1;
        }
    }

    // Deal with state at end of file
    match state {
        State::QString => {
            let token = Token::new(&current_id, token_start);
            err(ErrorKey::ParseError).msg("Quoted string not closed").loc(&token).push();
            parser.token(token);
        }
        State::Neutral => (),
    }

    parser.eof()
}

#[allow(clippy::module_name_repetitions)]
pub fn parse_json(entry: &FileEntry, content: &str) -> Block {
    let mut loc = Loc::from(entry);
    loc.line = 1;
    loc.column = 1;
    parse(loc, content)
}

#[allow(clippy::module_name_repetitions)]
pub fn parse_json_file(entry: &FileEntry) -> Option<Block> {
    let contents = match read_to_string(entry.fullpath()) {
        Ok(contents) => contents,
        Err(e) => {
            err(ErrorKey::ReadError).msg("could not read file").info(format!("{e:#}")).loc(entry).push();
            return None;
        }
    };
    if let Some(bomless) = contents.strip_prefix('\u{feff}') {
        Some(parse_json(entry, bomless))
    } else {
        Some(parse_json(entry, &contents))
    }
}
