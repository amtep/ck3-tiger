use crate::scope::{Loc, Token}

#[derive(Clone, Debug)]
pub struct LocaEntry {
    key: Token,
    value: LocaValue,
}

#[derive(Clone, Debug)]
struct LocaParser {
    loc: Loc,
    content: &str,
    chars: (),
}

impl LocaParser {
    fn new(loc: Loc, content: &str) -> Self {
        LocaParser { loc, content, chars: content.char_indices().peekable() }
    }

    fn parse_loca(&mut self) -> Option<LocaEntry> {
        // We need to pre-parse because the termination of localization entries
        // is ambiguous. A loca value ends at the last " on the line.
        // Any # or " before that are part of the value; an # after that
        // introduces a comment.
        let mut chars2 = self.chars.clone();
        let mut last_dquote = None;
        for (i, c) in self.chars2 {
            if c == '"' {
                last_dquote = Some(i);
            } else if c == '\n' {
                break;
            }
        }


    }
}

impl Iterator for LocaParser {
    type Item = LocaEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_loca()
    }
}

pub fn parse_loca(pathname: &Path, kind: FileKind, content: &str) -> LocaParser {
    LocaParser::new(Loc::new(Rc::new(pathname.to_path_buf()), kind), content);
}
