use crate::block::Block;
use crate::ck3::validate::validate_maa_stats;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Innovation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Innovation, Innovation::add)
}

impl Innovation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Innovation, key, block, Box::new(Self {}));
    }
}

impl DbKind for Innovation {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for token in block.get_field_values("flag") {
            db.add_flag(Item::InnovationFlag, token.clone());
        }
    }

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
        vd.field_trigger("potential", Tooltipped::No, &mut sc);
        vd.field_trigger("can_progress", Tooltipped::Yes, &mut sc);

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

        vd.multi_field_value("flag");

        vd.multi_field_item("unlock_building", Item::Building);
        vd.multi_field_item("unlock_decision", Item::Decision);
        vd.multi_field_item("unlock_casus_belli", Item::CasusBelli);
        vd.multi_field_item("unlock_maa", Item::MenAtArms);
        vd.multi_field_item("unlock_law", Item::Law);

        vd.multi_field_item("custom", Item::Localization);

        vd.multi_field_validated_block("maa_upgrade", |block, data| {
            let mut vd = Validator::new(block, data);
            if let Some(token) = vd.field_value("type") {
                if !data.item_exists(Item::MenAtArms, token.as_str())
                    && !data.item_exists(Item::MenAtArmsBase, token.as_str())
                {
                    let msg = format!("{token} is not a men-at-arms type or base type");
                    err(ErrorKey::MissingItem).msg(msg).loc(token).push();
                }
            }
            validate_maa_stats(&mut vd);
        });
    }
}
