use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashSet;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::pdxfile::PdxEncoding;
use crate::report::{untidy, warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Climate {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::Climate, PdxEncoding::Utf8OptionalBom, ".txt", LoadAsFile::No, Recursive::No, Climate::add)
}

impl Climate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Climate, key, block, Box::new(Self {}));
    }

    /// Check if there are any duplicate provinces between climate items.
    /// The check for duplicate provinces within one climate item is done in [`Climate::validate`].
    pub fn validate_all(db: &Db, _data: &Everything) {
        let mut other_seen: TigerHashSet<&Token> = TigerHashSet::default();
        for (_, block) in db.iter_key_block(Item::Climate) {
            for value in block.iter_values() {
                if let Some(&other) = other_seen.get(value) {
                    let msg = format!("province {value} has two climates");
                    warn(ErrorKey::Conflict)
                        .msg(msg)
                        .loc(value)
                        .loc_msg(other, "other climate")
                        .push();
                }
            }
            other_seen.extend(block.iter_values());
        }
    }
}

impl DbKind for Climate {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut seen: TigerHashSet<&Token> = TigerHashSet::default();
        for token in vd.values() {
            data.verify_exists(Item::Province, token);
            if let Some(&other) = seen.get(token) {
                let msg = format!("duplicate province {token}");
                untidy(ErrorKey::Conflict).msg(msg).loc(token).loc_msg(other, "other").push();
            } else {
                seen.insert(token);
            }
        }
    }
}
