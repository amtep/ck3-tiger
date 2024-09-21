use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ProposalType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ProposalType, ProposalType::add)
}

impl ProposalType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ProposalType, key, block, Box::new(Self {}));
    }
}

impl DbKind for ProposalType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("texture", Item::File);
        vd.field_bool("has_dynamic_texture");
        vd.field_value("dynamic_texture_postfix");

        vd.field_value("open_popup");
        let mut sc = ScopeContext::new(Scopes::None, key);
        vd.field_target("days", &mut sc, Scopes::Value);
    }
}
