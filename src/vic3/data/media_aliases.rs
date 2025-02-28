use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashSet;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, untidy};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MediaAlias {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::MediaAlias, MediaAlias::add)
}

impl MediaAlias {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::MediaAlias, key, block, Box::new(Self {}));
    }
}

impl DbKind for MediaAlias {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("video", Item::File);
        vd.field_item("audio", Item::Sound);
        vd.field_item("texture", Item::File);
        vd.field_item("fallback", Item::MediaAlias);
        check_fallback_cycle(key, block, data);
    }
}

fn check_fallback_cycle(key: &Token, block: &Block, data: &Everything) {
    let mut fallback;
    if let Some(key) = block.get_field_value("fallback") {
        fallback = key;
    } else {
        return;
    }
    let mut seen = TigerHashSet::default();
    seen.insert(key.as_str());
    loop {
        if seen.contains(fallback.as_str()) {
            let msg = "fallbacks cycle back to the same key";
            // TODO: check if fatal
            untidy(ErrorKey::Loop).strong().msg(msg).loc(fallback).push();
            break;
        }
        seen.insert(fallback.as_str());
        if let Some((_, block)) = data.get_key_block(Item::MediaAlias, fallback.as_str()) {
            if let Some(key) = block.get_field_value("fallback") {
                fallback = key;
            } else {
                break;
            }
        } else {
            break;
        }
    }
}
