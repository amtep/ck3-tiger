use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{ModifKinds, validate_modifs};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PowerBlocCoaPiece {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PowerBlocCoaPiece, PowerBlocCoaPiece::add)
}

impl PowerBlocCoaPiece {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PowerBlocCoaPiece, key, block, Box::new(Self {}));
    }
}

impl DbKind for PowerBlocCoaPiece {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let pathname = format!("gfx/coat_of_arms/colored_emblems/{key}");
        data.verify_exists_implied(Item::File, &pathname, key);

        let mut vd = Validator::new(block, data);

        vd.field_integer("colors");
        vd.req_field("piece");
        vd.field_choice("piece", &["shield_pattern", "shield_frame", "center", "top", "side"]);
    }
}

#[derive(Clone, Debug)]
pub struct PowerBlocMapTexture {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PowerBlocMapTexture, PowerBlocMapTexture::add)
}

impl PowerBlocMapTexture {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PowerBlocMapTexture, key, block, Box::new(Self {}));
    }
}

impl DbKind for PowerBlocMapTexture {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        // undocumented item type

        let mut vd = Validator::new(block, data);

        vd.req_field("texture");
        vd.field_item("texture", Item::File);
    }
}

#[derive(Clone, Debug)]
pub struct PowerBlocIdentity {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PowerBlocIdentity, PowerBlocIdentity::add)
}

impl PowerBlocIdentity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PowerBlocIdentity, key, block, Box::new(Self {}));
    }
}

impl DbKind for PowerBlocIdentity {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.field_validated_block("visible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("possible", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("can_join", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });

        vd.field_item("icon", Item::File);
        vd.field_item("background", Item::File);

        vd.field_validated_block("power_bloc_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::all(), vd);
        });
        vd.advice_field(
            "participant_modifier",
            "docs say participant_modifier but it's member_modifier",
        );
        for field in &["member_modifier", "leader_modifier", "non_leader_modifier"] {
            vd.field_validated_block(field, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::all(), vd);
            });
        }

        vd.field_script_value_full("mandate_progress", Scopes::PowerBloc, true);

        let mut sc_cohesion = ScopeContext::new(Scopes::Country, key);
        sc_cohesion.define_name("power_bloc", Scopes::PowerBloc, key);
        sc_cohesion.define_name("with_country", Scopes::Country, key);
        sc_cohesion.define_name("without_country", Scopes::Country, key);
        vd.field_script_value_full("cohesion", &mut sc_cohesion, true);

        // undocumented
        vd.field_validated_block("can_leave", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("on_created", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_script_value_full("ai_weight", Scopes::Country, false);
    }
}

#[derive(Clone, Debug)]
pub struct PowerBlocName {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PowerBlocName, PowerBlocName::add)
}

impl PowerBlocName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PowerBlocName, key, block, Box::new(Self {}));
    }
}

impl DbKind for PowerBlocName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.req_field("trigger");
        vd.field_validated_block("trigger", |block, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("selected_identity", Scopes::PowerBlocIdentity, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
    }
}
