use lalrpop_util::lalrpop_mod;

use crate::report::ErrorKey;

lalrpop_mod! {
    #[allow(clippy::pedantic)]
    #[allow(clippy::if_then_some_else_none)]
    parser, "/parse/ignore/parser.rs"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IgnoreSize {
    #[default]
    Line,
    Block,
    File,
    Begin,
    End,
}

#[derive(Debug, Clone, Default)]
pub struct IgnoreFilter {
    key: Option<ErrorKey>,
    text: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct IgnoreSpec {
    pub size: IgnoreSize,
    pub filter: IgnoreFilter,
}

pub fn parse_comment(comment: &str) -> Option<IgnoreSpec> {
    parser::CommentParser::new().parse(comment).ok()
}

impl IgnoreSpec {
    fn set_key(mut self, key: ErrorKey) -> Self {
        self.filter.key = Some(key);
        self
    }

    fn set_text(mut self, text: String) -> Self {
        self.filter.text = Some(text);
        self
    }

    fn merge(mut self, other: Self) -> Self {
        if other.size != IgnoreSize::Line {
            self.size = other.size;
        }
        if other.filter.key.is_some() {
            self.filter.key = other.filter.key;
        }
        if other.filter.text.is_some() {
            self.filter.text = other.filter.text;
        }
        self
    }
}

impl IgnoreFilter {
    pub fn matches(&self, other_key: ErrorKey, other_text: &str) -> bool {
        if let Some(key) = self.key {
            if key != other_key {
                return false;
            }
        }
        if let Some(text) = &self.text {
            if !other_text.contains(text) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bare() {
        let result = parse_comment("tiger-ignore");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::Line);
            assert!(spec.filter.key.is_none());
            assert!(spec.filter.text.is_none());
        }
    }

    #[test]
    fn test_non_ignore() {
        let result = parse_comment("a random comment");
        assert!(result.is_none());
    }

    #[test]
    fn test_bare_trailing() {
        let result = parse_comment("tiger-ignore with explanation");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::Line);
            assert!(spec.filter.key.is_none());
            assert!(spec.filter.text.is_none());
        }
    }

    #[test]
    fn test_block() {
        let result = parse_comment("tiger-ignore(block)");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::Block);
            assert!(spec.filter.key.is_none());
            assert!(spec.filter.text.is_none());
        }
    }

    #[test]
    fn test_file() {
        let result = parse_comment("tiger-ignore(file)");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::File);
            assert!(spec.filter.key.is_none());
            assert!(spec.filter.text.is_none());
        }
    }

    #[test]
    fn test_begin() {
        let result = parse_comment("tiger-ignore(begin)");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::Begin);
            assert!(spec.filter.key.is_none());
            assert!(spec.filter.text.is_none());
        }
    }

    #[test]
    fn test_end() {
        let result = parse_comment("tiger-ignore(end)");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::End);
            assert!(spec.filter.key.is_none());
            assert!(spec.filter.text.is_none());
        }
    }

    #[test]
    fn test_block_with_key() {
        let result = parse_comment("tiger-ignore(block, key=missing-item)");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::Block);
            assert_eq!(spec.filter.key, Some(ErrorKey::MissingItem));
            assert!(spec.filter.text.is_none());
        }
    }

    #[test]
    fn test_block_with_quoted_text() {
        let result = parse_comment("tiger-ignore(block, text=\"missing english\")");
        assert!(result.is_some());
        if let Some(spec) = result {
            assert_eq!(spec.size, IgnoreSize::Block);
            assert!(spec.filter.key.is_none());
            assert_eq!(spec.filter.text, Some("missing english".to_owned()));
        }
    }
}
