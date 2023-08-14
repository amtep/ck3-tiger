use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SubjectType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::SubjectType, SubjectType::add)
}

impl SubjectType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SubjectType, key, block, Box::new(Self {}));
    }
}

impl DbKind for SubjectType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_bool("allow_change_country_flag");
        vd.field_bool("use_overlord_map_color");
        vd.field_bool("use_overlord_ruler");
        vd.field_bool("annex_on_country_formation");
        vd.field_bool("can_start_own_diplomatic_plays");
        vd.field_bool("breaks_if_subject_not_protected");
        vd.field_bool("join_overlord_wars");
        vd.field_bool("can_have_subjects");
        vd.field_bool("overlord_must_be_higher_rank");
        vd.field_bool("overlord_must_be_same_country_type");
        vd.field_bool("use_for_release_country");
        vd.field_bool("gives_prestige_to_overlord");
        vd.field_bool("subservient_to_overlord");

        vd.field_numeric("convoy_contribution");

        vd.field_item("diplomatic_action", Item::DiplomaticAction);

        vd.field_list_items("country_type_change_alternatives", Item::SubjectType);
        vd.field_list_items("can_change_subject_type_from", Item::SubjectType);
        vd.field_list_items("valid_overlord_country_types", Item::CountryType);
        vd.field_list_items("valid_subject_country_types", Item::CountryType);
        vd.field_list_items("valid_overlord_ranks", Item::CountryRank);
        vd.field_list_items("valid_subject_ranks", Item::CountryRank);
    }
}
