use std::fs::read;
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

use anyhow::Result;
use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};

use crate::fileset::FileEntry;
use crate::report::ErrorLoc;
use crate::token::{Loc, Token};

#[derive(Clone, Debug)]
struct CsvParser<'a> {
    loc: Loc,
    offset: usize,
    content: &'a str,
    header_lines: usize,
    chars: Peekable<Chars<'a>>,
}

impl<'a> CsvParser<'a> {
    fn new(mut loc: Loc, header_lines: usize, content: &'a str) -> Self {
        loc.line = 1;
        loc.column = 1;
        let chars = content.chars().peekable();
        Self { loc, offset: 0, content, header_lines, chars }
    }

    fn next_char(&mut self) {
        // self.loc is always the loc of the peekable char
        if let Some(c) = self.chars.next() {
            self.offset += c.len_utf8();
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
            if c.is_ascii_whitespace() {
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

    /// Return the next CSV line, or None if at end of file
    fn parse_csv(&mut self) -> Option<Vec<Token>> {
        // Loop until we have a non-comment line.
        loop {
            self.skip_whitespace();
            if self.chars.peek() == Some(&'#') {
                self.skip_line();
            } else if self.header_lines > 0 {
                self.skip_line();
                self.header_lines -= 1;
            } else {
                break;
            }
        }
        self.chars.peek()?;

        let mut vec = Vec::new();
        let mut loc = self.loc.clone();
        let mut start_offset = self.offset;

        while let Some(c) = self.chars.peek() {
            match c {
                '#' | '\n' | ';' => {
                    let s = self.content[start_offset..self.offset].to_string();
                    vec.push(Token::new(s, loc));
                    if c == &';' {
                        self.next_char();
                        loc = self.loc.clone();
                        start_offset = self.offset;
                    } else {
                        break;
                    }
                }
                _ => self.next_char(),
            }
        }

        self.skip_line();
        Some(vec)
    }
}

pub struct CsvReader<'a> {
    parser: CsvParser<'a>,
}

impl<'a> Iterator for CsvReader<'a> {
    type Item = Vec<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.parse_csv()
    }
}

pub fn read_csv(fullpath: &Path) -> Result<String> {
    let bytes = read(fullpath)?;
    WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).map_err(anyhow::Error::msg)
}

#[allow(clippy::module_name_repetitions)]
pub fn parse_csv<'a>(entry: &FileEntry, header_lines: usize, content: &'a str) -> CsvReader<'a> {
    let parser = CsvParser::new(entry.into_loc(), header_lines, content);
    CsvReader { parser }
}
