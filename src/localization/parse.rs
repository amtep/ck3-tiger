use std::iter::Peekable;
use std::mem::swap;
use std::path::Path;
use std::rc::Rc;
use std::str::Chars;

use crate::errors::{error, warn, ErrorKey};
use crate::everything::FileKind;
use crate::localization::{get_file_lang, Code, CodeArg, CodeChain, LocaEntry, LocaValue};
use crate::scope::{Loc, Token};

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

fn is_key_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '\''
}

fn is_code_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'a> LocaParser<'a> {
    fn new(mut loc: Loc, content: &'a str, lang: &'static str) -> Self {
        let mut chars = content.chars().peekable();
        if chars.peek() == Some(&'\u{feff}') {
            loc.offset += '\u{feff}'.len_utf8();
            chars.next();
        } else {
            warn(
                &Token::from(&loc),
                ErrorKey::Encoding,
                "Expected UTF-8 BOM encoding",
            );
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
            } else {
                self.next_char();
            }
        }
        self.next_char();
    }

    fn error_line(&mut self, key: Token) -> Option<LocaEntry> {
        self.skip_line();
        Some(LocaEntry {
            key,
            value: LocaValue::Error,
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
        error(
            &Token::from(&self.loc),
            ErrorKey::Localization,
            &format!(
                "Unexpected character {:?}, {}",
                self.chars.peek().unwrap(),
                expected
            ),
        );
    }

    // Look ahead to the last `"` on the line
    fn find_dquote(&mut self) -> Option<usize> {
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

    fn parse_text(&mut self) {
        let loc = self.loc.clone();
        while let Some(c) = self.chars.peek() {
            match c {
                '[' | '#' | '$' | '@' | '\\' => break,
                '"' if self.loc.offset == self.loca_end => break,
                _ => self.next_char(),
            }
        }
        let s = self.content[loc.offset..self.loc.offset].to_string();
        self.value.push(LocaValue::Text(Token::new(s, loc)));
    }

    fn parse_code_args(&mut self) -> Vec<CodeArg> {
        self.next_char(); // eat the opening (
        let mut v = Vec::new();

        loop {
            self.skip_linear_whitespace();
            if self.chars.peek() == Some(&'\'') {
                self.next_char();
                let loc = self.loc.clone();
                while let Some(&c) = self.chars.peek() {
                    match c {
                        '\'' | '\n' | '"' => break,
                        ')' | ']' => warn(
                            &Token::from(&self.loc),
                            ErrorKey::Localization,
                            "Possible unterminated argument string",
                        ),
                        _ => {}
                    }
                    self.next_char();
                }
                if self.chars.peek() != Some(&'\'') {
                    self.value.push(LocaValue::Error);
                    return Vec::new();
                }
                let s = self.content[loc.offset..self.loc.offset].to_string();
                v.push(CodeArg::Literal(Token::new(s, loc)));
                self.next_char();
            } else {
                CodeArg::Chain(self.parse_code_inner());
            }
            self.skip_linear_whitespace();
            if self.chars.peek() != Some(&',') {
                break;
            } else {
                self.next_char();
            }
        }
        if self.chars.peek() == Some(&')') {
            self.next_char();
        } else {
            self.unexpected_char("expected `)`");
        }
        v
    }

    fn parse_code_code(&mut self) -> Code {
        let loc = self.loc.clone();
        while let Some(&c) = self.chars.peek() {
            if is_code_char(c) {
                self.next_char();
            } else {
                break;
            }
        }
        let s = self.content[loc.offset..self.loc.offset].to_string();
        let name = Token::new(s, loc);
        if self.chars.peek() == Some(&'(') {
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
            if self.chars.peek() != Some(&'.') {
                break;
            }
            self.next_char(); // Eat the '.'
        }
        CodeChain { codes: v }
    }

    fn parse_format(&mut self) -> Option<Token> {
        if self.chars.peek() == Some(&'|') {
            self.next_char(); // eat the |
            let loc = self.loc.clone();
            while let Some(&c) = self.chars.peek() {
                if c == '$' || c == ']' || c == '\n' {
                    break;
                } else {
                    self.next_char();
                }
            }
            let s = self.content[loc.offset..self.loc.offset].to_string();
            Some(Token::new(s, loc))
        } else {
            None
        }
    }

    fn parse_code(&mut self) {
        self.next_char(); // eat the opening [
        self.skip_linear_whitespace();

        let chain = self.parse_code_inner();

        self.skip_linear_whitespace();
        let format = self.parse_format();
        if self.chars.peek() == Some(&']') {
            self.next_char();
            self.value.push(LocaValue::Code(chain, format));
        } else {
            self.unexpected_char("expected `]`");
            self.value.push(LocaValue::Error);
        }
    }

    fn parse_markup(&mut self) {
        let loc = self.loc.clone();
        self.next_char(); // skip the #
        if self.chars.peek() == Some(&'!') {
            self.next_char();
            let s = self.content[loc.offset..self.loc.offset].to_string();
            self.value.push(LocaValue::MarkupEnd(Token::new(s, loc)));
        } else {
            while let Some(&c) = self.chars.peek() {
                if c.is_whitespace() {
                    self.next_char();
                    break;
                } else if is_key_char(c) {
                    self.next_char();
                } else {
                    warn(
                        &Token::from(loc),
                        ErrorKey::Localization,
                        "#markup should be followed by a space",
                    );
                    self.value.push(LocaValue::Error);
                    return;
                }
            }
            let s = self.content[loc.offset..self.loc.offset].to_string();
            self.value.push(LocaValue::Markup(Token::new(s, loc)));
        }
    }

    // TODO: check if you can put code between $ $
    fn parse_keyword(&mut self) {
        self.next_char(); // Skip the $
        let loc = self.loc.clone();
        let key = self.get_key();
        let format = self.parse_format();
        if self.chars.peek() != Some(&'$') {
            // TODO: check if there is a closing $, adapt warning text
            warn(
                &key,
                ErrorKey::Localization,
                "didn't recognize the key between $",
            );
            self.value.push(LocaValue::Error);
            return;
        }
        let s = self.content[loc.offset..self.loc.offset].to_string();
        self.value
            .push(LocaValue::Keyword(Token::new(s, loc), format));
        self.next_char();
    }

    fn parse_icon(&mut self) {
        let mut value = Vec::new();
        swap(&mut value, &mut self.value);
        self.next_char(); // eat the @

        while let Some(&c) = self.chars.peek() {
            match c {
                '$' => self.parse_keyword(),
                '\\' => self.parse_escape(),
                '!' => break,
                c if is_key_char(c) => {
                    let key = self.get_key();
                    self.value.push(LocaValue::Text(key));
                }
                _ => {
                    self.unexpected_char("expected icon name");
                    swap(&mut value, &mut self.value);
                    self.value.push(LocaValue::Error);
                    return;
                }
            }
            if matches!(self.value.last(), Some(&LocaValue::Error)) {
                return;
            }
        }
        swap(&mut value, &mut self.value);
        self.value.push(LocaValue::Icon(value));
    }

    fn parse_escape(&mut self) {
        let loc = self.loc.clone();
        self.next_char(); // Skip the \
        let s = match self.chars.peek() {
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

    /// Return the next `LocaEntry`, or None if there are no more in the file.
    fn parse_loca(&mut self) -> Option<LocaEntry> {
        // Loop until we have a key. Once we have a key, we'll definitely
        // return a LocaEntry for the current line, though it might be an Error.
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
                    continue;
                }
                None => return None,
            }
        }

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
        if matches!(self.chars.peek(), Some('#') | Some('\n') | None) {
            if self.expecting_language {
                if key.as_str() != format!("l_{}", self.language) {
                    error(
                        &key,
                        ErrorKey::Localization,
                        &format!("wrong language header, should be `l_{}:`", self.language),
                    );
                }
                self.expecting_language = false;
                self.skip_line();
                // Recursing here is safe because it can happen only once.
                return self.parse_loca();
            } else {
                error(&key, ErrorKey::Localization, "key with no value");
                return self.error_line(key);
            }
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
        self.loca_end = match self.find_dquote() {
            Some(i) => i,
            None => {
                error(
                    &Token::from(&self.loc),
                    ErrorKey::Localization,
                    "localization entry without ending quote",
                );
                return self.error_line(key);
            }
        };

        self.value = Vec::new();
        while let Some(c) = self.chars.peek() {
            match c {
                '[' => self.parse_code(),
                '#' => self.parse_markup(),
                '$' => self.parse_keyword(),
                '@' => self.parse_icon(),
                '\\' => self.parse_escape(),
                '"' if self.loc.offset == self.loca_end => {
                    self.next_char();
                    break;
                }
                _ => self.parse_text(),
            }
            if matches!(self.value.last(), Some(&LocaValue::Error)) {
                return self.error_line(key);
            }
        }

        self.skip_linear_whitespace();
        match self.chars.peek() {
            None | Some('#') | Some('\n') => (),
            _ => {
                warn(
                    &Token::from(&self.loc),
                    ErrorKey::Localization,
                    "content after final `\"` on line",
                );
            }
        }

        self.skip_line();
        Some(LocaEntry {
            key,
            value: LocaValue::Concat(std::mem::take(&mut self.value)),
        })
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

pub fn parse_loca<'a>(pathname: &Path, kind: FileKind, content: &'a str) -> LocaReader<'a> {
    // lang can be unwrapped here because we wouldn't be called to parse a bad filename
    let lang = get_file_lang(pathname.file_name().unwrap()).unwrap();
    let parser = LocaParser::new(
        Loc::new(Rc::new(pathname.to_path_buf()), kind),
        content,
        lang,
    );
    LocaReader { parser }
}
