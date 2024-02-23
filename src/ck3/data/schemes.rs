use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_duration, validate_modifiers_with_base};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Scheme {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Scheme, Scheme::add)
}

impl Scheme {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Scheme, key, block, Box::new(Self {}));
    }
}

impl DbKind for Scheme {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("scheme", Scopes::Scheme, key);
        sc.define_name("target", Scopes::Character, key);
        sc.define_name("owner", Scopes::Character, key);
        sc.define_name("exposed", Scopes::Bool, key);

        // let modif = format!("max_{key}_schemes_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_power_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_power_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_resistance_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_resistance_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_action");
        data.verify_exists_implied(Item::Localization, &loca, key);
        vd.req_field("desc");
        vd.field_validated_sc("desc", &mut sc, validate_desc);
        vd.req_field("success_desc");
        vd.field_validated_sc("success_desc", &mut sc, validate_desc); // undocumented
        vd.field_validated_sc("discovery_desc", &mut sc, validate_desc); // undocumented

        vd.req_field("skill");
        vd.field_item("skill", Item::Skill);
        vd.field_numeric("power_per_skill_point");
        vd.field_numeric("resistance_per_skill_point");
        vd.field_numeric("power_per_agent_skill_point");
        vd.field_numeric("spymaster_power_per_skill_point");
        vd.field_numeric("spymaster_resistance_per_skill_point");
        vd.field_numeric("tier_resistance");

        vd.field_bool("hostile");

        let icon = vd.field_value("icon").unwrap_or(key);
        let pathname = format!("gfx/interface/icons/scheme_types/{icon}.dds");
        data.verify_exists_implied(Item::File, &pathname, icon);

        vd.field_validated_block("allow", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::Yes);
        });
        vd.field_validated_block("valid", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_integer("agent_join_threshold");
        vd.field_integer("agent_leave_threshold");
        vd.field_bool("uses_agents");
        vd.field_bool("uses_resistance");

        vd.field_validated_block("valid_agent", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_key_block("agent_join_chance", |key, block, data| {
            let mut sc = sc.clone();
            sc.define_name("gift", Scopes::Bool, key);
            validate_modifiers_with_base(block, data, &mut sc);
        });
        vd.field_validated_block_sc("agent_success_chance", &mut sc, validate_modifiers_with_base);
        vd.field_validated_key_block("base_success_chance", |key, block, data| {
            let mut sc = sc.clone();
            sc.change_root(Scopes::Scheme, key.clone());
            validate_modifiers_with_base(block, data, &mut sc);
        });

        // TODO: check that maximum >= minimum ?
        vd.field_integer_range("maximum_success", 0..=100);
        vd.field_integer_range("minimum_success", 0..=100);
        vd.field_integer_range("maximum_secrecy", 0..=100);
        vd.field_integer_range("minimum_secrecy", 0..=100);
        vd.field_integer_range("maximum_progress_chance", 0..=100);
        vd.field_integer_range("minimum_progress_chance", 0..=100);

        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_bool("is_secret");
        vd.field_validated_block("use_secrecy", |block, data| {
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_integer("base_secrecy");

        vd.field_validated_key_block("on_start", |key, block, data| {
            let mut sc = sc.clone();
            sc.change_root(Scopes::Scheme, key.clone());
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.req_field("on_ready");
        vd.field_validated_key_block("on_ready", |key, block, data| {
            let mut sc = sc.clone();
            sc.change_root(Scopes::Scheme, key.clone());
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_monthly", |key, block, data| {
            let mut sc = sc.clone();
            sc.change_root(Scopes::Scheme, key.clone());
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_invalidated", |key, block, data| {
            let mut sc = sc.clone();
            sc.change_root(Scopes::Scheme, key.clone());
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_validated_block("on_agent_join", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("on_agent_leave", |block, data| {
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_key_block("on_agent_exposed", |key, block, data| {
            let mut sc = sc.clone();
            sc.define_name("agent", Scopes::Character, key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_bool("freeze_scheme_when_traveling");
        vd.field_bool("freeze_scheme_when_traveling_target");
        vd.field_bool("cancel_scheme_when_traveling");
        vd.field_bool("cancel_scheme_when_traveling_target");
    }
}
