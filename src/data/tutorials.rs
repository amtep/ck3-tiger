use crate::block::Block;
use crate::context::ScopeContext;
use crate::datatype::Datatype;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::gui::validate_datatype_field;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct TutorialLesson {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3.union(GameFlags::Vic3), Item::TutorialLesson, TutorialLesson::add)
}

impl TutorialLesson {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TutorialLesson, key, block, Box::new(Self {}));
    }
}

impl DbKind for TutorialLesson {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        let chain = block.get_field_value("chain");
        for (key, block) in block.iter_definitions() {
            if !key.is("trigger") && !key.is("trigger_transition") {
                db.add(
                    Item::TutorialLessonStep,
                    key.clone(),
                    block.clone(),
                    Box::new(TutorialLessonStep { chain: chain.cloned() }),
                );
            }
        }
    }

    #[allow(unused_variables)] // for `key` when not vic3
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("chain", Item::TutorialLessonChain);
        vd.field_bool("start_automatically");

        vd.field_validated_key_block("trigger", |key, block, data| {
            let mut sc = ScopeContext::new(game_tutorial_scope(), key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        // TODO: register these as item Flags and check them for
        // the IsTutorialTagOpen gui function?
        // Downside: some mods may check other mods' tags for compatibility.
        vd.multi_field_value("gui_tag");
        // TODO: check that this is a widget name
        vd.field_value("highlight_widget");
        // TODO: verify this works in CK3 too
        vd.field_validated_key("highlight_widget_dynamic_loc", |key, bv, data| {
            validate_datatype_field(Datatype::Unknown, key, bv, data, false);
        });
        #[cfg(feature = "vic3")]
        {
            let mut sc = ScopeContext::new(Scopes::JournalEntry, key);
            vd.multi_field_target("highlight_target", &mut sc, Scopes::all());
        }

        vd.multi_field_validated_block("trigger_transition", validate_trigger_transition);

        vd.field_integer("delay");
        vd.field_integer("default_lesson_step_delay");

        vd.field_bool("finish_gamestate_tutorial");
        vd.field_bool("shown_in_encyclopedia");
        vd.field_item("encyclopedia_text", Item::Localization);

        // The tutorial lesson steps are validated in `TutorialLessonStep`
        vd.unknown_block_fields(|_, _| ());
    }
}

#[derive(Clone, Debug)]
pub struct TutorialLessonChain {
    gamestate_tutorial: bool,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3.union(GameFlags::Vic3), Item::TutorialLessonChain, TutorialLessonChain::add)
}

impl TutorialLessonChain {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        let gamestate_tutorial =
            block.get_field_bool("save_progress_in_gamestate").unwrap_or(false);
        db.add(Item::TutorialLessonChain, key, block, Box::new(Self { gamestate_tutorial }));
    }
}

impl DbKind for TutorialLessonChain {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_key_block("trigger", |key, block, data| {
            // TODO: verify root scope
            let mut sc = ScopeContext::new(game_tutorial_scope(), key);
            validate_trigger(block, data, &mut sc, Tooltipped::No);
        });

        vd.field_integer("delay");
        vd.field_bool("save_progress_in_gamestate");
    }

    fn has_property(
        &self,
        _key: &Token,
        _block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        property == "gamestate_tutorial" && self.gamestate_tutorial
    }
}

/// These are added by the [`TutorialLesson`] item loader
#[derive(Clone, Debug)]
pub struct TutorialLessonStep {
    chain: Option<Token>,
}

impl DbKind for TutorialLessonStep {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("text", Item::Localization);
        vd.field_item("header_info", Item::Localization); // undocumented

        // see gui_tag in TutorialLesson
        vd.multi_field_value("gui_tag");
        vd.field_value("window_name");
        // TODO: check that this is a widget name
        vd.multi_field_value("highlight_widget");
        // TODO: verify this works in CK3 too
        vd.field_validated_key("highlight_widget_dynamic_loc", |key, bv, data| {
            validate_datatype_field(Datatype::Unknown, key, bv, data, false);
        });
        #[cfg(feature = "vic3")]
        {
            let mut sc = ScopeContext::new(Scopes::JournalEntry, key);
            vd.multi_field_target("highlight_target", &mut sc, Scopes::all());
        }

        // TODO: These two are not used in vanilla and the docs are a bit unclear
        vd.field_item("soundeffect", Item::Sound);
        vd.field_item("voice", Item::Sound);

        vd.field_bool("repeat_sound_effect");
        vd.field_integer("delay");
        vd.field_value("animation");
        vd.field_bool("shown_in_encyclopedia");
        vd.field_item("encyclopedia_text", Item::Localization);

        vd.multi_field_validated_block("gui_transition", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("button_id");
            vd.field_item("button_text", Item::Localization);
            vd.field_validated_value("target", validate_lesson_target);
            vd.field_validated_key_block("enabled", |key, block, data| {
                let mut sc = ScopeContext::new(game_tutorial_scope(), key);
                validate_trigger(block, data, &mut sc, Tooltipped::No);
            });
        });
        vd.multi_field_validated_block("trigger_transition", validate_trigger_transition);

        // TODO: verify this works in Vic3 too
        vd.field_validated_key_block("interface_effect", |key, block, data| {
            // TODO: need a general way to restrict effects to interface effects only
            let mut sc = ScopeContext::new(Scopes::None, key);
            validate_effect(block, data, &mut sc, Tooltipped::No);
        });

        if self.chain.as_ref().map_or(false, |t| {
            data.item_has_property(Item::TutorialLessonChain, t.as_str(), "gamestate_tutorial")
        }) {
            vd.field_bool("pause_game");
            vd.field_bool("force_pause_game");
            vd.field_validated_key_block("effect", |key, block, data| {
                let mut sc = ScopeContext::new(game_tutorial_scope(), key);
                validate_effect(block, data, &mut sc, Tooltipped::No);
            });
        } else {
            vd.ban_field("pause_game", || "gamestate tutorial chains");
            vd.ban_field("force_pause_game", || "gamestate tutorial chains");
            vd.ban_field("effect", || "gamestate tutorial chains");
        }
    }
}

fn validate_trigger_transition(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_validated_key_block("trigger", |key, block, data| {
        let mut sc = ScopeContext::new(game_tutorial_scope(), key);
        validate_trigger(block, data, &mut sc, Tooltipped::No);
    });

    vd.field_validated_value("target", validate_lesson_target);
    vd.field_value("button_id");
    vd.field_item("button_text", Item::Localization);
}

fn validate_lesson_target(_key: &Token, mut vd: ValueValidator) {
    vd.maybe_is("lesson_finish");
    vd.maybe_is("lesson_abort");
    vd.item(Item::TutorialLessonStep);
}

fn game_tutorial_scope() -> Scopes {
    match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => Scopes::Character,
        #[cfg(feature = "vic3")]
        Game::Vic3 => Scopes::Country,
    }
}
