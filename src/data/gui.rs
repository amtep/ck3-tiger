//! Validate files in `gui/`

use std::mem::drop;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use fnv::FnvHashMap;

use crate::block::{Block, BlockItem, Field, BV};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::gui::{BuiltinWidget, GuiBlock, GuiBlockFrom};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::pdxfile::PdxFile;
use crate::report::{err, fatal, untidy, warn, ErrorKey, Severity};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Debug, Default)]
pub struct Gui {
    files: FnvHashMap<PathBuf, Vec<GuiWidget>>,
    templates: FnvHashMap<String, GuiTemplate>,
    // Type keys are stored in lowercase because type lookup is case-insensitive
    types: FnvHashMap<String, GuiType>,
    layers: FnvHashMap<String, GuiLayer>,
    // TextIcon is in a Vec because a single icon can have multiple definitions with different
    // iconsize parameters.
    texticons: FnvHashMap<String, Vec<TextIcon>>,
    textformats: FnvHashMap<String, TextFormat>,
    // This is indexed by a (colorblindmode, textformatname) pair
    textformats_colorblind: FnvHashMap<(String, String), TextFormat>,
    widget_names: FnvHashMap<String, Token>,
}

impl Gui {
    #[allow(clippy::collapsible_else_if)] // clippy is suggesting bad style here
    fn load_widget(&mut self, filename: PathBuf, key: Token, mut block: Block) {
        if key.is("texticon") {
            if let Some(icon) = block.get_field_value("icon") {
                self.load_texticon(icon.clone(), block);
            } else {
                warn(ErrorKey::FieldMissing)
                    .strong()
                    .msg("texticon without icon field")
                    .loc(block)
                    .push();
            }
        } else if key.is("textformatting") {
            let colorblindmode = block.get_field_value("color_blind_mode").cloned();
            for item in block.drain() {
                if let BlockItem::Field(Field(key, _, BV::Value(_))) = item {
                    if key.is("color_blind_mode") {
                        continue;
                    }
                } else if let BlockItem::Field(Field(key, _, BV::Block(block))) = item {
                    if key.is("format") {
                        if let Some(token) = block.get_field_value("name") {
                            self.load_textformat(token.clone(), block, colorblindmode.clone());
                        } else {
                            warn(ErrorKey::FieldMissing)
                                .strong()
                                .msg("format without name field")
                                .loc(block)
                                .push();
                        }
                    } else {
                        let msg = "unknown key in textformatting";
                        untidy(ErrorKey::UnknownField).msg(msg).loc(key).push();
                    }
                } else {
                    let msg = format!("unknown {} in textformatting", item.describe());
                    untidy(ErrorKey::Validation).msg(msg).loc(item).push();
                }
            }
        } else {
            if let Some(name) = block.get_field_value("name") {
                self.widget_names.insert(name.to_string(), name.clone());
            }
            if let Some(guifile) = self.files.get_mut(&filename) {
                guifile.push(GuiWidget::new(key, block));
            } else {
                self.files.insert(filename, vec![GuiWidget::new(key, block)]);
            }
        }
    }

    fn load_types(&mut self, block: &Block) {
        #[derive(Copy, Clone)]
        enum Expecting<'a> {
            Type,
            Header,
            Body(&'a Token, &'a Token),
        }

        let mut stage = Expecting::Type;
        for item in block.iter_items() {
            match stage {
                Expecting::Type => {
                    if let Some(field) = item.get_field() {
                        let msg = format!("unexpected {}", field.describe());
                        if field.is_eq() {
                            let info = "did you forget the `type` keyword?";
                            warn(ErrorKey::ParseError).msg(msg).info(info).loc(field).push();
                        } else {
                            warn(ErrorKey::ParseError).msg(msg).loc(field).push();
                        }
                    } else if let Some(token) = item.expect_value() {
                        if token.is("type") || token.is("local_type") {
                            stage = Expecting::Header;
                        } else {
                            let msg = format!("unexpected token `{token}`");
                            err(ErrorKey::ParseError).msg(msg).loc(token).push();
                        }
                    }
                }
                Expecting::Header => {
                    if let Some((key, token)) = item.get_assignment() {
                        stage = Expecting::Body(key, token);
                    } else {
                        err(ErrorKey::ParseError).msg("expected type header").loc(item).push();
                        stage = Expecting::Type;
                    }
                }
                Expecting::Body(name, base) => {
                    if let Some(block) = item.expect_block() {
                        self.load_type(name.clone(), base.clone(), block.clone());
                    }
                    stage = Expecting::Type;
                }
            }
        }
    }

    pub fn load_type(&mut self, key: Token, base: Token, block: Block) {
        let key_lc = key.as_str().to_lowercase();

        if let Some(other) = self.types.get(&key_lc) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "gui type");
            }
        }
        self.types.insert(key_lc, GuiType::new(key, base, block));
    }

    pub fn load_template(&mut self, key: Token, block: Block) {
        if let Some(other) = self.templates.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "gui template");
            }
        }
        self.templates.insert(key.to_string(), GuiTemplate::new(key, block));
    }

    pub fn load_layer(&mut self, key: Token, block: Block) {
        if let Some(other) = self.layers.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "gui layer");
            }
        }
        self.layers.insert(key.to_string(), GuiLayer::new(key, block));
    }

    pub fn load_texticon(&mut self, key: Token, block: Block) {
        // TODO: warn about exact duplicates? where the iconsize is the same
        if let Some(vec) = self.texticons.get_mut(key.as_str()) {
            vec.push(TextIcon::new(key, block));
        } else {
            self.texticons.insert(key.to_string(), vec![TextIcon::new(key, block)]);
        }
    }

    pub fn load_textformat(&mut self, key: Token, block: Block, color_blind_mode: Option<Token>) {
        if let Some(cbm) = color_blind_mode {
            let index = (cbm.to_string(), key.to_string());
            if let Some(other) = self.textformats_colorblind.get(&index) {
                if other.key.loc.kind >= key.loc.kind {
                    let id = format!("textformat for {cbm}");
                    dup_error(&key, &other.key, &id);
                }
            }
            self.textformats_colorblind.insert(index, TextFormat::new(key, block, Some(cbm)));
        } else {
            if let Some(other) = self.textformats.get(key.as_str()) {
                if other.key.loc.kind >= key.loc.kind {
                    dup_error(&key, &other.key, "textformat");
                }
            }
            self.textformats.insert(key.to_string(), TextFormat::new(key, block, None));
        }
    }

    pub fn template_exists(&self, key: &str) -> bool {
        self.templates.contains_key(key)
    }

    pub fn iter_template_keys(&self) -> impl Iterator<Item = &Token> {
        self.templates.values().map(|item| &item.key)
    }

    pub fn type_exists(&self, key: &Lowercase) -> bool {
        self.types.contains_key(key.as_str()) || BuiltinWidget::builtin_current_game(key).is_some()
    }

    pub fn iter_type_keys(&self) -> impl Iterator<Item = &Token> {
        self.types.values().map(|item| &item.key)
    }

    pub fn layer_exists(&self, key: &str) -> bool {
        self.layers.contains_key(key)
    }

    pub fn iter_layer_keys(&self) -> impl Iterator<Item = &Token> {
        self.layers.values().map(|item| &item.key)
    }

    pub fn texticon_exists(&self, key: &str) -> bool {
        self.texticons.contains_key(key)
    }

    pub fn iter_texticon_keys(&self) -> impl Iterator<Item = &Token> {
        self.texticons.values().flat_map(|v| v.iter().map(|item| &item.key))
    }

    pub fn textformat_exists(&self, key: &str) -> bool {
        self.textformats.contains_key(key)
    }

    pub fn iter_textformat_keys(&self) -> impl Iterator<Item = &Token> {
        self.textformats.values().map(|item| &item.key)
    }

    pub fn name_exists(&self, key: &str) -> bool {
        self.widget_names.contains_key(key)
    }

    pub fn iter_names(&self) -> impl Iterator<Item = &Token> {
        self.widget_names.values()
    }

    pub fn validate(&self, data: &Everything) {
        for items in self.files.values() {
            for item in items {
                item.validate(data);
            }
        }
        for item in self.templates.values() {
            item.validate(data);
        }
        for item in self.types.values() {
            item.validate(data);
        }
        for item in self.layers.values() {
            item.validate(data);
        }
        for vec in self.texticons.values() {
            for item in vec {
                item.validate(data);
            }
        }
        for item in self.textformats.values() {
            item.validate(data);
        }
        for item in self.textformats_colorblind.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Gui {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("gui")
    }

    fn load_file(&self, entry: &FileEntry) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".gui") {
            return None;
        }

        PdxFile::read_optional_bom(entry)
    }

    fn handle_file(&mut self, entry: &FileEntry, mut block: Block) {
        #[derive(Clone, Debug)]
        enum Expecting {
            Widget,
            Types,
            TypesBody,
            Template,
            TemplateBody(Token),
            Layer,
            LayerBody(Token),
        }

        let mut expecting = Expecting::Widget;

        for item in block.drain() {
            match expecting {
                Expecting::Widget => {
                    if let BlockItem::Field(field) = item {
                        if let Some((key, token)) = field.get_assignment() {
                            if key.lowercase_is("template") || key.lowercase_is("local_template") {
                                expecting = Expecting::TemplateBody(token.clone());
                            } else {
                                err(ErrorKey::ParseError)
                                    .msg("unexpected assignment")
                                    .loc(key)
                                    .push();
                            }
                        } else if let Some((key, block)) = field.expect_into_definition() {
                            self.load_widget(PathBuf::from(entry.filename()), key, block);
                        }
                    } else if let Some(token) = item.expect_value() {
                        // TODO: figure out how local the local_template is
                        if token.lowercase_is("template") || token.lowercase_is("local_template") {
                            expecting = Expecting::Template;
                        } else if token.lowercase_is("types") {
                            expecting = Expecting::Types;
                        } else if token.lowercase_is("layer") {
                            expecting = Expecting::Layer;
                        } else {
                            let msg = format!("unexpected value `{token}`");
                            err(ErrorKey::ParseError).msg(msg).loc(token).push();
                        }
                    }
                }
                Expecting::Types => {
                    if let BlockItem::Field(field) = item {
                        let msg = format!("unexpected {}", field.describe());
                        let info = format!(
                            "After `Types {}` there shouldn't be an `{}`",
                            field.key(),
                            field.cmp()
                        );
                        err(ErrorKey::ParseError).msg(msg).info(info).loc(field).push();
                        expecting = Expecting::Widget;
                    } else if item.expect_value().is_some() {
                        expecting = Expecting::TypesBody;
                    } else {
                        expecting = Expecting::Widget;
                    }
                }
                Expecting::TypesBody => {
                    if let Some(block) = item.expect_block() {
                        self.load_types(block);
                    }
                    expecting = Expecting::Widget;
                }
                Expecting::Template => {
                    if let BlockItem::Field(field) = item {
                        let msg = format!("unexpected {}", field.describe());
                        let info = format!(
                            "After `template {}` there shouldn't be an `{}`",
                            field.key(),
                            field.cmp()
                        );
                        err(ErrorKey::ParseError).msg(msg).info(info).loc(field).push();
                        expecting = Expecting::Widget;
                    } else if let Some(token) = item.expect_into_value() {
                        expecting = Expecting::TemplateBody(token);
                    } else {
                        expecting = Expecting::Widget;
                    }
                }
                Expecting::TemplateBody(token) => {
                    if let Some(block) = item.expect_into_block() {
                        self.load_template(token.clone(), block);
                    }
                    expecting = Expecting::Widget;
                }
                Expecting::Layer => {
                    if let BlockItem::Field(field) = item {
                        let msg = format!("unexpected {}", field.describe());
                        let info = format!(
                            "After `layer {}` there shouldn't be an `{}`",
                            field.key(),
                            field.cmp()
                        );
                        err(ErrorKey::ParseError).msg(msg).info(info).loc(field).push();
                        expecting = Expecting::Widget;
                    } else if let Some(token) = item.expect_into_value() {
                        expecting = Expecting::LayerBody(token);
                    } else {
                        expecting = Expecting::Widget;
                    }
                }
                Expecting::LayerBody(token) => {
                    if let Some(block) = item.expect_into_block() {
                        self.load_layer(token.clone(), block);
                    }
                    expecting = Expecting::Widget;
                }
            }
        }
    }

    fn finalize(&mut self) {
        for item in self.types.values() {
            _ = item.builtin(&self.types);
            _ = item.gui_block(&self.types, &self.templates);
        }
        for item in self.templates.values() {
            _ = item.gui_block(&self.types, &self.templates);
        }
    }
}

#[derive(Clone, Debug)]
struct GuiWidget {
    key: Token,
    block: Block,
}

impl GuiWidget {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        data.verify_exists(Item::GuiType, &self.key);
        let guiblock = GuiBlock::from_block(
            GuiBlockFrom::WidgetKey(&self.key),
            &self.block,
            &data.gui.types,
            &data.gui.templates,
        );
        guiblock.validate(None, data);
    }
}

#[derive(Clone, Debug)]
struct TextIcon {
    #[allow(dead_code)]
    key: Token,
    block: Block,
}

impl TextIcon {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.set_max_severity(Severity::Warning);
        vd.field_value("icon");
        vd.field_validated_block("iconsize", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.set_max_severity(Severity::Warning);
            vd.field_item("texture", Item::File);
            vd.field_list_integers_exactly("size", 2);
            vd.field_list_integers_exactly("offset", 2);
            vd.field_integer("fontsize");
            vd.field_list_numeric_exactly("uv", 4);
        });
    }
}

#[derive(Clone, Debug)]
struct TextFormat {
    #[allow(dead_code)]
    key: Token,
    block: Block,
    color_blind_mode: Option<Token>,
}

impl TextFormat {
    pub fn new(key: Token, block: Block, color_blind_mode: Option<Token>) -> Self {
        Self { key, block, color_blind_mode }
    }

    pub fn validate(&self, data: &Everything) {
        if self.color_blind_mode.is_some() {
            // Color-blind modes must override existing textformats
            data.verify_exists(Item::TextFormat, &self.key);
        }
        let mut vd = Validator::new(&self.block, data);
        vd.set_max_severity(Severity::Warning);
        vd.field_value("name");
        vd.field_bool("override");
        vd.field_value("format"); // TODO
    }
}

#[derive(Debug)]
pub struct GuiTemplate {
    #[allow(dead_code)] // key is not used, but don't want to remove the code for storing it
    key: Token,
    block: Block,
    gui_block: RwLock<Option<Arc<GuiBlock>>>,
}

impl GuiTemplate {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block, gui_block: RwLock::new(None) }
    }

    pub fn validate(&self, data: &Everything) {
        // unwrapping the Option is safe because they were all calculated during finalize
        self.gui_block.read().unwrap().as_ref().unwrap().validate(None, data);
    }

    pub fn calculate_gui_block(
        &self,
        types: &FnvHashMap<String, GuiType>,
        templates: &FnvHashMap<String, GuiTemplate>,
    ) -> Arc<GuiBlock> {
        if let Ok(mut gui_block) = self.gui_block.try_write() {
            let calc = GuiBlock::from_block(GuiBlockFrom::Template, &self.block, types, templates);
            *gui_block = Some(calc.clone());
            calc
        } else {
            let msg = "cycle in type block definitions";
            fatal(ErrorKey::Loop).msg(msg).loc(&self.key).push();
            Arc::new(GuiBlock::default())
        }
    }

    pub fn gui_block(
        &self,
        types: &FnvHashMap<String, GuiType>,
        templates: &FnvHashMap<String, GuiTemplate>,
    ) -> Arc<GuiBlock> {
        if let Ok(gui_block) = self.gui_block.try_read() {
            if let Some(gui_block) = gui_block.clone() {
                // cloning the Option Arc
                gui_block
            } else {
                drop(gui_block);
                self.calculate_gui_block(types, templates)
            }
        } else {
            let msg = "cycle in type block definitions";
            fatal(ErrorKey::Loop).msg(msg).loc(&self.key).push();
            Arc::new(GuiBlock::default())
        }
    }
}

#[derive(Debug)]
pub struct GuiType {
    key: Token,
    base: Token,
    block: Block,
    // Is a scrollbar = scrollbar type definition, with key the same as base and base should be a
    // builtin type.
    is_builtin_wrapper: bool,
    // The outer Option is whether the result has been calculated;
    // the inner Option is that the result might not exist.
    #[allow(clippy::option_option)] // TODO
    builtin: RwLock<Option<Option<BuiltinWidget>>>,
    gui_block: RwLock<Option<Arc<GuiBlock>>>,
}

impl GuiType {
    pub fn new(key: Token, base: Token, block: Block) -> Self {
        let base_lc = Lowercase::new(base.as_str());
        // Precalculate the builtin field to give the recursive builtin calculation somewhere to terminate.
        let builtin = BuiltinWidget::builtin_current_game(&base_lc).map(Some);
        let is_builtin_wrapper = base_lc == Lowercase::new(key.as_str());
        Self {
            key,
            base,
            block,
            is_builtin_wrapper,
            builtin: RwLock::new(builtin),
            gui_block: RwLock::new(None),
        }
    }

    pub fn validate(&self, data: &Everything) {
        data.verify_exists(Item::GuiType, &self.base);
        let base_lc = Lowercase::new(self.base.as_str());
        if self.is_builtin_wrapper && BuiltinWidget::builtin_current_game(&base_lc).is_none() {
            err(ErrorKey::Loop)
                .msg("recursive definition of non-builtin type")
                .loc(&self.key)
                .push();
        }
        // Unwrapping the Option is safe because they were all calculated during finalize
        self.gui_block.read().unwrap().as_ref().unwrap().validate(None, data);
    }

    pub fn calculate_builtin(&self, types: &FnvHashMap<String, GuiType>) -> Option<BuiltinWidget> {
        if let Ok(mut builtin) = self.builtin.try_write() {
            let base_lc = self.base.as_str().to_lowercase();
            let calc = types.get(&base_lc).and_then(|t| t.builtin(types));
            *builtin = Some(calc);
            calc
        } else {
            let msg = "cycle in type definitions";
            fatal(ErrorKey::Loop).msg(msg).loc(&self.key).push();
            None
        }
    }

    pub fn builtin(&self, types: &FnvHashMap<String, GuiType>) -> Option<BuiltinWidget> {
        if let Ok(builtin) = self.builtin.try_read() {
            if let Some(builtin) = *builtin {
                builtin
            } else {
                drop(builtin);
                self.calculate_builtin(types)
            }
        } else {
            let msg = "cycle in type definitions";
            fatal(ErrorKey::Loop).msg(msg).loc(&self.key).push();
            None
        }
    }

    pub fn calculate_gui_block(
        &self,
        types: &FnvHashMap<String, GuiType>,
        templates: &FnvHashMap<String, GuiTemplate>,
    ) -> Arc<GuiBlock> {
        if let Ok(mut gui_block) = self.gui_block.try_write() {
            let from = if self.is_builtin_wrapper {
                GuiBlockFrom::TypeWrapper(&self.base)
            } else {
                GuiBlockFrom::TypeBase(&self.base)
            };
            let calc = GuiBlock::from_block(from, &self.block, types, templates);
            *gui_block = Some(calc.clone());
            calc
        } else {
            let msg = "cycle in type block definitions";
            fatal(ErrorKey::Loop).msg(msg).loc(&self.key).push();
            Arc::new(GuiBlock::default())
        }
    }

    pub fn gui_block(
        &self,
        types: &FnvHashMap<String, GuiType>,
        templates: &FnvHashMap<String, GuiTemplate>,
    ) -> Arc<GuiBlock> {
        if let Ok(gui_block) = self.gui_block.try_read() {
            // cloning the Option Arc
            if let Some(gui_block) = gui_block.clone() {
                gui_block
            } else {
                drop(gui_block);
                self.calculate_gui_block(types, templates)
            }
        } else {
            let msg = "cycle in type block definitions";
            fatal(ErrorKey::Loop).msg(msg).loc(&self.key).push();
            Arc::new(GuiBlock::default())
        }
    }
}

#[derive(Clone, Debug)]
struct GuiLayer {
    key: Token,
    block: Block,
}

impl GuiLayer {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.field_value("priority");
    }
}
