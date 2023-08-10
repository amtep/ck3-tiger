use crate::validator::Validator;
use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;
use crate::context::ScopeContext;
use crate::scopes::Scopes;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::modif::{validate_modifs, ModifKinds};

#[derive(Clone, Debug)]
pub struct Idea {}

impl Idea {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Idea, key, block, Box::new(Self {}));
    }
}

impl DbKind for Idea {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("idea_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("trigger");
        vd.req_field("group");
        vd.req_field("soundeffect");

        vd.field_validated_block("trigger", |b, data| {
            validate_trigger(b, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_choice("group", &["military_ideas", "civic_ideas", "oratory_ideas", "religious_ideas"]);
        vd.field_item("soundeffect", Item::Sound);
        
        validate_modifs(block, data, ModifKinds::Country, vd);
    }
}