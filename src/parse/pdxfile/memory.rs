//! Maintain the parser state for `@values` and `@:` directives.

use crate::block::Block;
use crate::helpers::TigerHashMap;
use crate::token::Token;

/// Definitions retained by the parser, to handle @values and macros.
#[derive(Clone, Default, Debug)]
pub struct PdxfileMemory {
    /// Pdx calls them variables even though they are constants.
    variables: TigerHashMap<String, Token>,
    /// Macros defined with `@:define`.
    blocks: TigerHashMap<String, Block>,
}

impl PdxfileMemory {
    pub fn merge(&mut self, other: PdxfileMemory) {
        self.variables.extend(other.variables);
        self.blocks.extend(other.blocks);
    }
}

pub struct CombinedMemory<'global> {
    global: &'global PdxfileMemory,
    local: PdxfileMemory,
}

impl<'global> CombinedMemory<'global> {
    pub fn new(global: &'global PdxfileMemory) -> Self {
        Self { global, local: PdxfileMemory::default() }
    }

    pub fn from_local(global: &'global PdxfileMemory, local: PdxfileMemory) -> Self {
        Self { global, local }
    }

    /// Get a previously set named value.
    pub fn get_variable(&self, key: &str) -> Option<&Token> {
        self.local.variables.get(key).or_else(|| self.global.variables.get(key))
    }

    /// Check if a variable has been defined previously.
    pub fn has_variable(&self, key: &str) -> bool {
        self.local.variables.contains_key(key) || self.global.variables.contains_key(key)
    }

    /// Insert a local value definition.
    pub fn set_variable(&mut self, key: String, value: Token) {
        self.local.variables.insert(key, value);
    }

    /// Retrieve a previously defined macro.
    pub fn get_block(&self, key: &str) -> Option<&Block> {
        self.local.blocks.get(key).or_else(|| self.global.blocks.get(key))
    }

    /// Check if a macro has been defined under this name.
    pub fn has_block(&self, key: &str) -> bool {
        self.local.blocks.contains_key(key) || self.global.blocks.contains_key(key)
    }

    /// Define a macro.
    pub fn define_block(&mut self, key: String, block: Block) {
        self.local.blocks.insert(key, block);
    }

    /// Return the global part of the memory.
    pub fn as_global(&self) -> &PdxfileMemory {
        self.global
    }

    /// Clone the local part of the memory.
    pub fn get_local(&self) -> PdxfileMemory {
        self.local.clone()
    }
}
