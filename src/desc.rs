use crate::block::{Block, BlockOrValue, DefinitionItem};
use crate::data::localization::Localization;
use crate::errorkey::ErrorKey;
use crate::errors::warn;

pub fn verify_desc_locas(desc: &BlockOrValue, locas: &Localization) {
    match desc {
        BlockOrValue::Token(t) => {
            if !t.as_str().contains(' ') {
                locas.verify_exists(t.as_str(), t);
            }
        }
        BlockOrValue::Block(b) => {
            verify_desc_block_locas(b, locas);
        }
    }
}

pub fn verify_desc_block_locas(desc: &Block, locas: &Localization) {
    for def in desc.iter_definitions_warn() {
        match def {
            DefinitionItem::Assignment(key, t) if key.is("desc") => {
                if !t.as_str().contains(' ') {
                    locas.verify_exists(t.as_str(), t);
                }
            }
            DefinitionItem::Assignment(key, _) | DefinitionItem::Keyword(key) => {
                warn(
                    key,
                    ErrorKey::Validation,
                    &format!("unexpected key in description"),
                );
            }
            DefinitionItem::Definition(key, b) => {
                if key.is("desc")
                    || key.is("first_valid")
                    || key.is("random_valid")
                    || key.is("triggered_desc")
                {
                    verify_desc_block_locas(b, locas);
                } else if key.is("trigger") {
                    continue;
                } else {
                    warn(
                        key,
                        ErrorKey::Validation,
                        &format!("unexpected key in description"),
                    );
                }
            }
        }
    }
}
