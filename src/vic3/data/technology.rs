use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Technology {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Technology, Technology::add)
}

impl Technology {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Technology, key, block, Box::new(Self {}));
    }
}

impl DbKind for Technology {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("era", Item::TechnologyEra);
        vd.field_item("texture", Item::File);
        vd.field_choice("category", &["production", "military", "society"]);

        vd.field_bool("can_research");
        vd.field_bool("should_update_map");

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });

        vd.field_list_items("unlocking_technologies", Item::Technology);

        vd.field_validated_block_rooted("on_researched", Scopes::Country, |block, data, sc| {
            validate_effect(block, data, sc, Tooltipped::No);
        });
        vd.field_script_value_rooted("ai_weight", Scopes::Country);
    }
}

#[derive(Clone, Debug)]
pub struct TechnologyEra {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::TechnologyEra, TechnologyEra::add)
}

impl TechnologyEra {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TechnologyEra, key, block, Box::new(Self {}));
    }
}

impl DbKind for TechnologyEra {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // Vanilla doesn't use these three
        vd.field_date("start_date");
        vd.field_date("end_date");
        vd.field_item("icon", Item::File);

        vd.field_numeric("technology_cost");
    }
}
