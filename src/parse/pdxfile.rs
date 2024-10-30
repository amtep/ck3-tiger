//! Parses a Pdx script file into a [`Block`].
//!
//! The main entry points are [`parse_pdx_file`], [`parse_pdx_macro`], and [`parse_pdx_internal`].

use std::path::PathBuf;

use lalrpop_util::{lalrpop_mod, ParseError};

use crate::block::{Block, Comparator, Eq};
use crate::fileset::{FileEntry, FileKind};
use crate::parse::cob::Cob;
use crate::parse::pdxfile::lexer::{LexError, Lexeme, Lexer};
pub use crate::parse::pdxfile::memory::GlobalMemory;
use crate::parse::pdxfile::memory::LocalMemory;
use crate::parse::ParserMemory;
use crate::report::{err, store_source_file, ErrorKey};
use crate::token::{leak, Loc, Token};

mod lexer;
pub mod memory;
lalrpop_mod! {
    #[allow(unused_variables)]
    #[allow(unused_imports)]
    #[allow(dead_code)]
    #[rustfmt::skip]
    #[allow(clippy::pedantic)]
    #[allow(clippy::if_then_some_else_none)]
    parser, "/parse/pdxfile/parser.rs"
}

/// Re-parse a macro (which is a scripted effect, trigger, or modifier that uses $ parameters)
/// after argument substitution. A full re-parse is needed because the game engine allows tricks
/// such as passing `#` as a macro argument in order to comment out the rest of a line.
pub fn parse_pdx_macro(inputs: &[Token], memory: &GlobalMemory) -> Block {
    let mut local = LocalMemory::new(memory);
    match parser::FileParser::new().parse(inputs, &mut local, Lexer::new(inputs)) {
        Ok(block) => block,
        Err(e) => {
            eprintln!("Internal error: re-parsing macro failed.\n{e}");
            Block::new(inputs[0].loc)
        }
    }
}

/// Parse a whole file into a `Block`.
fn parse_pdx(entry: &FileEntry, content: &'static str, memory: &ParserMemory) -> Block {
    let file_loc = Loc::from(entry);
    let mut loc = file_loc;
    loc.line = 1;
    loc.column = 1;
    let inputs = [Token::from_static_str(content, loc)];
    let mut local = LocalMemory::new(&memory.pdxfile);
    match parser::FileParser::new().parse(&inputs, &mut local, Lexer::new(&inputs)) {
        Ok(mut block) => {
            block.loc = file_loc;
            block
        }
        Err(e) => {
            eprintln!("Internal error: parsing file {} failed.\n{e}", entry.path().display());
            Block::new(inputs[0].loc)
        }
    }
}

/// Parse the content associated with the [`FileEntry`].
pub fn parse_pdx_file(
    entry: &FileEntry,
    content: String,
    offset: usize,
    parser: &ParserMemory,
) -> Block {
    let content = leak(content);
    store_source_file(entry.fullpath().to_path_buf(), &content[offset..]);
    parse_pdx(entry, &content[offset..], parser)
}

/// Parse a string into a [`Block`]. This function is meant for use by the validator itself, to
/// allow it to load game description data from internal strings that are in pdx script format.
pub fn parse_pdx_internal(input: &'static str, desc: &str) -> Block {
    let entry = FileEntry::new(PathBuf::from(desc), FileKind::Internal, PathBuf::from(desc));
    parse_pdx(&entry, input, &ParserMemory::default())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Kinds of [`MacroComponent`].
pub enum MacroComponentKind {
    Source,
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
fn split_macros(token: &Token) -> Vec<MacroComponent> {
    let mut vec = Vec::new();
    let mut index_loc = (0, token.loc);
    for lex in Lexer::new(&[token.clone()]).flatten() {
        #[allow(clippy::cast_possible_truncation)]
        if let (start, Lexeme::MacroParam(param), end) = lex {
            // The param token does not include the enclosing `$` chars, but the start..end range does.
            vec.push(MacroComponent {
                kind: MacroComponentKind::Source,
                token: token.subtoken(index_loc.0..start, index_loc.1),
            });
            // Do this before pushing `param` to the vec, because it uses `param`.
            index_loc = (end, param.loc);
            index_loc.1.column += 1 + param.as_str().chars().count() as u32;
            vec.push(MacroComponent { kind: MacroComponentKind::Macro, token: param });
        }
    }
    vec.push(MacroComponent {
        kind: MacroComponentKind::Source,
        token: token.subtoken(index_loc.0.., index_loc.1),
    });
    vec
}

// Definitions used by parser.lalrpop

type HasMacroParams = bool;

fn define_var(memory: &mut LocalMemory, token: &Token, cmp: Comparator, value: Token) {
    // A direct `@name = value` assignment gets the leading `@`,
    // while a `@:register_variable name = value` does not.
    let name = match token.as_str().strip_prefix('@') {
        Some(name) => name,
        None => token.as_str(),
    };
    if !matches!(cmp, Comparator::Equals(Eq::Single)) {
        let msg = format!("expected `{name} =`");
        err(ErrorKey::ReaderDirectives).msg(msg).loc(token).push();
    }
    if memory.has_variable(name) {
        let msg = format!("`{name}` is already defined as a reader variable");
        err(ErrorKey::ReaderDirectives).msg(msg).loc(token).push();
    } else if !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        let msg = "reader variable names must start with an ascii letter";
        err(ErrorKey::ReaderDirectives).msg(msg).loc(token).push();
    } else {
        memory.set_variable(name.to_string(), value);
    }
}

fn warn_macros(token: &Token, has_macro_params: bool) {
    if has_macro_params {
        let msg = "$-substitutions only work inside blocks";
        err(ErrorKey::Macro).msg(msg).loc(token).push();
    }
}

fn report_error(error: ParseError<usize, Lexeme, LexError>, mut file_loc: Loc) {
    match error {
        ParseError::InvalidToken { location: _ } // we don't pass `LexError`s
        | ParseError::User { error: _ } => unreachable!(),
        ParseError::UnrecognizedEof { location: _, expected: _ } => {
            let msg = "unexpected end of file";
            file_loc.line = 0;
            file_loc.column = 0;
            err(ErrorKey::ParseError).msg(msg).loc(file_loc).push();
        }
        ParseError::UnrecognizedToken { token: (_, lexeme, _), expected: _ }
        | ParseError::ExtraToken { token: (_, lexeme, _) } => {
            let msg = format!("unexpected {lexeme}");
            let token = lexeme.into_token();
            err(ErrorKey::ParseError).msg(msg).loc(token).push();
        }
    };
}

fn get_numeric_var(memory: &LocalMemory, name: &Token) -> f64 {
    if let Some(value) = name.get_number() {
        value
    } else if let Some(v) = memory.get_variable(name.as_str()) {
        if let Some(value) = v.get_number() {
            value
        } else {
            let msg = format!("expected reader variable `{name}` to be numeric");
            err(ErrorKey::ReaderDirectives).msg(msg).loc(name).loc_msg(v, "defined here").push();
            0.0
        }
    } else {
        let msg = format!("reader variable {name} not defined");
        err(ErrorKey::ReaderDirectives).msg(msg).loc(name).push();
        0.0
    }
}

/// A convenience trait to add some methods to [`char`]
#[allow(clippy::wrong_self_convention)]
trait CharExt {
    /// Can the char be part of an unquoted token?
    fn is_id_char(self) -> bool;
    /// Can the char be part of a reader variable name?
    fn is_local_value_char(self) -> bool;
    /// Can the char be part of a [`Comparator`]?
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
