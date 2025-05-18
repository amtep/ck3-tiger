use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::stringify_choices;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;
use crate::vic3::tables::misc::STANCES;

#[derive(Clone, Debug)]
pub struct Ideology {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Ideology, Ideology::add)
}

impl Ideology {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Ideology, key, block, Box::new(Self {}));
    }
}

impl DbKind for Ideology {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_bool("show_in_list");

        vd.field_bool("character_ideology");
        if block.field_value_is("character_ideology", "yes") {
            vd.field_trigger_rooted("country_trigger", Tooltipped::No, Scopes::Country);
            vd.field_trigger_rooted(
                "interest_group_leader_trigger",
                Tooltipped::No,
                Scopes::Character,
            );
            vd.field_trigger_rooted(
                "non_interest_group_leader_trigger",
                Tooltipped::No,
                Scopes::Character,
            );
            vd.field_script_value_rooted("interest_group_leader_weight", Scopes::InterestGroup);
            vd.field_script_value_rooted("non_interest_group_leader_weight", Scopes::Character);
            vd.ban_field("priority", || "character_ideology = no");
        } else {
            vd.ban_field("country_trigger", || "character_ideology = yes");
            vd.ban_field("interest_group_leader_trigger", || "character_ideology = yes");
            vd.ban_field("non_interest_group_leader_trigger", || "character_ideology = yes");
            vd.ban_field("interest_group_leader_weight", || "character_ideology = yes");
            vd.ban_field("non_interest_group_leader_weight", || "character_ideology = yes");
            vd.field_numeric("priority");
        }

        vd.unknown_block_fields(|key, block| {
            data.verify_exists(Item::LawGroup, key);
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::LawType, key);
                if !STANCES.contains(&value.as_str()) {
                    let msg = format!("expected one of {}", stringify_choices(STANCES));
                    warn(ErrorKey::Choice).msg(msg).loc(value).push();
                }
            });
        });
    }
}
