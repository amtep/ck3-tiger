use crate::block::Block;
use crate::data::portrait::validate_dna_modifiers;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct TraitPortraitModifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::TraitPortraitModifier, TraitPortraitModifier::add)
}

impl TraitPortraitModifier {
    #[allow(clippy::needless_pass_by_value)] // about the unused key
    pub fn add(db: &mut Db, _key: Token, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            db.add(Item::TraitPortraitModifier, key, block, Box::new(Self {}));
        }
    }
}

impl DbKind for TraitPortraitModifier {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_list_items("traits", Item::Trait);
        vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Character);
        vd.field_item("base", Item::TraitPortraitModifier);
        vd.advice_field("dna_modifier", "docs say `dna_modifier` but it's `dna_modifiers`");
        vd.field_validated_block("dna_modifiers", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                data.verify_exists(Item::PortraitType, key);
                validate_dna_modifiers(block, data);
            });
        });
    }
}
