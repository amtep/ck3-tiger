//! Maintain the parser state for `@values` and `@:` directives.

use crate::block::Block;
use crate::helpers::TigerHashMap;
use crate::token::Token;

/// Global state for all files, set in the `reader_export` directory.
#[derive(Clone, Default, Debug)]
pub struct GlobalMemory {
    /// Pdx calls them variables even though they are constants.
    variables: TigerHashMap<String, Token>,
    /// Macros defined with `@:define`.
    blocks: TigerHashMap<String, Block>,
}

/// State defined locally in a file.
#[derive(Clone, Debug)]
pub struct LocalMemory<'memory> {
    global: &'memory GlobalMemory,
    /// Pdx calls them variables even though they are constants.
    variables: TigerHashMap<String, Token>,
    /// Macros defined with `@:define`.
    blocks: TigerHashMap<String, Block>,
}

impl<'memory> LocalMemory<'memory> {
    pub fn new(global: &'memory GlobalMemory) -> Self {
        LocalMemory { global, variables: TigerHashMap::default(), blocks: TigerHashMap::default() }
    }

    /// Get a previously set named value.
    pub fn get_variable(&self, key: &str) -> Option<&Token> {
        self.variables.get(key).or_else(|| self.global.variables.get(key))
    }

    /// Check if a variable has been defined previously.
    pub fn has_variable(&self, key: &str) -> bool {
        self.variables.contains_key(key) || self.global.variables.contains_key(key)
    }

    /// Insert a local value definition.
    pub fn set_variable(&mut self, key: String, value: Token) {
        self.variables.insert(key, value);
    }

    /// Retrieve a previously defined macro.
    pub fn get_block(&self, key: &str) -> Option<&Block> {
        self.blocks.get(key).or_else(|| self.global.blocks.get(key))
    }

    /// Check if a macro has been defined under this name.
    pub fn has_block(&self, key: &str) -> bool {
        self.blocks.contains_key(key) || self.global.blocks.contains_key(key)
    }

    /// Define a macro.
    pub fn define_block(&mut self, key: String, block: Block) {
        self.blocks.insert(key, block);
    }

    /// Merge the local memory into its cloned global memory.
    pub fn to_global(&self) -> GlobalMemory {
        let mut new = self.global.clone();
        for (key, value) in &self.variables {
            new.variables.insert(key.clone(), value.clone());
        }
        for (key, value) in &self.blocks {
            new.blocks.insert(key.clone(), value.clone());
        }
        new
    }
}
