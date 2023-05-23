use std::iter::Peekable;
use std::str::Chars;

use crate::data::localization::{get_file_lang, LocaEntry, LocaValue, MacroValue};
use crate::datatype::{Code, CodeArg, CodeChain};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::fileset::FileEntry;
use crate::token::{Loc, Token};

fn is_key_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '\''
}

// code_char might end up being identical to key_char, since we can write [gameconcept] and
// game concepts can be any id
fn is_code_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\''
}

#[derive(Clone, Debug)]
struct LocaParser<'a> {
    loc: Loc,
    content: &'a str,
    chars: Peekable<Chars<'a>>,
    language: &'static str,
    expecting_language: bool,
    loca_end: usize,
    value: Vec<LocaValue>,
}

impl<'a> LocaParser<'a> {
    fn new(mut loc: Loc, content: &'a str, lang: &'static str) -> Self {
        let mut chars = content.chars().peekable();
        if chars.peek() == Some(&'\u{feff}') {
            loc.offset += '\u{feff}'.len_utf8();
            chars.next();
        } else {
            warn(&loc, ErrorKey::Encoding, "Expected UTF-8 BOM encoding");
        }
        LocaParser {
            loc,
            content,
            chars,
            language: lang,
            expecting_language: true,
            value: Vec::new(),
            loca_end: 0,
        }
    }

    fn next_char(&mut self) {
        // self.loc is always the loc of the peekable char
        if let Some(c) = self.chars.next() {
            self.loc.offset += c.len_utf8();
            if c == '\n' {
                self.loc.line += 1;
                self.loc.column = 1;
            } else {
                self.loc.column += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_linear_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() && *c != '\n' {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_line(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                break;
            }
            self.next_char();
        }
        self.next_char(); // Eat the newline
    }

    // This function returns an Option so that the caller can return
    // its value without further boilerplate.
    #[allow(clippy::unnecessary_wraps)]
    fn error_line(&mut self, key: Token) -> Option<LocaEntry> {
        self.skip_line();
        Some(LocaEntry {
            key,
            value: LocaValue::Error,
            orig: None,
        })
    }

    fn get_key(&mut self) -> Token {
        let loc = self.loc.clone();
        while let Some(c) = self.chars.peek() {
            if is_key_char(*c) {
                self.next_char();
            } else {
                break;
            }
        }
        let s = self.content[loc.offset..self.loc.offset].to_string();
        Token::new(s, loc)
    }

    fn unexpected_char(&mut self, expected: &str) {
        match self.chars.peek() {
            None => error(
                &self.loc,
                ErrorKey::Localization,
                &format!("Unexpected end of file, {expected}"),
            ),
            Some(c) => error(
                &self.loc,
                ErrorKey::Localization,
                &format!("Unexpected character `{c}`, {expected}"),
            ),
        };
    }

    // Look ahead to the last `"` on the line
    fn find_dquote(&self) -> Option<usize> {
        let mut offset = self.loc.offset;
        let mut dquote_offset = None;
        for c in self.chars.clone() {
            if c == '"' {
                dquote_offset = Some(offset);
            } else if c == '\n' {
                return dquote_offset;
            }
            offset += c.len_utf8();
        }
        dquote_offset
    }

    fn parse_format(&mut self) -> Option<Token> {
        if self.chars.peek() == Some(&'|') {
            self.next_char(); // eat the |
            let loc = self.loc.clone();
            let mut text = String::new();
            while let Some(&c) = self.chars.peek() {
                if c == '$' || c == ']' || c == '\n' {
                    break;
                }
                text.push(c);
                self.next_char();
            }
            Some(Token::new(text, loc))
        } else {
            None
        }
    }

    fn line_has_macros(&self) -> bool {
        for c in self.chars.clone() {
            if c == '\n' {
                return false;
            } else if c == '$' {
                return true;
            }
        }
        false
    }

    fn parse_macros(&mut self) {
        // TODO: vanilla uses $[DATE_MIN.GetStringShort|V]$ which breaks all my assumptions
        let mut v = Vec::new();
        let mut loc = self.loc.clone();
        while let Some(&c) = self.chars.peek() {
            if c == '$' {
                let s = self.content[loc.offset..self.loc.offset].to_string();
                v.push(MacroValue::Text(Token::new(s, loc)));

                if let Some(mv) = self.parse_keyword() {
                    v.push(mv);
                } else {
                    self.value.push(LocaValue::Error);
                    return;
                }
                loc = self.loc.clone();
            } else if c == '"' && self.loc.offset == self.loca_end {
                let s = self.content[loc.offset..self.loc.offset].to_string();
                v.push(MacroValue::Text(Token::new(s, loc)));
                self.value.push(LocaValue::Macro(v));
                self.next_char();
                return;
            } else {
                self.next_char();
            }
        }
        let s = self.content[loc.offset..self.loc.offset].to_string();
        v.push(MacroValue::Text(Token::new(s, loc)));
        self.value.push(LocaValue::Macro(v));
    }

    fn parse_keyword(&mut self) -> Option<MacroValue> {
        self.next_char(); // Skip the $
        let loc = self.loc.clone();
        let key = self.get_key();
        let end_loc = self.loc.clone();
        let format = self.parse_format();
        if self.chars.peek() != Some(&'$') {
            // TODO: check if there is a closing $, adapt warning text
            warn(
                key,
                ErrorKey::Localization,
                "didn't recognize a key between $",
            );
            return None;
        }
        let s = self.content[loc.offset..end_loc.offset].to_string();
        self.next_char();
        Some(MacroValue::Keyword(Token::new(s, loc), format))
    }

    fn skip_until_key(&mut self) {
        loop {
            // Skip comments and blank lines
            self.skip_whitespace();
            if self.chars.peek() == Some(&'#') {
                self.skip_line();
                continue;
            }

            match self.chars.peek() {
                Some(&c) if is_key_char(c) => break,
                Some(_) => {
                    self.unexpected_char("expected localization key");
                    self.skip_line();
                }
                None => break,
            }
        }
    }

    /// Return the next `LocaEntry`, or None if there are no more in the file.
    fn parse_loca(&mut self) -> Option<LocaEntry> {
        // Loop until we have a key. Once we have a key, we'll definitely
        // return a LocaEntry for the current line, though it might be an Error entry.
        self.skip_until_key();
        self.chars.peek()?;

        let key = self.get_key();
        self.skip_linear_whitespace();
        if self.chars.peek() == Some(&':') {
            self.next_char();
        } else {
            self.unexpected_char("expected `:`");
            return self.error_line(key);
        }

        // Skip optional number after :
        while let Some(c) = self.chars.peek() {
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }
        self.skip_linear_whitespace();

        // Now we should see the value. But what if the line ends here?
        if matches!(self.chars.peek(), Some('#' | '\n') | None) {
            if self.expecting_language {
                if !key.is(&format!("l_{}", self.language)) {
                    error(
                        key,
                        ErrorKey::Localization,
                        &format!("wrong language header, should be `l_{}:`", self.language),
                    );
                }
                self.expecting_language = false;
                self.skip_line();
                // Recursing here is safe because it can happen only once.
                return self.parse_loca();
            }
            error(&key, ErrorKey::Localization, "key with no value");
            return self.error_line(key);
        } else if self.expecting_language {
            error(
                &key,
                ErrorKey::Localization,
                &format!("expected language header `l_{}:`", self.language),
            );
            self.expecting_language = false;
            // Continue to parse this entry as usual
        }
        if self.chars.peek() == Some(&'"') {
            self.next_char();
        } else {
            self.unexpected_char("expected `\"`");
            return self.error_line(key);
        }

        // We need to pre-parse because the termination of localization entries
        // is ambiguous. A loca value ends at the last " on the line.
        // Any # or " before that are part of the value; an # after that
        // introduces a comment.
        if let Some(i) = self.find_dquote() {
            self.loca_end = i;
        } else {
            let msg = "localization entry without ending quote";
            error(&self.loc, ErrorKey::Localization, msg);
            return self.error_line(key);
        }

        self.value = Vec::new();
        let s = self.content[self.loc.offset..self.loca_end].to_string();
        let token = Token::new(s, self.loc.clone());

        // We also need to pre-parse because $macros$ can appear anywhere and
        // we don't know how to parse the results until we know what to
        // substitute. If there are macros in the line, return it as a special
        // `LocaValue::Macro` array
        if self.line_has_macros() {
            self.parse_macros();
            if matches!(self.value.last(), Some(&LocaValue::Error)) {
                return self.error_line(key);
            }
        } else {
            let mut parser = ValueParser::new(vec![&token]);
            self.value = parser.parse_value();
            while self.loc.offset <= self.loca_end {
                self.next_char();
            }
        }

        self.skip_linear_whitespace();
        match self.chars.peek() {
            None | Some('#' | '\n') => (),
            _ => {
                let msg = "content after final `\"` on line";
                warn(&self.loc, ErrorKey::Localization, msg);
            }
        }

        self.skip_line();
        let value = if self.value.len() == 1 {
            std::mem::take(&mut self.value[0])
        } else {
            LocaValue::Concat(std::mem::take(&mut self.value))
        };
        Some(LocaEntry {
            key,
            value,
            orig: Some(token),
        })
    }
}

pub struct ValueParser<'a> {
    loc: Loc,
    content: Vec<&'a Token>,
    content_iters: Vec<Peekable<Chars<'a>>>,
    content_idx: usize,
    value: Vec<LocaValue>,
}

// TODO: some duplication of helper functions between `LocaParser` and `ValueParser`
impl<'a> ValueParser<'a> {
    pub fn new(content: Vec<&'a Token>) -> Self {
        assert!(!content.is_empty());

        Self {
            loc: content[0].loc.clone(),
            content_iters: content
                .iter()
                .map(|t| t.as_str().chars().peekable())
                .collect(),
            content,
            content_idx: 0,
            value: Vec::new(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        let p = self.content_iters[self.content_idx].peek();
        if p.is_none() {
            if self.content_idx + 1 == self.content.len() {
                None
            } else {
                self.content_idx += 1;
                self.loc = self.content[self.content_idx].loc.clone();
                self.peek()
            }
        } else {
            p.copied()
        }
    }

    fn next_char(&mut self) {
        // self.peek advances content_idx if needed
        if self.peek().is_some() {
            if let Some(c) = self.content_iters[self.content_idx].next() {
                self.loc.offset += c.len_utf8();
                self.loc.column += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn unexpected_char(&mut self, expected: &str, errorkey: ErrorKey) {
        // TODO: handle EOF better
        let c = self.peek().unwrap_or(' ');
        let msg = format!("Unexpected character `{c}`, {expected}");
        error(&self.loc, errorkey, &msg);
    }

    fn get_key(&mut self) -> Token {
        let loc = self.loc.clone();
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if is_key_char(c) {
                text.push(c);
                self.next_char();
            } else {
                break;
            }
        }
        Token::new(text, loc)
    }

    fn parse_format(&mut self) -> Option<Token> {
        if self.peek() == Some('|') {
            self.next_char(); // eat the |
            let loc = self.loc.clone();
            let mut text = String::new();
            while let Some(c) = self.peek() {
                if c == '$' || c == ']' {
                    break;
                }
                text.push(c);
                self.next_char();
            }
            Some(Token::new(text, loc))
        } else {
            None
        }
    }

    fn parse_code_args(&mut self) -> Vec<CodeArg> {
        self.next_char(); // eat the opening (
        let mut v = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek() == Some('\'') {
                self.next_char();
                let loc = self.loc.clone();
                let mut text = String::new();
                let mut parens: isize = 0;
                while let Some(c) = self.peek() {
                    match c {
                        '\'' => break,
                        ']' | ')' if parens == 0 => warn(
                            &self.loc,
                            ErrorKey::Localization,
                            "Possible unterminated argument string",
                        ),
                        '(' => parens += 1,
                        ')' => parens -= 1,
                        _ => (),
                    }
                    text.push(c);
                    self.next_char();
                }
                if self.peek() != Some('\'') {
                    self.value.push(LocaValue::Error);
                    return Vec::new();
                }
                v.push(CodeArg::Literal(Token::new(text, loc)));
                self.next_char();
            } else if self.peek() == Some(')') {
                // Empty () means no arguments
            } else {
                v.push(CodeArg::Chain(self.parse_code_inner()));
            }
            self.skip_whitespace();
            if self.peek() != Some(',') {
                break;
            }
            self.next_char(); // Eat the comma
        }
        if self.peek() == Some(')') {
            self.next_char();
        } else {
            self.unexpected_char("expected `)`", ErrorKey::Datafunctions);
        }
        v
    }

    fn parse_code_code(&mut self) -> Code {
        let loc = self.loc.clone();
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if is_code_char(c) {
                text.push(c);
                self.next_char();
            } else {
                break;
            }
        }
        let name = Token::new(text, loc);
        if self.peek() == Some('(') {
            Code {
                name,
                arguments: self.parse_code_args(),
            }
        } else {
            Code {
                name,
                arguments: Vec::new(),
            }
        }
    }

    fn parse_code_inner(&mut self) -> CodeChain {
        let mut v = Vec::new();
        loop {
            v.push(self.parse_code_code());
            if self.peek() != Some('.') {
                break;
            }
            self.next_char(); // Eat the '.'
        }
        CodeChain { codes: v }
    }

    fn parse_code(&mut self) {
        self.next_char(); // eat the opening [
        self.skip_whitespace();

        let chain = self.parse_code_inner();

        self.skip_whitespace();
        let format = self.parse_format();
        if self.peek() == Some(']') {
            self.next_char();
            self.value.push(LocaValue::Code(chain, format));
        } else {
            self.unexpected_char("expected `]`", ErrorKey::Datafunctions);
            self.value.push(LocaValue::Error);
        }
    }

    fn parse_markup(&mut self) {
        let loc = self.loc.clone();
        let mut text = "#".to_string();
        self.next_char(); // skip the #
        if self.peek() == Some('!') {
            text.push('!');
            self.next_char();
            self.value.push(LocaValue::MarkupEnd(Token::new(text, loc)));
        } else {
            // TODO This may have become complicated enough to need its own state machine
            let mut at_end = false;
            while let Some(c) = self.peek() {
                if c.is_whitespace() {
                    self.next_char();
                    break;
                } else if !at_end && (is_key_char(c) || c == ';') {
                    // TODO: check that ';' is a separator, not at the beginning or end
                    text.push(c);
                    self.next_char();
                } else if !at_end && c == ':' {
                    // #indent_newline:2 parsing
                    // #color:{1.0,1.0,1.0} parsing
                    text.push(c);
                    self.next_char();
                    let mut in_braces = false; // only one level of braces
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() || in_braces || c == '{' {
                            if c == '{' {
                                in_braces = true;
                            } else if c == '}' {
                                in_braces = false;
                            }
                            text.push(c);
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    at_end = true;
                } else {
                    let msg = "#markup should be followed by a space";
                    warn(loc, ErrorKey::Localization, msg);
                    self.value.push(LocaValue::Error);
                    return;
                }
            }
            self.value.push(LocaValue::Markup(Token::new(text, loc)));
        }
    }

    fn parse_icon(&mut self) {
        self.next_char(); // eat the @

        if let Some(c) = self.peek() {
            if is_key_char(c) {
                let key = self.get_key();
                self.value.push(LocaValue::Icon(key));
            } else {
                self.unexpected_char("expected icon name", ErrorKey::Localization);
                self.value.push(LocaValue::Error);
                return;
            }
        } else {
            self.unexpected_char("expected icon name", ErrorKey::Localization);
            self.value.push(LocaValue::Error);
            return;
        }

        if self.peek() == Some('!') {
            self.next_char();
        } else {
            self.unexpected_char("expected `!`", ErrorKey::Localization);
            self.value.push(LocaValue::Error);
        }
    }

    fn parse_escape(&mut self) {
        let loc = self.loc.clone();
        self.next_char(); // Skip the \
        let s = match self.peek() {
            Some('n') => '\n'.to_string(),
            Some(c) => c.to_string(),
            None => {
                self.value.push(LocaValue::Error);
                return;
            }
        };
        self.next_char();
        self.value.push(LocaValue::Text(Token::new(s, loc)));
    }

    fn parse_text(&mut self) {
        let loc = self.loc.clone();
        let mut text = String::new();
        while let Some(c) = self.peek() {
            match c {
                '[' | '#' | '@' | '\\' => break,
                _ => {
                    text.push(c);
                    self.next_char();
                }
            }
        }
        self.value.push(LocaValue::Text(Token::new(text, loc)));
    }

    pub fn parse_value(&mut self) -> Vec<LocaValue> {
        while let Some(c) = self.peek() {
            match c {
                '[' => self.parse_code(),
                '#' => self.parse_markup(),
                '@' => self.parse_icon(),
                '\\' => self.parse_escape(),
                _ => self.parse_text(),
            }
            if matches!(self.value.last(), Some(&LocaValue::Error)) {
                return vec![LocaValue::Error];
            }
        }
        std::mem::take(&mut self.value)
    }
}

pub struct LocaReader<'a> {
    parser: LocaParser<'a>,
}

impl<'a> Iterator for LocaReader<'a> {
    type Item = LocaEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.parse_loca()
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn parse_loca<'a>(entry: &FileEntry, content: &'a str) -> LocaReader<'a> {
    // lang can be unwrapped here because we wouldn't be called to parse a bad filename
    let lang = get_file_lang(entry.path().file_name().unwrap()).unwrap();
    let mut loc = Loc::for_entry(entry);
    loc.line = 1;
    loc.column = 1;
    let parser = LocaParser::new(loc, content, lang);
    LocaReader { parser }
}
