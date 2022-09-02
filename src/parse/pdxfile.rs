use anyhow::{bail, Result};
use fnv::FnvHashMap;
use std::mem::swap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::block::{Block, BlockOrValue, Comparator};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn, warn_info};
use crate::fileset::FileKind;
use crate::token::{Loc, Token};

#[derive(Copy, Clone, Debug)]
enum State {
    Neutral,
    QString,
    Id,
    Comparator,
    Calculation,
    Comment,
}

#[allow(clippy::wrong_self_convention)]
trait CharExt {
    fn is_id_char(self) -> bool;
    fn is_comparator_char(self) -> bool;
}

impl CharExt for char {
    fn is_id_char(self) -> bool {
        self.is_alphabetic()
            || self.is_ascii_digit()
            || self == '.'
            || self == ':'
            || self == '$'
            || self == '_'
            || self == '-'
            || self == '&'
            || self == '/'
            || self == '|'
            || self == '\''
    }

    fn is_comparator_char(self) -> bool {
        self == '<' || self == '>' || self == '!' || self == '='
    }
}

struct ParseLevel {
    block: Block,
    key: Option<Token>,
    comp: Option<(Comparator, Token)>,
    tag: Option<Token>,
}

struct Parser {
    pathname: Rc<PathBuf>,
    current: ParseLevel,
    stack: Vec<ParseLevel>,
    brace_error: bool,
    local_macros: FnvHashMap<String, f64>,
}

impl Parser {
    fn unknown_char(c: char, loc: Loc) {
        let token = Token::new(c.to_string(), loc);
        error(
            token,
            ErrorKey::ParseError,
            &format!("Unrecognized character {}", c),
        );
    }

    fn token(&mut self, token: Token) {
        // Special case parsing of color = hsv { ... }
        if token.is("hsv") {
            self.current.tag = Some(token);
            return;
        }
        if let Some(key) = self.current.key.take() {
            if let Some((comp, _)) = self.current.comp.take() {
                if let Some(local_macro) = key.as_str().strip_prefix('@') {
                    if let Ok(value) = token.as_str().parse::<f64>() {
                        self.local_macros.insert(local_macro.to_string(), value);
                    } else {
                        error(token, ErrorKey::ParseError, "can't parse local value");
                    }
                } else if let Some(local_macro) = token.as_str().strip_prefix('@') {
                    if let Some(value) = self.local_macros.get(local_macro) {
                        let token = Token::new(value.to_string(), token.loc);
                        self.current
                            .block
                            .add_key_value(key, comp, BlockOrValue::Token(token));
                    } else {
                        error(token, ErrorKey::ParseError, "local value not defined");
                    }
                } else {
                    self.current
                        .block
                        .add_key_value(key, comp, BlockOrValue::Token(token));
                }
            } else {
                self.current.block.add_value(BlockOrValue::Token(key));
                self.current.key = Some(token);
            }
        } else {
            self.current.key = Some(token);
        }
    }

    fn block_value(&mut self, mut block: Block) {
        // Like token(), but block values cannot become keys
        if let Some(tag) = self.current.tag.take() {
            block.tag = Some(tag);
        }
        if let Some(key) = self.current.key.take() {
            if let Some((comp, _)) = self.current.comp.take() {
                self.current
                    .block
                    .add_key_value(key, comp, BlockOrValue::Block(block));
            } else {
                self.current.block.add_value(BlockOrValue::Token(key));
                self.current.block.add_value(BlockOrValue::Block(block));
            }
        } else {
            self.current.block.add_value(BlockOrValue::Block(block));
        }
    }

    fn comparator(&mut self, token: Token) {
        let cmp = Comparator::from_token(&token).unwrap_or_else(|| {
            error(
                &token,
                ErrorKey::ParseError,
                &format!("Unrecognized comparator '{}'", token),
            );
            Comparator::Eq
        });

        if self.current.key.is_none() {
            let msg = format!("Unexpected comparator '{}'", token);
            error(token, ErrorKey::ParseError, &msg);
        } else {
            if self.current.comp.is_some() {
                let msg = &format!("Double comparator '{}'", token);
                error(&token, ErrorKey::ParseError, msg);
            }
            self.current.comp = Some((cmp, token));
        }
    }

    fn end_assign(&mut self) {
        if let Some(key) = self.current.key.take() {
            if let Some((_, comp_token)) = self.current.comp.take() {
                error(comp_token, ErrorKey::ParseError, "Comparator without value");
            }
            self.current.block.add_value(BlockOrValue::Token(key));
        }
    }

    fn open_brace(&mut self, loc: Loc) {
        let mut new_level = ParseLevel {
            block: Block::new(loc),
            key: None,
            comp: None,
            tag: None,
        };
        swap(&mut new_level, &mut self.current);
        self.stack.push(new_level);
    }

    fn close_brace(&mut self, loc: Loc) {
        self.end_assign();
        if let Some(mut prev_level) = self.stack.pop() {
            swap(&mut self.current, &mut prev_level);
            self.block_value(prev_level.block);
            if loc.column == 1 && !self.stack.is_empty() {
                warn_info(&Token::new("}".to_string(), loc),
                ErrorKey::BracePlacement,
                "possible brace error",
                "This closing brace is at the start of a line but does not end a top-level item."
                );
            }
        } else {
            self.brace_error = true;
            error(
                &Token::new("}".to_string(), loc),
                ErrorKey::ParseError,
                "Unexpected }",
            );
        }
    }

    fn eof(mut self) -> Result<Block> {
        self.end_assign();
        while let Some(mut prev_level) = self.stack.pop() {
            self.brace_error = true;
            error(
                &Token::new("{".to_string(), self.current.block.loc.clone()),
                ErrorKey::ParseError,
                "Opening { was never closed",
            );
            swap(&mut self.current, &mut prev_level);
            self.block_value(prev_level.block);
        }
        // Brace errors mean we shouldn't try to use the file at all,
        // since its structure is unclear. Validating such a file would
        // just produce a cascade of irrelevant errors.
        if self.brace_error {
            bail!(
                "Could not parse {} due to brace mismatch",
                self.pathname.display()
            );
        }
        Ok(self.current.block)
    }
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::too_many_lines)] // many lines are natural for state machines
pub fn parse_pdx(pathname: &Path, kind: FileKind, content: &str) -> Result<Block> {
    let pathname = Rc::new(pathname.to_path_buf());
    let mut loc = Loc::new(pathname.clone(), kind);
    let block_loc = Loc::for_file(pathname.clone(), kind);
    let mut parser = Parser {
        pathname,
        current: ParseLevel {
            block: Block::new(block_loc),
            key: None,
            comp: None,
            tag: None,
        },
        stack: Vec::new(),
        brace_error: false,
        local_macros: FnvHashMap::default(),
    };
    let mut state = State::Neutral;
    let mut token_start = loc.clone();

    for (i, c) in content.char_indices() {
        loc.offset = i;
        let next_i = i + c.len_utf8();

        match state {
            State::Neutral => {
                if c.is_whitespace() {
                } else if c == '"' {
                    state = State::QString;
                } else if c == '#' {
                    state = State::Comment;
                } else if c.is_comparator_char() {
                    state = State::Comparator;
                } else if c == '@' || c.is_id_char() {
                    // @ can start tokens but is special
                    state = State::Id;
                } else if c == '{' {
                    parser.open_brace(loc.clone());
                } else if c == '}' {
                    parser.close_brace(loc.clone());
                } else {
                    Parser::unknown_char(c, loc.clone());
                }
                token_start = loc.clone();
            }
            State::Comment => {
                if c == '\n' {
                    state = State::Neutral;
                }
            }
            State::QString => {
                if c == '"' {
                    state = State::Id;
                } else if c == '\n' {
                    let s = content[token_start.offset..next_i].replace('"', "");
                    let token = Token::new(s, token_start.clone());
                    warn(token, ErrorKey::ParseError, "Quoted string not closed");
                }
            }
            State::Id => {
                if c == '"' {
                    // The quoted string actually becomes part of this id
                    state = State::QString;
                } else if c.is_id_char() {
                } else if c == '[' && loc.offset == token_start.offset + 1 {
                    state = State::Calculation;
                } else {
                    let id = content[token_start.offset..i].replace('"', "");
                    let token = Token::new(id, token_start.clone());
                    parser.token(token);

                    if c.is_comparator_char() {
                        state = State::Comparator;
                    } else if c.is_whitespace() {
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c == '{' {
                        parser.open_brace(loc.clone());
                        state = State::Neutral;
                    } else if c == '}' {
                        parser.close_brace(loc.clone());
                        state = State::Neutral;
                    } else {
                        Parser::unknown_char(c, loc.clone());
                        state = State::Neutral;
                    }
                    token_start = loc.clone();
                }
            }
            State::Calculation => {
                // TODO: we should probably parse these and do math on them, and return
                // the resulting values as part of the tokens
                if c == ']' {
                    let id = content[token_start.offset..=i].to_string();
                    let token = Token::new(id, token_start.clone());
                    parser.token(token);
                    state = State::Neutral;
                    token_start = loc.clone();
                }
            }
            State::Comparator => {
                if c.is_comparator_char() {
                } else {
                    let s = content[token_start.offset..i].to_string();
                    let token = Token::new(s, token_start.clone());
                    parser.comparator(token);

                    if c == '"' {
                        state = State::QString;
                    } else if c.is_id_char() {
                        state = State::Id;
                    } else if c.is_whitespace() {
                        state = State::Neutral;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c == '{' {
                        parser.open_brace(loc.clone());
                        state = State::Neutral;
                    } else if c == '}' {
                        parser.close_brace(loc.clone());
                        state = State::Neutral;
                    } else {
                        Parser::unknown_char(c, loc.clone());
                        state = State::Neutral;
                    }
                    token_start = loc.clone();
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
            let s = content[token_start.offset..].to_string();
            let token = Token::new(s, token_start);
            error(&token, ErrorKey::ParseError, "Quoted string not closed");
            parser.token(token);
        }
        State::Id => {
            let s = content[token_start.offset..].to_string();
            let token = Token::new(s, token_start);
            parser.token(token);
        }
        State::Comparator => {
            let s = content[token_start.offset..].to_string();
            let token = Token::new(s, token_start);
            parser.comparator(token);
        }
        _ => (),
    }

    parser.eof()
}
