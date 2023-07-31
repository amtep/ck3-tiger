use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::{Block, BlockItem, Field, BV};
use crate::context::ScopeContext;
use crate::data::localization::LocaValue;
use crate::datatype::{validate_datatypes, Datatype};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
#[cfg(feature = "ck3")]
use crate::game::Game;
use crate::helpers::dup_error;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::parse::localization::ValueParser;
use crate::pdxfile::PdxFile;
use crate::report::{
    err, error, error_info, old_warn, untidy, warn, warn_info, ErrorKey, Severity,
};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
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
                            warn_info(field, ErrorKey::ParseError, &msg, info);
                        } else {
                            old_warn(field, ErrorKey::ParseError, &msg);
                        }
                    } else if let Some(token) = item.expect_value() {
                        if token.is("type") || token.is("local_type") {
                            stage = Expecting::Header;
                        } else {
                            let msg = format!("unexpected token `{token}`");
                            error(token, ErrorKey::ParseError, &msg);
                        }
                    }
                }
                Expecting::Header => {
                    if let Some((key, token)) = item.get_assignment() {
                        stage = Expecting::Body(key, token);
                    } else {
                        error(item, ErrorKey::ParseError, "expected type header");
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

    pub fn type_exists(&self, key: &Lowercase) -> bool {
        self.types.contains_key(key.as_str()) || BUILTIN_TYPES.contains(&key.as_str())
    }

    pub fn layer_exists(&self, key: &str) -> bool {
        self.layers.contains_key(key)
    }

    pub fn texticon_exists(&self, key: &str) -> bool {
        self.texticons.contains_key(key)
    }

    pub fn textformat_exists(&self, key: &str) -> bool {
        self.textformats.contains_key(key)
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

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".gui") {
            return None;
        }

        PdxFile::read_optional_bom(entry, fullpath)
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
                            error(token, ErrorKey::ParseError, &msg);
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
                        error_info(field, ErrorKey::ParseError, &msg, &info);
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
                        error_info(field, ErrorKey::ParseError, &msg, &info);
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
                        error_info(field, ErrorKey::ParseError, &msg, &info);
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
        validate_gui(&self.block, data);
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

#[derive(Clone, Debug)]
struct GuiTemplate {
    #[allow(dead_code)] // key is not used, but don't want to remove the code for storing it
    key: Token,
    block: Block,
}

impl GuiTemplate {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        validate_gui(&self.block, data);
    }
}

#[derive(Clone, Debug)]
struct GuiType {
    #[allow(dead_code)] // TODO
    key: Token,
    base: Token,
    block: Block,
}

impl GuiType {
    pub fn new(key: Token, base: Token, block: Block) -> Self {
        Self { key, base, block }
    }

    pub fn validate(&self, data: &Everything) {
        data.verify_exists(Item::GuiType, &self.base);
        validate_gui(&self.block, data);
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

// TODO: do full validation
fn validate_gui(block: &Block, data: &Everything) {
    enum Expecting {
        Field,
        SubstBlock,
        SubstBlockBody,
        BlockOverride,
        BlockOverrideBody,
    }

    let mut expecting = Expecting::Field;
    for item in block.iter_items() {
        match expecting {
            Expecting::Field => {
                if let Some(field) = item.get_field() {
                    if field.key().is("block") {
                        field.expect_assignment();
                        expecting = Expecting::SubstBlockBody;
                    } else if field.key().is("blockoverride") {
                        field.expect_assignment();
                        expecting = Expecting::BlockOverrideBody;
                    } else {
                        validate_field(field, data);
                    }
                } else if let Some(token) = item.expect_value() {
                    if token.is("block") {
                        expecting = Expecting::SubstBlock;
                    } else if token.is("blockoverride") {
                        expecting = Expecting::BlockOverride;
                    } else {
                        // allow unexpected values because we might be parsing a `size = { x y }`
                        // we can warn about them once we parse such fields selectively
                        // warn(token, ErrorKey::ParseError, "unexpected value");
                    }
                }
            }
            Expecting::SubstBlock => {
                if item.expect_value().is_some() {
                    expecting = Expecting::SubstBlockBody;
                } else {
                    expecting = Expecting::Field;
                }
            }
            Expecting::BlockOverride => {
                if item.expect_value().is_some() {
                    expecting = Expecting::BlockOverrideBody;
                } else {
                    expecting = Expecting::Field;
                }
            }
            Expecting::SubstBlockBody | Expecting::BlockOverrideBody => {
                if let Some(block) = item.expect_block() {
                    validate_gui(block, data);
                }
                expecting = Expecting::Field;
            }
        }
    }
}

fn validate_field(field: &Field, data: &Everything) {
    let key = field.key();
    if key.is("default_format") {
        field.expect_assignment();
        return;
    } else if key.is("texture") || key.is("progresstexture") || key.is("noprogresstexture") {
        if let Some((_, token)) = field.expect_assignment() {
            if !token.starts_with("[") {
                data.verify_exists(Item::File, token);
                return;
            }
        }
    } else if key.is("tooltip") || key.is("text") {
        if let Some((_, token)) = field.expect_assignment() {
            if !token.starts_with("[") {
                data.verify_exists(Item::Localization, token);
                return;
            }
        }
    } else if key.is("raw_text") {
        if let Some((_, token)) = field.expect_assignment() {
            if !token.starts_with("[") {
                // Some raw_text fields do contain loca keys
                data.mark_used(Item::Localization, token.as_str());
                return;
            }
        }
    }
    match field.bv() {
        BV::Value(token) => {
            let mut valuevec = ValueParser::new(vec![token]).parse_value();
            let value = if valuevec.len() == 1 {
                std::mem::take(&mut valuevec[0])
            } else {
                LocaValue::Concat(valuevec)
            };
            if key.is("datacontext") {
                if let LocaValue::Code(chain, _) = value {
                    // TODO: figure out the actual scope context here. Perhaps it should be a strict scope with no root and no names defined?
                    let mut sc = ScopeContext::new_unrooted(Scopes::all(), key);
                    sc.set_strict_scopes(false);
                    validate_datatypes(&chain, data, &mut sc, Datatype::Unknown, "", true);
                } else {
                    let msg = "expected whole field to be a single [ ] clause";
                    old_warn(token, ErrorKey::Validation, msg);
                }
            } else {
                validate_gui_loca(key, value, data);
            }
        }
        BV::Block(block) => {
            validate_gui(block, data);
        }
    }
}

fn validate_gui_loca(key: &Token, value: LocaValue, data: &Everything) {
    match value {
        LocaValue::Concat(v) => {
            for value in v {
                validate_gui_loca(key, value, data);
            }
        }
        #[allow(unused_variables)] // vic3 does not use fmt
        LocaValue::Code(chain, fmt) => {
            // |E is the formatting used for game concepts in ck3
            #[cfg(feature = "ck3")]
            if Game::is_ck3() {
                if let Some(fmt) = fmt {
                    if fmt.as_str().contains('E') || fmt.as_str().contains('e') {
                        if let Some(name) = chain.as_gameconcept() {
                            data.verify_exists(Item::GameConcept, name);
                            return;
                        }
                    }
                }
            }

            // TODO: figure out the actual scope context here. Perhaps it should be a strict scope with no root and no names defined?
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), key);
            sc.set_strict_scopes(false);
            validate_datatypes(&chain, data, &mut sc, Datatype::Unknown, "", false);
        }
        _ => (),
    }
}

/// Widget types that are defined by the game engine and don't need to be defined in gui script.
// There might be some more that should be feature = "ck3". TODO: compare vic3 and ck3 vanilla
const BUILTIN_TYPES: &[&str] = &[
    "active_item",
    "animation",
    "attachto",
    "background",
    #[cfg(feature = "vic3")]
    "button",
    "buttontext",
    "button_group",
    "cameracontrolwidget",
    "checkbutton",
    "click_modifiers",
    "colormap_picker",
    "colorpicker",
    "colorpicker_reticule_icon",
    "container",
    "contextmenu",
    "datacontext_from_model",
    "drag_drop_icon",
    "drag_drop_target",
    "dockable_container",
    "dropdown",
    "dynamicgridbox",
    "editbox",
    "expand_item",
    "expandbutton",
    "fixedgridbox",
    "flowcontainer",
    #[cfg(feature = "ck3")]
    "game_button",
    "glow",
    "glow_generation_rules",
    "hbox",
    "icon",
    "icon_button_small_round",
    "item",
    "keyframe_editor_lane_container",
    "line",
    "line_deprecated",
    "list",
    "margin_widget",
    "marker",
    #[cfg(feature = "vic3")]
    "minimap",
    #[cfg(feature = "vic3")]
    "minimap_window",
    "modify_texture",
    "overlappingitembox",
    #[cfg(feature = "vic3")]
    "piechart",
    #[cfg(feature = "vic3")]
    "pieslice",
    #[cfg(feature = "vic3")]
    "plotline",
    "portrait_button",
    "progressbar",
    #[cfg(feature = "vic3")]
    "right_click_menu_widget",
    "scrollarea",
    "scrollbar",
    "scrollbar_horizontal",
    "scrollbar_vertical",
    "scrollwidget",
    "state",
    "text_occluder",
    "textbox",
    "timeline_texts",
    "tools_dragdrop_widget",
    "tools_player_timeline",
    "tools_keyframe_button",
    "tools_keyframe_editor",
    "tools_keyframe_editor_lane",
    "tools_table",
    "tree",
    #[cfg(feature = "vic3")]
    "treemapslice",
    "vbox",
    #[cfg(feature = "vic3")]
    "webwindow",
    "widget",
    "window",
    "zoomarea",
];
