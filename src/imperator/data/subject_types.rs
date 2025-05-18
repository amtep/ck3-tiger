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
pub struct SubjectType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::SubjectType, SubjectType::add)
}

impl SubjectType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SubjectType, key, block, Box::new(Self {}));
    }
}

impl DbKind for SubjectType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("future_overlord", Scopes::Country, key);
        sc.define_name("former_overlord", Scopes::Country, key);

        // Yes! There are actually that many loc keys!
        let prefixes = vec![
            "OFFER_", "CANCEL_", "BREAK_", "OFFER_", "CANCEL_", "BREAK_", "OFFER_", "CANCEL_",
            "CANCEL_", "BREAK_", "OFFER_", "CANCEL_", "BREAK_", "OFFER_", "CANCEL_", "BREAK_",
            "OFFER_", "OFFER_", "OFFER_", "OFFER_", "CANCEL_", "CANCEL_", "BREAK_", "BREAK_",
            "AM_", "LEAD_",
        ];

        let suffixes = vec![
            "{key}TITLE",
            "{key}TITLE",
            "{key}TITLE",
            "{key}_CATEGORY",
            "{key}_CATEGORY",
            "{key}_CATEGORY",
            "{key}_DESC",
            "{key}_DESC",
            "{key}_NEWDESC",
            "{key}_DESC",
            "{key}_REQDESC",
            "{key}_REQDESC",
            "{key}_REQDESC",
            "{key}_TOO_LOW",
            "{key}_NOT_IN_TRUCE",
            "{key}_NOT_IN_TRUCE",
            "{key}_ALREADY_SUBJECT",
            "{key}_AT_WAR",
            "{key}_TOOLTIP_HEADER",
            "{key}_FLAVOR",
            "{key}_TOOLTIP_HEADER",
            "{key}_FLAVOR",
            "{key}_TOOLTIP_HEADER",
            "{key}_FLAVOR",
            "{key}",
            "{key}",
        ];

        for (prefix, suffix) in prefixes.iter().zip(suffixes.iter()) {
            let loca = format!("{}{}", prefix, suffix.replace("{key}", key.as_str()));
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_item("subject_pays", Item::Price);

        vd.field_bool("joins_overlord_in_war");
        vd.field_bool("protected_when_attacked");
        vd.field_bool("has_overlords_ruler");
        vd.field_bool("can_be_integrated");
        vd.field_bool("costs_diplomatic_slot");
        vd.field_bool("subject_can_cancel");
        vd.field_bool("has_limited_diplomacy");
        vd.field_bool("only_trade_with_overlord");

        vd.field_numeric("offset");
        vd.field_numeric("scale");

        vd.field_validated_block("overlord_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("subject_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_trigger("allow", Tooltipped::No, &mut sc);
        vd.field_effect("on_enable", Tooltipped::No, &mut sc);
        vd.field_effect("on_disable", Tooltipped::No, &mut sc);
        vd.field_effect("on_monthly", Tooltipped::No, &mut sc);

        // TODO - diplo_chance block needs to be ignored here
        vd.no_warn_remaining();
    }
}
