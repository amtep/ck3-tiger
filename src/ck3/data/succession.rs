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
pub struct SuccessionAppointment {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SuccessionAppointment, SuccessionAppointment::add)
}

impl SuccessionAppointment {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SuccessionAppointment, key, block, Box::new(Self {}));
    }
}

impl DbKind for SuccessionAppointment {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("title", Scopes::LandedTitle, key);

        vd.field_script_value("candidate_score", &mut sc);
        vd.field_list_choice("default_candidates", CANDIDATE_TYPES);
        vd.field_list_choice("invested_candidates", CANDIDATE_TYPES);

        vd.field_bool("allow_children");
    }
}

const CANDIDATE_TYPES: &[&str] = &[
    "holder_close_family",
    "holder_close_extended_family",
    "holder_house_member",
    "landed_vassal",
    "landed_vassal_close_family",
    "landed_vassal_close_extended_family",
    "landed_vassal_house_member",
    "unlanded_noble_house_head",
    "unlanded_noble_close_family",
    "unlanded_noble_close_extended_family",
    "unlanded_noble_house_member",
    "holder_councilor",
    "holder_court_position",
    "direct_subject",
];
