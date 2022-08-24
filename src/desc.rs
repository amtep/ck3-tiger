use crate::block::{Block, BlockOrValue, DefinitionItem};
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::localization::Localization;

pub fn verify_desc_locas(desc: &BlockOrValue, locas: &Localization, context: &str) {
    match desc {
        BlockOrValue::Token(t) => {
            if !t.as_str().contains(' ') {
                locas.verify_have_key(t.as_str(), t, context);
            }
        }
        BlockOrValue::Block(b) => {
            verify_desc_block_locas(b, locas, context);
        }
    }
}

pub fn verify_desc_block_locas(desc: &Block, locas: &Localization, context: &str) {
    for def in desc.iter_definitions_warn() {
        match def {
            DefinitionItem::Assignment(key, t) if key.is("desc") => {
                if !t.as_str().contains(' ') {
                    locas.verify_have_key(t.as_str(), t, context);
                }
            }
            DefinitionItem::Assignment(key, _) | DefinitionItem::Keyword(key) => {
                warn(
                    key,
                    ErrorKey::Validation,
                    &format!("unexpected key in {}", context),
                );
            }
            DefinitionItem::Definition(key, b) => {
                if key.is("desc")
                    || key.is("first_valid")
                    || key.is("random_valid")
                    || key.is("triggered_desc")
                {
                    verify_desc_block_locas(b, locas, context);
                } else if key.is("trigger") {
                    continue;
                } else {
                    warn(
                        key,
                        ErrorKey::Validation,
                        &format!("unexpected key in {}", context),
                    );
                }
            }
        }
    }
}
