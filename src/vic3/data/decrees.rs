use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Decree {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Decree, Decree::add)
}

impl Decree {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Decree, key, block, Box::new(Self {}));
    }
}

impl DbKind for Decree {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::State, key);
        sc.define_name("country", Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("texture", Item::File);
        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::State | ModifKinds::Building, vd);
        });
        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_list_items("unlocking_laws", Item::LawType);
        vd.field_script_value("cost", &mut sc); // TODO: verify if a script value is allowed here
        vd.field_script_value("ai_weight", &mut sc);

        vd.replaced_field("valid", "country_trigger and state_trigger");
        vd.field_trigger_rooted("country_trigger", Tooltipped::Yes, Scopes::Country);
        vd.field_trigger_rooted("state_trigger", Tooltipped::Yes, Scopes::State);
    }
}
