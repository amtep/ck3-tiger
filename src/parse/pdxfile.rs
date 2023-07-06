use std::mem::{swap, take};
use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::block::{Block, Comparator, BV};
use crate::fileset::{FileEntry, FileKind};
use crate::report::{error, warn, warn_info, ErrorKey};
use crate::token::{Loc, Token};

#[derive(Copy, Clone, Debug)]
enum State {
    Neutral,
    QString,
    Id,
    Comparator,
    Calculation,
    CalculationId,
    Comment,
}

#[derive(Clone, Debug)]
enum Calculation {
    Value(f64),
    Add,
    Subtract,
    Multiply,
    Divide(Loc),
}

impl Calculation {
    fn is_value(&self) -> bool {
        match self {
            Calculation::Value(_) => true,
            Calculation::Add
            | Calculation::Subtract
            | Calculation::Multiply
            | Calculation::Divide(_) => false,
        }
    }

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
                    match calc[i] {
                        Calculation::Multiply => {
                            calc.splice(i - 1..=i + 1, vec![Calculation::Value(value1 * value2)]);
                            i -= 1;
                        }
                        Calculation::Divide(ref loc) => {
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
        return 0.0;
    }
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

    fn is_comparator_char(self) -> bool {
        self == '<' || self == '>' || self == '!' || self == '=' || self == '?'
    }
}

#[derive(Clone, Debug, Default)]
pub struct LocalMacros {
    values: FnvHashMap<String, f64>,
    text: FnvHashMap<String, String>,
}

impl LocalMacros {
    fn get_value(&self, key: &str) -> Option<f64> {
        // key can be a local macro or a literal numeric value
        self.values.get(key).copied().or_else(|| key.parse().ok())
    }

    fn get_as_string(&self, key: &str) -> Option<String> {
        if let Some(value) = self.values.get(key) {
            Some(value.to_string())
        } else {
            self.text.get(key).map(ToString::to_string)
        }
    }

    fn insert(&mut self, key: &str, value: &str) {
        if let Ok(value) = value.parse::<f64>() {
            self.values.insert(key.to_string(), value);
        } else {
            self.text.insert(key.to_string(), value.to_string());
        }
    }
}

struct ParseLevel {
    block: Block,
    start: usize,
    key: Option<Token>,
    comp: Option<(Comparator, Token)>,
    tag: Option<Token>,
    contains_macro_parms: bool,
}

struct Parser {
    current: ParseLevel,
    stack: Vec<ParseLevel>,
    local_macros: LocalMacros,
    calculation_stack: Vec<Vec<Calculation>>,
    calculation: Vec<Calculation>,
}

impl Parser {
    fn unknown_char(c: char, loc: Loc) {
        let token = Token::new(c.to_string(), loc);
        let msg = format!("Unrecognized character {c}");
        error(token, ErrorKey::ParseError, &msg);
    }

    fn calculation_start(&mut self) {
        self.calculation = Vec::new();
        self.calculation_stack = Vec::new();
    }

    fn calculation_op(&mut self, op: Calculation, loc: &Loc) {
        if let Some(Calculation::Value(_)) = self.calculation.last() {
            self.calculation.push(op);
        } else if let Calculation::Subtract = op {
            // accept negation
            self.calculation.push(op);
        } else {
            let msg = "operator `{op}` without left-hand value";
            error(loc, ErrorKey::LocalValues, &msg);
        }
    }

    fn calculation_next(&mut self, local_macro: &Token) {
        if let Some(value) = self.local_macros.get_value(local_macro.as_str()) {
            self.calculation.push(Calculation::Value(value));
        } else {
            let msg = format!("local value {local_macro} not defined");
            error(local_macro, ErrorKey::LocalValues, &msg);
        }
    }

    fn calculation_push(&mut self, loc: &Loc) {
        if let Some(Calculation::Value(_)) = self.calculation.last() {
            let msg = "calculation has two values with no operator in between";
            error(loc, ErrorKey::LocalValues, msg);
        }
        self.calculation_stack.push(take(&mut self.calculation));
    }

    fn calculation_pop(&mut self, loc: &Loc) {
        if let Some(mut calc) = self.calculation_stack.pop() {
            calc.push(Calculation::Value(self.calculation_result()));
            self.calculation = calc;
        } else {
            let msg = "found `)` without corresponding `(`";
            warn(loc, ErrorKey::LocalValues, msg);
        }
    }

    fn calculation_result(&mut self) -> f64 {
        Calculation::calculate(take(&mut self.calculation))
    }

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
            if let Some((comp, _)) = self.current.comp.take() {
                if let Some(local_macro) = key.as_str().strip_prefix('@') {
                    self.local_macros.insert(local_macro, token.as_str());
                } else if let Some(local_macro) = token.as_str().strip_prefix('@') {
                    // Check for a '!' to avoid looking up macros in gui code that uses @icon! syntax
                    if token.as_str().contains('!') {
                        self.current
                            .block
                            .add_key_value(key, comp, BV::Value(token));
                    } else if let Some(value) = self.local_macros.get_as_string(local_macro) {
                        let token = Token::new(value, token.loc);
                        self.current
                            .block
                            .add_key_value(key, comp, BV::Value(token));
                    } else {
                        error(token, ErrorKey::LocalValues, "local value not defined");
                    }
                } else {
                    self.current
                        .block
                        .add_key_value(key, comp, BV::Value(token));
                }
            } else {
                if let Some(local_macro) = key.as_str().strip_prefix('@') {
                    if let Some(value) = self.local_macros.get_as_string(local_macro) {
                        let token = Token::new(value, key.loc);
                        self.current.block.add_value(BV::Value(token));
                    } else {
                        error(&key, ErrorKey::LocalValues, "local value not defined");
                        self.current.block.add_value(BV::Value(key));
                    }
                } else {
                    self.current.block.add_value(BV::Value(key));
                }
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
                    .add_key_value(key, comp, BV::Block(block));
            } else {
                self.current.block.add_value(BV::Value(key));
                self.current.block.add_value(BV::Block(block));
            }
        } else {
            self.current.block.add_value(BV::Block(block));
        }
    }

    fn comparator(&mut self, token: Token) {
        let cmp = Comparator::from_token(&token).unwrap_or_else(|| {
            let msg = format!("Unrecognized comparator '{token}'");
            error(&token, ErrorKey::ParseError, &msg);
            Comparator::Eq
        });

        if self.current.key.is_none() {
            let msg = format!("Unexpected comparator '{token}'");
            error(token, ErrorKey::ParseError, &msg);
        } else {
            if self.current.comp.is_some() {
                let msg = &format!("Double comparator '{token}'");
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
            if let Some(local_macro) = key.as_str().strip_prefix('@') {
                if let Some(value) = self.local_macros.get_as_string(local_macro) {
                    let token = Token::new(value, key.loc);
                    self.current.block.add_value(BV::Value(token));
                } else {
                    error(&key, ErrorKey::LocalValues, "local value not defined");
                    self.current.block.add_value(BV::Value(key));
                }
            } else {
                self.current.block.add_value(BV::Value(key));
            }
        }
    }

    fn open_brace(&mut self, loc: Loc, offset: usize) {
        let mut new_level = ParseLevel {
            block: Block::new(loc),
            start: offset,
            key: None,
            comp: None,
            tag: None,
            contains_macro_parms: false,
        };
        swap(&mut new_level, &mut self.current);
        self.stack.push(new_level);
    }

    fn close_brace(&mut self, loc: Loc, content: &str, offset: usize) {
        self.end_assign();
        if let Some(mut prev_level) = self.stack.pop() {
            swap(&mut self.current, &mut prev_level);
            if self.stack.is_empty() && prev_level.contains_macro_parms {
                // skip the { } in constructing s
                let s = content[prev_level.start + 1..offset - 1].to_string();
                let mut loc = prev_level.block.loc.clone();
                loc.column += 1;
                let token = Token::new(s, prev_level.block.loc.clone());
                prev_level.block.source = Some((split_macros(&token), self.local_macros.clone()));
            } else {
                self.current.contains_macro_parms |= prev_level.contains_macro_parms;
            }
            self.block_value(prev_level.block);
            if loc.column == 1 && !self.stack.is_empty() {
                warn_info(Token::new("}".to_string(), loc),
                          ErrorKey::BracePlacement,
                          "possible brace error",
                          "This closing brace is at the start of a line but does not end a top-level item.",
                );
            }
        } else {
            error(
                Token::new("}".to_string(), loc),
                ErrorKey::ParseError,
                "Unexpected }",
            );
        }
    }

    fn eof(mut self) -> Block {
        self.end_assign();
        while let Some(mut prev_level) = self.stack.pop() {
            error(
                &Token::new("{".to_string(), self.current.block.loc.clone()),
                ErrorKey::ParseError,
                "Opening { was never closed",
            );
            swap(&mut self.current, &mut prev_level);
            self.block_value(prev_level.block);
        }
        self.current.block
    }
}

#[allow(clippy::too_many_lines)] // many lines are natural for state machines
fn parse(blockloc: Loc, inputs: &[Token], local_macros: LocalMacros) -> Block {
    let mut parser = Parser {
        current: ParseLevel {
            block: Block::new(blockloc.clone()),
            start: 0,
            key: None,
            comp: None,
            tag: None,
            contains_macro_parms: false,
        },
        stack: Vec::new(),
        local_macros,
        calculation_stack: Vec::new(),
        calculation: Vec::new(),
    };
    let mut state = State::Neutral;
    let mut token_start = blockloc.clone();
    let mut calculation_start = blockloc;
    let mut current_id = String::new();

    for token in inputs {
        let content = token.as_str();
        let mut loc = token.loc.clone();

        for (i, c) in content.char_indices() {
            match state {
                State::Neutral => {
                    if c.is_ascii_whitespace() {
                    } else if c == '"' {
                        token_start = loc.clone();
                        state = State::QString;
                    } else if c == '#' {
                        state = State::Comment;
                    } else if c.is_comparator_char() {
                        token_start = loc.clone();
                        state = State::Comparator;
                        current_id.push(c);
                    } else if c == '@' {
                        // @ can start tokens but is special
                        calculation_start = loc.clone();
                        current_id.push(c);
                        token_start = loc.clone();
                        state = State::Id;
                    } else if c == '$' {
                        parser.current.contains_macro_parms = true;
                        token_start = loc.clone();
                        state = State::Id;
                        current_id.push(c);
                    } else if c.is_id_char() {
                        token_start = loc.clone();
                        state = State::Id;
                        current_id.push(c);
                    } else if c == '{' {
                        parser.open_brace(loc.clone(), i);
                    } else if c == '}' {
                        parser.close_brace(loc.clone(), content, i);
                    } else {
                        Parser::unknown_char(c, loc.clone());
                    }
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
                        let token = Token::new(take(&mut current_id), token_start.clone());
                        warn(token, ErrorKey::ParseError, "Quoted string not closed");
                    } else {
                        current_id.push(c);
                    }
                }
                State::Id => {
                    if c == '"' {
                        // The quoted string actually becomes part of this id
                        state = State::QString;
                    } else if c == '$' {
                        parser.current.contains_macro_parms = true;
                        current_id.push(c);
                    } else if c == '[' && current_id == "@" {
                        state = State::Calculation;
                        parser.calculation_start();
                    } else if c.is_id_char() {
                        current_id.push(c);
                    } else {
                        let token = Token::new(take(&mut current_id), token_start.clone());
                        parser.token(token);

                        if c.is_comparator_char() {
                            current_id.push(c);
                            token_start = loc.clone();
                            state = State::Comparator;
                        } else if c.is_ascii_whitespace() {
                            state = State::Neutral;
                        } else if c == '#' {
                            state = State::Comment;
                        } else if c == '{' {
                            parser.open_brace(loc.clone(), i);
                            state = State::Neutral;
                        } else if c == '}' {
                            parser.close_brace(loc.clone(), content, i);
                            state = State::Neutral;
                        } else {
                            Parser::unknown_char(c, loc.clone());
                            state = State::Neutral;
                        }
                    }
                }
                State::Calculation => {
                    current_id.clear();
                    if c.is_ascii_whitespace() {
                    } else if c == '+' {
                        parser.calculation_op(Calculation::Add, &loc);
                    } else if c == '-' {
                        parser.calculation_op(Calculation::Subtract, &loc);
                    } else if c == '*' {
                        parser.calculation_op(Calculation::Multiply, &loc);
                    } else if c == '/' {
                        parser.calculation_op(Calculation::Divide(loc.clone()), &loc);
                    } else if c == '(' {
                        parser.calculation_push(&loc);
                    } else if c == ')' {
                        parser.calculation_pop(&loc);
                    } else if c == ']' {
                        let token = Token::new(
                            parser.calculation_result().to_string(),
                            calculation_start.clone(),
                        );
                        parser.token(token);
                        state = State::Neutral;
                    } else if c.is_id_char() {
                        token_start = loc.clone();
                        state = State::CalculationId;
                        current_id.push(c);
                    }
                }
                State::CalculationId => {
                    if c.is_ascii_whitespace()
                        || c == '+'
                        || c == '/'
                        || c == '*'
                        || c == '-'
                        || c == '('
                        || c == ')'
                    {
                        let token = Token::new(take(&mut current_id), token_start.clone());
                        parser.calculation_next(&token);
                        state = State::Calculation;
                        if c == '+' {
                            parser.calculation_op(Calculation::Add, &loc);
                        } else if c == '-' {
                            parser.calculation_op(Calculation::Subtract, &loc);
                        } else if c == '*' {
                            parser.calculation_op(Calculation::Multiply, &loc);
                        } else if c == '/' {
                            parser.calculation_op(Calculation::Divide(loc.clone()), &loc);
                        } else if c == '(' {
                            parser.calculation_push(&loc);
                        } else if c == ')' {
                            parser.calculation_pop(&loc);
                        }
                    } else if c == ']' {
                        let token = Token::new(take(&mut current_id), token_start.clone());
                        parser.calculation_next(&token);

                        let token = Token::new(
                            parser.calculation_result().to_string(),
                            calculation_start.clone(),
                        );
                        parser.token(token);
                        state = State::Neutral;
                    } else if c.is_id_char() {
                        current_id.push(c);
                    } else {
                        Parser::unknown_char(c, loc.clone());
                        current_id.clear();
                        state = State::Neutral;
                    }
                }
                State::Comparator => {
                    if c.is_comparator_char() {
                        current_id.push(c);
                    } else {
                        let token = Token::new(take(&mut current_id), token_start.clone());
                        parser.comparator(token);

                        if c == '"' {
                            token_start = loc.clone();
                            state = State::QString;
                        } else if c == '@' {
                            // @ can start tokens but is special
                            calculation_start = loc.clone();
                            current_id.push(c);
                            token_start = loc.clone();
                            state = State::Id;
                        } else if c == '$' {
                            parser.current.contains_macro_parms = true;
                            token_start = loc.clone();
                            state = State::Id;
                            current_id.push(c);
                        } else if c.is_id_char() {
                            token_start = loc.clone();
                            state = State::Id;
                            current_id.push(c);
                        } else if c.is_ascii_whitespace() {
                            state = State::Neutral;
                        } else if c == '#' {
                            state = State::Comment;
                        } else if c == '{' {
                            parser.open_brace(loc.clone(), i);
                            state = State::Neutral;
                        } else if c == '}' {
                            parser.close_brace(loc.clone(), content, i);
                            state = State::Neutral;
                        } else {
                            Parser::unknown_char(c, loc.clone());
                            state = State::Neutral;
                        }
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
    }

    // Deal with state at end of file
    match state {
        State::QString => {
            let token = Token::new(current_id, token_start);
            error(&token, ErrorKey::ParseError, "Quoted string not closed");
            parser.token(token);
        }
        State::Id => {
            let token = Token::new(current_id, token_start);
            parser.token(token);
        }
        State::Comparator => {
            let token = Token::new(current_id, token_start);
            parser.comparator(token);
        }
        _ => (),
    }

    parser.eof()
}

#[allow(clippy::module_name_repetitions)]
pub fn parse_pdx(entry: &FileEntry, mut content: &str) -> Block {
    let blockloc = Loc::for_entry(entry);
    let mut loc = blockloc.clone();
    loc.line = 1;
    loc.column = 1;
    // If the file ends with a ^Z, remove it.
    // A ^Z anywhere else might be an error, if it interrupts the game reading the file.
    // TODO: needs testing.
    if let Some(stripped) = content.strip_suffix('\u{001A}') {
        content = stripped;
    }
    parse(
        blockloc,
        &[Token::new(content.to_string(), loc)],
        LocalMacros::default(),
    )
}

pub fn parse_pdx_macro(inputs: &[Token], local_macros: LocalMacros) -> Block {
    parse(inputs[0].loc.clone(), inputs, local_macros)
}

pub fn parse_pdx_internal(input: &str, desc: &str) -> Block {
    let entry = FileEntry::new(PathBuf::from(desc), FileKind::Internal);
    let loc = Loc::for_entry(&entry);
    let input = Token::new(input.to_string(), loc.clone());
    parse(loc, &[input], LocalMacros::default())
}

// Simplified parsing just to get the macro arguments
pub fn split_macros(content: &Token) -> Vec<Token> {
    #[derive(Eq, PartialEq)]
    enum State {
        Normal,
        InQString,
        InComment,
    }
    let mut state = State::Normal;
    let mut vec = Vec::new();
    let mut loc = content.loc.clone();
    let mut last_loc = loc.clone();
    let mut last_pos = 0;
    for (i, c) in content.as_str().char_indices() {
        match state {
            State::InComment => {
                if c == '\n' {
                    state = State::Normal;
                }
            }
            State::InQString => {
                if c == '\n' || c == '"' {
                    state = State::Normal;
                }
            }
            State::Normal => {
                if c == '#' {
                    state = State::InComment;
                } else if c == '"' {
                    state = State::InQString;
                } else if c == '$' {
                    vec.push(Token::new(
                        content.as_str()[last_pos..i].to_string(),
                        last_loc,
                    ));
                    last_loc = loc.clone();
                    // Skip the current '$'
                    last_loc.column += 1;
                    last_pos = i + 1;
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
    vec.push(Token::new(
        content.as_str()[last_pos..].to_string(),
        last_loc,
    ));
    vec
}
