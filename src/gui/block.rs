use std::sync::Arc;

use fnv::{FnvHashMap, FnvHashSet};

use crate::block::{Block, BlockItem, Comparator, Eq::Single, Field, BV};
use crate::data::gui::{GuiTemplate, GuiType};
use crate::everything::Everything;
use crate::gui::validate::validate_property;
use crate::gui::{BuiltinWidget, WidgetProperty};
use crate::lowercase::Lowercase;
use crate::report::{err, untidy, warn, ErrorKey};
use crate::token::Token;

/// An element of a [`GuiBlock`]
#[derive(Debug, Clone)]
enum GuiItem {
    /// A property assignment.
    Property(WidgetProperty, Token, BV),
    /// A contained widget.
    Widget(Lowercase<'static>, Arc<GuiBlock>),
    /// A named block whose contents can be substituted. Will be inlined later.
    Subst(String, Arc<GuiBlock>),
    /// A named block whose contents will be inserted into any Subst of the same name.
    /// In most cases, Override will be skipped because it was already applied.
    /// The exception is in templates when they get inlined.
    Override(Token, Arc<GuiBlock>),
}

/// A processed version of a [`Block`] meant for `.gui` files.
#[derive(Debug, Clone, Default)]
pub struct GuiBlock {
    /// The widget's ultimate base type, if known.
    builtin: Option<BuiltinWidget>,
    /// The definition of the base type of this widget type.
    base: Option<Arc<GuiBlock>>,
    /// The contents of this block.
    items: Vec<GuiItem>,
    /// The names of all named blocks in this block, its base types, and its children.
    substnames: FnvHashSet<String>,
}

/// An indication of where this [`Block`] was found, to help with determining the metadata for the
/// resulting [`GuiBlock`].
#[derive(Debug, Clone, Copy)]
pub enum GuiBlockFrom<'a> {
    /// A template being evaluated standalone
    Template,
    /// No parent yet; for example inside a blockoverride
    NoParent,
    /// A widget declaration, either at the top of a file or a contained widget
    WidgetKey(&'a Token),
    /// A type declaration
    TypeBase(&'a Token),
    /// A type declaration that's a wrapper around a builtin type, like `scrollbar = scrollbar {`.
    TypeWrapper(&'a Token),
}

impl GuiBlock {
    /// Process a [`Block`] into a [`GuiBlock`].
    pub fn from_block(
        from: GuiBlockFrom,
        block: &Block,
        types: &FnvHashMap<String, GuiType>,
        templates: &FnvHashMap<String, GuiTemplate>,
    ) -> Arc<Self> {
        enum Expecting<'a> {
            Field,
            SubstBlock,
            SubstBlockBody(&'a Token),
            BlockOverride,
            BlockOverrideBody(&'a Token),
        }
        let mut state = Expecting::Field;

        // Blank slate to work on
        let mut gui = Self {
            builtin: None,
            base: None,
            items: Vec::new(),
            substnames: FnvHashSet::default(),
        };

        // Fill in `builtin` and `base` fields if the base type is known
        match from {
            GuiBlockFrom::Template | GuiBlockFrom::NoParent => (),
            GuiBlockFrom::WidgetKey(base) | GuiBlockFrom::TypeBase(base) => {
                if let Some(basetype) = types.get(&base.as_str().to_lowercase()) {
                    gui.builtin = basetype.builtin(types);
                    let gui_block = basetype.gui_block(types, templates);
                    gui.substnames = gui_block.substnames.clone();
                    gui.base = Some(gui_block);
                }
            }
            GuiBlockFrom::TypeWrapper(base) => {
                if let Some(basetype) = types.get(&base.as_str().to_lowercase()) {
                    gui.builtin = basetype.builtin(types);
                }
            }
        }

        for item in block.iter_items() {
            match state {
                Expecting::Field => {
                    if let BlockItem::Field(Field(key, cmp, bv)) = item {
                        let key_lc = Lowercase::new(key.as_str());
                        if !matches!(cmp, Comparator::Equals(Single)) {
                            let msg = format!("expected `{key} =`, found `{cmp}`");
                            untidy(ErrorKey::Validation).msg(msg).loc(key).push();
                        }
                        if key_lc == "block" {
                            if let Some(value) = bv.expect_value() {
                                state = Expecting::SubstBlockBody(value);
                            }
                        } else if key_lc == "blockoverride" {
                            if let Some(value) = bv.expect_value() {
                                state = Expecting::BlockOverrideBody(value);
                            }
                        } else if key_lc == "using" {
                            if let Some(value) = bv.expect_value() {
                                if let Some(template) = templates.get(value.as_str()) {
                                    gui.inline(&template.gui_block(types, templates));
                                } else {
                                    untidy(ErrorKey::Gui).msg("template not found").loc(key).push();
                                }
                            }
                        } else if let Ok(prop) = WidgetProperty::try_from(&key_lc) {
                            gui.items.push(GuiItem::Property(prop, key.clone(), bv.clone()));
                        } else if types.get(key_lc.as_str()).is_some()
                            || BuiltinWidget::is_builtin_current_game(&key_lc)
                        {
                            if let Some(block) = bv.expect_block() {
                                let guiblock = GuiBlock::from_block(
                                    GuiBlockFrom::WidgetKey(key),
                                    block,
                                    types,
                                    templates,
                                );
                                gui.substnames.extend(guiblock.substnames.iter().cloned());
                                gui.items.push(GuiItem::Widget(key_lc.into_owned(), guiblock));
                            }
                        } else if let Ok(builtin) = BuiltinWidget::try_from(&key_lc) {
                            // If we got here, then it must be a builtin but not for the current game
                            let msg = format!(
                                "builtin widget `{key}` is only for {}",
                                builtin.to_game_flags()
                            );
                            err(ErrorKey::WrongGame).weak().msg(msg).loc(key).push();
                        } else {
                            let msg = format!("unknown gui field `{key}`");
                            err(ErrorKey::UnknownField).weak().msg(msg).loc(key).push();
                        }
                    } else if let Some(key) = item.expect_value() {
                        let key_lc = Lowercase::new(key.as_str());
                        if key_lc == "block" {
                            state = Expecting::SubstBlock;
                        } else if key_lc == "blockoverride" {
                            state = Expecting::BlockOverride;
                        } else {
                            warn(ErrorKey::Gui).msg("unexpected value").loc(key).push();
                        }
                    }
                }
                Expecting::SubstBlock => {
                    if let Some(name) = item.expect_value() {
                        state = Expecting::SubstBlockBody(name);
                    } else {
                        state = Expecting::Field;
                    }
                }
                Expecting::BlockOverride => {
                    if let Some(name) = item.expect_value() {
                        state = Expecting::BlockOverrideBody(name);
                    } else {
                        state = Expecting::Field;
                    }
                }
                Expecting::SubstBlockBody(name) => {
                    if let Some(block) = item.expect_block() {
                        let guiblock =
                            GuiBlock::from_block(GuiBlockFrom::NoParent, block, types, templates);
                        gui.substnames.insert(name.to_string());
                        gui.substnames.extend(guiblock.substnames.iter().cloned());
                        gui.items.push(GuiItem::Subst(name.to_string(), guiblock));
                    }
                    state = Expecting::Field;
                }
                Expecting::BlockOverrideBody(name) => {
                    if let Some(block) = item.expect_block() {
                        if gui.substnames.contains(name.as_str()) {
                            let guiblock = GuiBlock::from_block(
                                GuiBlockFrom::NoParent,
                                block,
                                types,
                                templates,
                            );
                            gui.apply_override(name, &guiblock);
                            gui.items.push(GuiItem::Override(name.clone(), guiblock));
                        } else if !matches!(from, GuiBlockFrom::Template | GuiBlockFrom::NoParent) {
                            // TODO: can't do this until we search Widget fields for overrides as well.
                            // let msg = format!("did not find block for blockoverride `{name}`");
                            // err(ErrorKey::Gui).msg(msg).loc(name).push();
                        }
                    }
                    state = Expecting::Field;
                }
            }
        }
        Arc::new(gui)
    }

    pub fn inline(&mut self, other: &Arc<GuiBlock>) {
        self.substnames.extend(other.substnames.iter().cloned());
        for item in &other.items {
            if let GuiItem::Override(name, gui_block) = item {
                self.apply_override(name, gui_block);
            }
            self.items.push(item.clone());
        }
    }

    // TODO: return an indication of whether the blockoverride found a block
    pub fn apply_override(&mut self, name: &Token, overrideblock: &Arc<GuiBlock>) {
        if !self.substnames.contains(name.as_str()) {
            return;
        }

        self.substnames.extend(overrideblock.substnames.iter().cloned());

        if let Some(mut base) = self.base.clone() {
            self.base = Some(Self::apply_override_arc(&mut base, name, overrideblock));
        }

        for item in &mut self.items {
            match item {
                GuiItem::Property(_, _, _) | GuiItem::Override(_, _) => (),
                GuiItem::Widget(_, gui) => {
                    *gui = Self::apply_override_arc(gui, name, overrideblock);
                }
                GuiItem::Subst(substname, gui) => {
                    if name.is(substname) {
                        *gui = overrideblock.clone(); // cloning the Arc
                    }
                }
            }
        }
    }

    // TODO: this could maybe be made more efficient by checking substnames before the call,
    // and not cloning the Arc at all if that check doesn't pass.
    pub fn apply_override_arc(
        gui: &mut Arc<GuiBlock>,
        name: &Token,
        overrideblock: &Arc<GuiBlock>,
    ) -> Arc<GuiBlock> {
        if !gui.substnames.contains(name.as_str()) {
            return gui.clone(); // cloning the Arc
        }

        let gui_mut = Arc::make_mut(gui); // clones the inner GuiBlock if needed
        gui_mut.apply_override(name, overrideblock);
        gui.clone() // cloning the Arc
    }

    pub fn properties(&self) -> FnvHashMap<WidgetProperty, (&Token, &BV)> {
        let mut map = self.base.as_ref().map_or(FnvHashMap::default(), |base| base.properties());

        for item in &self.items {
            match item {
                GuiItem::Property(prop, key, bv) => {
                    map.insert(*prop, (key, bv));
                }
                GuiItem::Subst(_, block) => {
                    map.extend(block.properties());
                }
                GuiItem::Widget(_, _) | GuiItem::Override(_, _) => (),
            }
        }
        map
    }

    pub fn validate(&self, _builtin: Option<BuiltinWidget>, data: &Everything) {
        for (prop, (key, bv)) in &self.properties() {
            // TODO: check that this property can be in this builtin type
            // TODO: format_override can have multiple values
            validate_property(*prop, key, bv, data);
        }

        for item in &self.items {
            match item {
                GuiItem::Property(_, _, _) | GuiItem::Override(_, _) => (),
                GuiItem::Subst(_, gui_block) => {
                    gui_block.validate(self.builtin, data);
                }
                GuiItem::Widget(_, gui_block) => {
                    gui_block.validate(None, data);
                }
            }
        }
    }
}
