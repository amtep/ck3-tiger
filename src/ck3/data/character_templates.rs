use crate::block::Block;
use crate::ck3::validate::{
    validate_random_culture, validate_random_faith, validate_random_traits_list,
};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target_ok_this;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CharacterTemplate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CharacterTemplate, CharacterTemplate::add)
}

impl CharacterTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterTemplate, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterTemplate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut sc = ScopeContext::new_unrooted(Scopes::all(), key);
        sc.set_strict_scopes(false);
        let b = Block::new(key.loc.clone());
        self.validate_call(key, block, key, &b, data, &mut sc);
    }

    // TODO: lots of duplication between this and "create_character" effect
    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _from: &Token,
        from_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        // `from_block` is used to suppress warnings about targets that won't be used
        let mut vd = Validator::new(block, data);
        vd.field_item("name", Item::Localization);
        vd.field_script_value("age", sc);
        if let Some(token) = vd.field_value("gender") {
            if !token.is("male") && !token.is("female") {
                validate_target_ok_this(token, data, sc, Scopes::Character);
            }
        }
        vd.multi_field_item("trait", Item::Trait);
        vd.multi_field_validated_block_sc("random_traits_list", sc, validate_random_traits_list);
        vd.field_bool("random_traits");
        vd.field_script_value("gender_female_chance", sc);
        if from_block.has_key("culture") {
            vd.field_value("culture");
        } else {
            vd.field_target("culture", sc, Scopes::Culture);
        }
        if from_block.has_key("faith") {
            vd.field_value("faith");
        } else {
            vd.field_target("faith", sc, Scopes::Faith);
        }
        vd.multi_field_validated_block_sc("random_culture", sc, validate_random_culture);
        vd.multi_field_validated_block_sc("random_faith", sc, validate_random_faith);
        vd.field_script_value("health", sc);
        vd.field_script_value("diplomacy", sc);
        vd.field_script_value("intrigue", sc);
        vd.field_script_value("learning", sc);
        vd.field_script_value("martial", sc);
        vd.field_script_value("prowess", sc);
        vd.field_script_value("stewardship", sc);
        vd.field_target("dynasty_house", sc, Scopes::DynastyHouse);
        vd.field_choice("dynasty", &["generate", "inherit", "none"]);
        vd.field_validated_key_block("after_creation", |key, block, data| {
            sc.open_scope(Scopes::Character, key.clone());
            validate_effect(block, data, sc, Tooltipped::No);
            sc.close();
        });
    }
}
