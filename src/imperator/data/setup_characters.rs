use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::context::ScopeContext;
use crate::scopes::Scopes;
use crate::everything::Everything;
use crate::item::{Item, ItemLoader};
use crate::game::GameFlags;
use crate::pdxfile::PdxEncoding;
use crate::token::Token;
use crate::imperator::effect_validation::validate_create_character;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SetupCharacters {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::SetupCharacters, SetupCharacters::add)
}

impl SetupCharacters {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SetupCharacters, key, block, Box::new(Self {}));
    }
}

impl DbKind for SetupCharacters {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        vd.field_item("country", Item::Localization);
        vd.unknown_block_fields(|key, block| {
            let vd = Validator::new(block, data);
            validate_create_character(key, block, data, &mut sc, vd, Tooltipped::No);
        });
    }
}

#[derive(Clone, Debug)]
pub struct PostSetupCharacters {}

inventory::submit! {
    ItemLoader::Full(GameFlags::all(), Item::PostSetupCharacters, PdxEncoding::Utf8OptionalBom, ".txt", true, PostSetupCharacters::add)
}

impl PostSetupCharacters {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PostSetupCharacters, key, block, Box::new(Self {}));
    }
}

impl DbKind for PostSetupCharacters {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("country", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("countries", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.unknown_block_fields(|key, block| {
                    data.verify_exists(Item::Localization, key);
                    let mut vd = Validator::new(block, data);
                    vd.multi_field_validated_block("ruler_term", |block, data| {
                        let mut vd = Validator::new(block, data);
                        vd.field_integer("character");
                        vd.field_date("start_date");
                        vd.field_item("government", Item::GovernmentType);
                    });
                });
            });
        });
    }
}