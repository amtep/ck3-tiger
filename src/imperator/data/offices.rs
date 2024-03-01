use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Office {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::Office, Office::add)
}

impl Office {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Office, key.clone(), block, Box::new(Self {}));

        for office in &["civic_tech", "religious_tech", "oratory_tech", "military_tech"] {
            db.add_flag(Item::Office, Token::new(office, key.loc));
        }
    }
}

impl DbKind for Office {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("type", &["monarchy", "republic", "tribal"]);
        vd.field_choice("skill", &["martial", "charisma", "zeal", "finesse"]);

        vd.field_validated_block("skill_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country | ModifKinds::Character, vd);
        });

        vd.field_validated_block("personal_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
    }
}
