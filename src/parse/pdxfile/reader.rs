//! Remembers the `@` definitions seen by the parser.

use crate::block::Block;
use crate::helpers::TigerHashMap;
use crate::token::bump;

/// Tracks the @-values defined in this file.
/// Values starting with `@` are local to a file (except for the ones in `reader_export/`),
/// and are evaluated at parse time.
/// Other directives are explained in `reader_export/_reader_export.info` in the game files.
#[derive(Clone, Debug, Default)]
pub struct ReaderValues {
    /// @-values defined as numbers. Calculations can be done with these in `@[ ... ]` blocks.
    numeric: TigerHashMap<String, (f64, &'static str)>,
    /// @-values defined as text. These can be substituted at other locations in the script.
    text: TigerHashMap<String, &'static str>,
    /// Macros defined with `@:define`
    blocks: TigerHashMap<String, Block>,
}

impl ReaderValues {
    /// Get the value of a numeric @-value or numeric literal.
    ///
    /// The [`f64`] representation is lossy compared to the fixed-point numbers used in the script,
    /// but that hasn't been a problem so far.
    // TODO: the interface here is a bit confusing, the way it mixes number parsing with an actual
    // value lookup.
    pub fn get_value(&self, key: &str) -> Option<f64> {
        // key can be a local macro or a literal numeric value
        self.numeric.get(key).map(|(v, _)| v).copied().or_else(|| key.parse().ok())
    }

    /// Get the text form of a numeric or text @-value.
    pub fn get_as_str(&self, key: &str) -> Option<&'static str> {
        if let Some(value) = self.numeric.get(key) {
            Some(value.1)
        } else {
            self.text.get(key).copied()
        }
    }

    /// Check if a variable has been defined previously.
    pub fn has_variable(&self, key: &str) -> bool {
        self.numeric.contains_key(key) || self.text.contains_key(key)
    }

    /// Insert a local @-value definition.
    pub fn set_variable(&mut self, key: &str, value: &str) {
        let key = key.to_string();
        let value = bump(value);
        if let Ok(num) = value.parse::<f64>() {
            self.numeric.insert(key, (num, value));
        } else {
            self.text.insert(key, value);
        }
    }

    pub fn has_block(&self, key: &str) -> bool {
        self.blocks.contains_key(key)
    }

    pub fn define_block(&mut self, key: &str, block: Block) {
        self.blocks.insert(key.to_string(), block);
    }

    pub fn get_block(&self, key: &str) -> Option<&Block> {
        self.blocks.get(key)
    }
}
