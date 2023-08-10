use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CharacterTrait {}

impl CharacterTrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterTrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterTrait {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("type", &["health", "military", "personality", "status"]);

        vd.field_bool("congenital");

        vd.field_validated_block("unit", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("country", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        // TODO - dna_modifiers block should be ignored
        vd.no_warn_remaining();

        validate_modifs(block, data, ModifKinds::Character, vd);
    }
}
