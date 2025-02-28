use std::sync::Arc;

use crate::block::{BV, Block, BlockItem, Comparator, Eq::Single, Field};
use crate::data::gui::{GuiTemplate, GuiType};
use crate::everything::Everything;
use crate::gui::validate::validate_property;
use crate::gui::{BuiltinWidget, GuiValidation, PropertyContainer, WidgetProperty};
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::lowercase::Lowercase;
use crate::report::{ErrorKey, err, untidy, warn};
use crate::token::Token;

/// An element of a [`GuiBlock`]
#[derive(Debug, Clone)]
enum GuiItem {
    /// A property assignment.
    Property(WidgetProperty, Token, BV),
    /// A contained widget.
    Widget(Arc<GuiBlock>),
    /// A property which contains other properties. It can have `Subst` blocks too.
    ComplexProperty(Arc<GuiBlock>),
    /// A property which contains a widget. It can have Subst blocks too.
    /// Recursive widgets (ones that have `recursive = yes`) are handled as normal `Property` items instead.
    WidgetProperty(Arc<GuiBlock>),
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
    /// The widget's ultimate base type or complex property type, if known.
    /// This determines which properties are valid in this block.
    container: Option<PropertyContainer>,
    /// The definition of the base type of this widget type.
    base: Option<Arc<GuiBlock>>,
    /// The contents of this block.
    items: Vec<GuiItem>,
    /// The names of all named blocks in this block, its base types, and its children.
    substnames: TigerHashSet<String>,
}

/// An indication of where this [`Block`] was found, to help with determining the metadata for the
/// resulting [`GuiBlock`].
#[derive(Debug, Clone, Copy)]
pub enum GuiBlockFrom<'a> {
    /// A template being evaluated standalone.
    Template,
    /// No parent yet; for example inside a blockoverride.
    NoParent,
    /// A widget declaration, either at the top of a file or a contained widget.
    WidgetKey(&'a Token),
    /// A widget property that contains other gui elements.
    PropertyKey(WidgetProperty),
    /// A type declaration.
    TypeBase(&'a Token),
    /// A type declaration that's a wrapper around a builtin type of the same name, like `scrollbar = scrollbar {`.
    TypeWrapper(&'a Token),
}

impl GuiBlock {
    /// Process a [`Block`] into a [`GuiBlock`].
    pub fn from_block(
        from: GuiBlockFrom,
        block: &Block,
        types: &TigerHashMap<Lowercase<'static>, GuiType>,
        templates: &TigerHashMap<&'static str, GuiTemplate>,
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
            container: None,
            base: None,
            items: Vec::new(),
            substnames: TigerHashSet::default(),
        };

        // Fill in `container` and `base` fields if known
        match from {
            GuiBlockFrom::Template | GuiBlockFrom::NoParent => (),
            GuiBlockFrom::WidgetKey(base) | GuiBlockFrom::TypeBase(base) => {
                if let Some(basetype) = types.get(&Lowercase::new(base.as_str())) {
                    gui.container = basetype.builtin(types).map(PropertyContainer::from);
                    let gui_block = basetype.gui_block(types, templates);
                    gui.substnames.clone_from(&gui_block.substnames);
                    gui.base = Some(gui_block);
                }
            }
            GuiBlockFrom::PropertyKey(prop) => {
                gui.container = PropertyContainer::try_from(prop).ok();
            }
            GuiBlockFrom::TypeWrapper(base) => {
                if let Some(basetype) = types.get(&Lowercase::new(base.as_str())) {
                    gui.container = basetype.builtin(types).map(PropertyContainer::from);
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
                            let validation = GuiValidation::from_property(prop);
                            if validation == GuiValidation::ComplexProperty {
                                if let Some(block) = bv.expect_block() {
                                    let guiblock = GuiBlock::from_block(
                                        GuiBlockFrom::PropertyKey(prop),
                                        block,
                                        types,
                                        templates,
                                    );
                                    gui.items.push(GuiItem::ComplexProperty(guiblock));
                                }
                            } else if validation == GuiValidation::Widget {
                                // If the bv is a Value (should be a template name) or if it is a
                                // Block with recursive = yes, then store it as a normal Property.
                                // Otherwise store it as a WidgetProperty.
                                // TODO: tooltipwidget is always treated as recursive
                                match bv {
                                    BV::Block(block)
                                        if !block.field_value_is("recursive", "yes")
                                            && prop != WidgetProperty::tooltipwidget =>
                                    {
                                        let guiblock = GuiBlock::from_block(
                                            GuiBlockFrom::PropertyKey(prop),
                                            block,
                                            types,
                                            templates,
                                        );
                                        gui.items.push(GuiItem::WidgetProperty(guiblock));
                                    }
                                    _ => {
                                        gui.items.push(GuiItem::Property(
                                            prop,
                                            key.clone(),
                                            bv.clone(),
                                        ));
                                    }
                                }
                            } else {
                                gui.items.push(GuiItem::Property(prop, key.clone(), bv.clone()));
                            }
                        } else if types.get(key_lc.as_str()).is_some()
                            || BuiltinWidget::builtin_current_game(&key_lc).is_some()
                        {
                            if let Some(block) = bv.expect_block() {
                                let guiblock = GuiBlock::from_block(
                                    GuiBlockFrom::WidgetKey(key),
                                    block,
                                    types,
                                    templates,
                                );
                                gui.substnames.extend(guiblock.substnames.iter().cloned());
                                gui.items.push(GuiItem::Widget(guiblock));
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
                GuiItem::Widget(gui)
                | GuiItem::ComplexProperty(gui)
                | GuiItem::WidgetProperty(gui) => {
                    *gui = Self::apply_override_arc(gui, name, overrideblock);
                }
                GuiItem::Subst(substname, gui) => {
                    if name.is(substname) {
                        *gui = Arc::clone(overrideblock);
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
            return Arc::clone(gui);
        }

        let gui_mut = Arc::make_mut(gui); // clones the inner GuiBlock if needed
        gui_mut.apply_override(name, overrideblock);
        Arc::clone(gui)
    }

    /// Validate the property fields of this [`GuiBlock`] and all its contents.
    ///
    /// `container` is extra information to be used if `self.container` is `None`.
    pub fn validate(&self, container: Option<PropertyContainer>, data: &Everything) {
        let container = self.container.or(container);
        if let Some(base) = &self.base {
            base.validate(container, data);
        }

        for item in &self.items {
            match item {
                GuiItem::Property(prop, key, bv) => {
                    validate_property(*prop, container, key, bv, data);
                }
                GuiItem::Subst(_, gui_block) => {
                    gui_block.validate(container, data);
                }
                GuiItem::Widget(gui_block)
                | GuiItem::ComplexProperty(gui_block)
                | GuiItem::WidgetProperty(gui_block) => {
                    gui_block.validate(None, data);
                }
                GuiItem::Override(_, _) => (),
            }
        }
    }
}
