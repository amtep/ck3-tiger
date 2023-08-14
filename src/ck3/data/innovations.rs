use crate::block::Block;
use crate::ck3::validate::validate_maa_stats;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{error, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Innovation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Innovation, Innovation::add)
}

impl Innovation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::InnovationFlag, token.clone());
        }
        db.add(Item::Innovation, key, block, Box::new(Self {}));
    }
}

impl DbKind for Innovation {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Culture, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("culture_era", Item::CultureEra);
        vd.field_choice(
            "group",
            &["culture_group_military", "culture_group_civic", "culture_group_regional"],
        );
        vd.field_item("icon", Item::File);

        vd.field_item("region", Item::Region);
        vd.field_validated_block("potential", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("can_progress", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        // TODO: everything after this duplicates CultureEra validation,
        // except the `type` field in `maa_upgrade`
        vd.field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("culture_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Culture, vd);
        });
        vd.field_validated_block("county_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::County, vd);
        });
        vd.field_validated_block("province_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });

        vd.field_values("flag");

        vd.field_items("unlock_building", Item::Building);
        vd.field_items("unlock_decision", Item::Decision);
        vd.field_items("unlock_casus_belli", Item::CasusBelli);
        vd.field_items("unlock_maa", Item::MenAtArms);
        vd.field_items("unlock_law", Item::Law);

        vd.field_items("custom", Item::Localization);

        vd.field_validated_blocks("maa_upgrade", |block, data| {
            let mut vd = Validator::new(block, data);
            if let Some(token) = vd.field_value("type") {
                if !data.item_exists(Item::MenAtArms, token.as_str())
                    && !data.item_exists(Item::MenAtArmsBase, token.as_str())
                {
                    let msg = format!("{token} is not a men-at-arms type or base type");
                    error(token, ErrorKey::MissingItem, &msg);
                }
            }
            validate_maa_stats(&mut vd);
        });
    }
}
