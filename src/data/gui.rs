//! Validate files in `gui/`

use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::{Block, BlockItem, Comparator, Eq::Single, Field, BV};
use crate::context::ScopeContext;
use crate::data::localization::LocaValue;
use crate::datatype::{validate_datatypes, Datatype};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
#[cfg(feature = "ck3")]
use crate::game::Game;
use crate::game::GameFlags;
use crate::helpers::{dup_error, stringify_choices};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::parse::localization::ValueParser;
use crate::pdxfile::PdxFile;
use crate::report::{
    err, error, error_info, old_warn, untidy, warn, warn_info, ErrorKey, Severity,
};
use crate::scopes::Scopes;
use crate::token::Token;

use GuiValidation::*;

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
        let base_lc = self.base.as_str().to_lowercase();
        if base_lc == self.key.as_str().to_lowercase() && !BUILTIN_TYPES.contains(&&*base_lc) {
            err(ErrorKey::Loop)
                .msg("recursive definition of non-builtin type")
                .loc(&self.key)
                .push();
        }
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
                        warn(ErrorKey::ParseError).msg("unexpected value").loc(token).push();
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

fn validate_known_field(key: &Token, bv: &BV, data: &Everything, validation: GuiValidation) {
    match validation {
        GuiValidation::Template => {
            if let Some(value) = bv.expect_value() {
                data.verify_exists(Item::GuiTemplate, value);
                // Templates are validated separately, so we're done.
            }
        }
        GuiValidation::UncheckedValue | GuiValidation::Format => {
            // TODO: validate Format as a format string
            _ = bv.expect_value();
        }
        GuiValidation::Datatype | GuiValidation::Datamodel => {
            validate_datatype_field(Datatype::Unknown, key, bv, data, false);
        }
        GuiValidation::Datacontext => {
            validate_datatype_field(Datatype::Unknown, key, bv, data, true);
        }
        GuiValidation::Boolean => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::bool, key, bv, data, false);
                } else if !value.lowercase_is("yes") && !value.lowercase_is("no") {
                    // TODO: decide based on the field name whether to upgrade to error?
                    warn(ErrorKey::Validation).msg("expected yes or no").loc(value).push();
                }
            }
        }
        GuiValidation::Align => {
            if let Some(value) = bv.expect_value() {
                for part in value.split('|') {
                    if !ALIGN.contains(&part.as_str()) {
                        let msg = format!("unknown {key} {part}");
                        let info = format!("known {key}s are {}", stringify_choices(ALIGN));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(part).push();
                    }
                }
            }
        }
        GuiValidation::Integer => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::int32, key, bv, data, false);
                } else {
                    value.expect_integer();
                }
            }
        }
        GuiValidation::UnsignedInteger => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::uint32, key, bv, data, false);
                } else if let Some(i) = value.expect_integer() {
                    if i < 0 {
                        let msg = format!("{key} needs an unsigned integer");
                        warn(ErrorKey::Range).msg(msg).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::Number => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    validate_datatype_field(Datatype::float, key, bv, data, false);
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::NumberOrInt32 => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need a way to express it can be int32 or float
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::NumberF => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need a way to express it can be int32 or float
                    validate_datatype_field(Datatype::float, key, bv, data, false);
                } else if let Some(value) = value.strip_suffix("f") {
                    // TODO: this f is used in vanilla; check it really works.
                    value.expect_number();
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::NumberOrPercent => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need a way to express it can be int32 or float
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else if let Some(value) = value.strip_suffix("%") {
                    value.expect_number();
                } else {
                    value.expect_number();
                }
            }
        }
        GuiValidation::TwoNumberOrPercent => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector2f, key, bv, data, false);
            }
            BV::Block(block) => {
                for value in block.iter_values_warn() {
                    if let Some(value) = value.strip_suffix("%") {
                        value.expect_number();
                    } else {
                        value.expect_number();
                    }
                }
            }
        },
        GuiValidation::CVector2f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector2f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_numbers_exactly(2);
            }
        },
        GuiValidation::CVector2i => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector2i, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_integers_exactly(2);
            }
        },
        GuiValidation::CVector3f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector3f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_numbers_exactly(3);
            }
        },
        GuiValidation::CVector4f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector4f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_tokens_numbers_exactly(4);
            }
        },
        GuiValidation::Color => match bv {
            BV::Value(_) => {
                // TODO: can be CVector4f or CString
                validate_datatype_field(Datatype::Unknown, key, bv, data, false);
            }
            BV::Block(block) => {
                validate_gui_color(block, data);
            }
        },
        GuiValidation::CString => {
            validate_datatype_field(Datatype::CString, key, bv, data, false);
        }
        GuiValidation::Item(itype) => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need some way of specifying "stringable" datatypes
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else {
                    data.verify_exists(itype, value);
                }
            }
        }
        GuiValidation::ItemOrBlank(itype) => {
            if let Some(value) = bv.expect_value() {
                if value.starts_with("[") {
                    // TODO: need some way of specifying "stringable" datatypes
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else if !value.is("") {
                    data.verify_exists(itype, value);
                }
            }
        }
        GuiValidation::Blendmode => {
            if let Some(value) = bv.expect_value() {
                let value_lc = value.as_str().to_lowercase();
                if !BLENDMODES.contains(&&*value_lc) {
                    let msg = "unknown blendmode";
                    let info = format!("expected one of {}", stringify_choices(BLENDMODES));
                    warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                }
            }
        }
        GuiValidation::MouseButton(choices) => {
            if let Some(value) = bv.expect_value() {
                // TODO: datatype is only really used by button_ignore.
                // Is it valid for the others?
                if value.starts_with("[") {
                    // TODO: need some way of specifying "stringable" datatypes
                    validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                } else {
                    let value_lc = value.as_str().to_lowercase();
                    if !choices.contains(&&*value_lc) {
                        let msg = "unknown mouse button";
                        let info = format!("expected one of {}", stringify_choices(choices));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::MouseButtonSet(choices) => {
            if let Some(value) = bv.expect_value() {
                for part in value.split('|') {
                    let part_lc = part.as_str().to_lowercase();
                    if !choices.contains(&&*part_lc) {
                        let msg = "unknown mouse button";
                        let info = format!("expected one of {}", stringify_choices(choices));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::Choice(choices) => {
            if let Some(value) = bv.expect_value() {
                let value_lc = value.as_str().to_lowercase();
                if !choices.contains(&&*value_lc) {
                    let msg = "unknown value";
                    let info = format!("expected one of {}", stringify_choices(choices));
                    warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                }
            }
        }
        GuiValidation::ChoiceSet(choices) => {
            if let Some(value) = bv.expect_value() {
                for part in value.split('|') {
                    let part_lc = part.as_str().to_lowercase();
                    if !choices.contains(&&*part_lc) {
                        let msg = "unknown value";
                        let info = format!("expected one of {}", stringify_choices(choices));
                        warn(ErrorKey::Choice).msg(msg).info(info).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::Widget => {
            match bv {
                BV::Value(value) => {
                    data.verify_exists(Item::GuiTemplate, value);
                    // TODO: verify that this is a template containing one widget.
                }
                BV::Block(block) => {
                    if !block.iter_items().count() == 1 {
                        let msg = format!("{key} should have a block with just one widget");
                        err(ErrorKey::Validation).msg(msg).loc(block).push();
                    }
                    validate_gui(block, data);
                }
            }
        }
        GuiValidation::FormatOverride => {
            if let Some(block) = bv.expect_block() {
                let mut count = 0;
                for value in block.iter_values_warn() {
                    count += 1;
                    data.verify_exists(Item::TextFormat, value);
                    if count == 3 {
                        let msg = "expected exactly 2 text formats";
                        warn(ErrorKey::Validation).msg(msg).loc(value).push();
                    }
                }
            }
        }
        GuiValidation::RawText => {
            if let Some(value) = bv.expect_value() {
                let valuevec = ValueParser::new(vec![value]).parse_value();
                for v in valuevec {
                    validate_gui_loca(key, v, data);
                }
                if !value.starts_with("[") {
                    // raw text can still be a localization key sometimes
                    data.mark_used(Item::Localization, value.as_str());
                }
            }
        }
        GuiValidation::Text => {
            if let Some(value) = bv.expect_value() {
                let valuevec = ValueParser::new(vec![value]).parse_value();
                for v in valuevec {
                    validate_gui_loca(key, v, data);
                }
                if !value.starts_with("[") && !value.as_str().contains(' ') {
                    data.verify_exists(Item::Localization, value);
                }
            }
        }
    }
}

fn validate_field(field: &Field, data: &Everything) {
    let Field(key, cmp, bv) = field;

    if *cmp != Comparator::Equals(Single) {
        let msg = format!("expected only `key =`, not `{cmp}`");
        untidy(ErrorKey::Validation).msg(msg).loc(key).push();
    }

    let key_lc = key.as_str().to_lowercase();
    let game = GameFlags::game();
    for (name, validation, gameflags) in GUI_FIELDS {
        if key_lc == *name {
            if gameflags.contains(game) {
                validate_known_field(key, bv, data, *validation);
            } else {
                let msg = format!("{key} is only for {gameflags}");
                err(ErrorKey::WrongGame).weak().msg(msg).loc(key).push();
            }
            return;
        }
    }
    match bv {
        BV::Value(_) => {
            let msg = format!("unknown gui field {key}");
            err(ErrorKey::UnknownField).weak().msg(msg).loc(key).push();
        }
        BV::Block(block) => {
            data.verify_exists(Item::GuiType, key);
            validate_gui(block, data);
        }
    }
}

fn validate_datatype_field(
    dtype: Datatype,
    key: &Token,
    bv: &BV,
    data: &Everything,
    allow_promote: bool,
) {
    if let Some(value) = bv.expect_value() {
        if value.starts_with("[") {
            let valuevec = ValueParser::new(vec![value]).parse_value();
            if valuevec.len() == 1 {
                let mut sc = ScopeContext::new(Scopes::None, key);
                match &valuevec[0] {
                    // TODO: validate format
                    LocaValue::Code(chain, format) => {
                        validate_datatypes(
                            chain,
                            data,
                            &mut sc,
                            dtype,
                            "",
                            format.as_ref(),
                            allow_promote,
                        );
                    }
                    LocaValue::Error => (),
                    _ => {
                        let msg = "expected whole field to be a [ ] expression";
                        warn(ErrorKey::Validation).msg(msg).loc(value).push();
                    }
                }
            } else {
                let msg = "expected whole field to be a single [ ] expression";
                warn(ErrorKey::Validation).msg(msg).loc(value).push();
            }
        } else {
            let msg = "expected a [ ] expression here";
            warn(ErrorKey::Validation).msg(msg).loc(value).push();
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
        LocaValue::Code(chain, format) => {
            // |E is the formatting used for game concepts in ck3
            #[cfg(feature = "ck3")]
            if Game::is_ck3() {
                if let Some(ref format) = format {
                    if format.as_str().contains('E') || format.as_str().contains('e') {
                        if let Some(name) = chain.as_gameconcept() {
                            data.verify_exists(Item::GameConcept, name);
                            return;
                        }
                    }
                }
            }

            let mut sc = ScopeContext::new(Scopes::None, key);
            validate_datatypes(
                &chain,
                data,
                &mut sc,
                Datatype::Unknown,
                "",
                format.as_ref(),
                false,
            );
        }
        _ => (),
    }
}

fn validate_gui_color(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut count = 0;
    for token in vd.values() {
        count += 1;
        // TODO: verify whether gui really does support precise numbers.
        // They're used in a few places by vanilla but that doesn't mean it works...
        // TODO: check ranges
        token.expect_precise_number();
    }
    if count != 4 {
        warn(ErrorKey::Colors).msg("expected 4 color values").loc(block).push();
    }
}

const LAYOUT_POLICIES: &[&str] = &["expanding", "fixed", "growing", "preferred", "shrinking"];

const BLENDMODES: &[&str] =
    &["add", "alphamultiply", "colordodge", "darken", "mask", "multiply", "normal", "overlay"];

// TODO: warn about contradicting alignments (left|right or top|vcenter)
// TODO: is nobaseline only for text widgets?
const ALIGN: &[&str] =
    &["left", "right", "top", "bottom", "center", "hcenter", "vcenter", "nobaseline"];

/// The various values or blocks or datatype calculations that the gui fields can take.
#[derive(Debug, Clone, Copy)]
enum GuiValidation {
    /// The name of a template to inline
    Template,
    /// Accept any value; we don't know.
    UncheckedValue,
    /// A datatype expression; we don't know the specific type.
    Datatype,
    /// A datatype expression that ends with a promote. Can be any type.
    // TODO: check that it is a singular type, not a collection
    Datacontext,
    /// A datatype expression that returns a collection of some sort.
    // TODO: check that it is indeed a collection
    Datamodel,
    /// "yes", "no", or a [`Datatype::bool`] expression.
    Boolean,
    /// A `|`-separated list of values from [`ALIGN`].
    Align,
    /// An integer value or a [`Datatype::int32`] expression.
    Integer,
    /// A non-negative integer value or a [`Datatype::uint32`] expression.
    UnsignedInteger,
    /// A numeric value or a [`Datatype::float`] expression.
    Number,
    /// A numeric value or a [`Datatype::float`] or [`Datatype::int32`] expression.
    NumberOrInt32,
    /// A numeric value possibly followed by an `f` or a [`Datatype::float`] expression.
    // TODO: the f is used in vanilla. Check if it's harmless or maybe required.
    NumberF,
    /// A numeric value, possibly followed by a `%` mark, or a [`Datatype::int32`] or
    /// [`Datatype::float`] expression.
    NumberOrPercent,
    /// A block with two numeric values, possibly followed by `%` marks, or a
    /// [`Datatype::CVector2f`] expression.
    TwoNumberOrPercent,
    /// A block with 2 numbers, or a [`Datatype::CVector2f`] expression.
    CVector2f,
    /// A block with 2 integers, or a [`Datatype::CVector2i`] expression.
    CVector2i,
    /// A block with 3 numbers, or a [`Datatype::CVector3f`] expression.
    CVector3f,
    /// A block with 4 numbers, or a [`Datatype::CVector4f`] expression.
    CVector4f,
    /// A block with 4 numbers (RGBA), or a [`Datatype::CVector4f`] expression.
    /// The difference with [`GuiValidation::CVector4f`] is in the error messages.
    Color,
    /// A datatype expression returning a CString
    CString,
    /// A key for this item type, or a datatype expression that can be converted to string.
    Item(Item),
    /// A key for this item type, or a datatype expression that can be converted to string.
    /// May be the empty string.
    ItemOrBlank(Item),
    /// One of the strings in the [`BLENDMODES`] array
    Blendmode,
    /// One of the mouse buttons in the array given here.
    MouseButton(&'static [&'static str]),
    /// A `|`-separated list of the mouse buttons in the array given here.
    MouseButtonSet(&'static [&'static str]),
    /// One of the strings in the array given here.
    Choice(&'static [&'static str]),
    /// A `|`-separated list of strings from the array given here.
    ChoiceSet(&'static [&'static str]),
    /// A block containing one widget, or the name of a template containing one widget.
    Widget,
    /// A string containing a localization text format specifier starting with `#`
    Format,
    /// A block containing two textformat names
    FormatOverride,
    /// Some text. May contain datatypes or loca keys, but it's optional.
    RawText,
    /// Some text. May contain datatypes or loca keys.
    Text,
}

/// Definitions of all fields that can be used in gui widgets.
// TODO - imperator - remove the non-imperator ones from GameFlags::all(), and
// add any that are missing. It may help to define a GameFlags::Ck3Vic3 for convenience.
const GUI_FIELDS: &[(&str, GuiValidation, GameFlags)] = &[
    ("accept_tabs", Boolean, GameFlags::all()),
    ("addcolumn", NumberOrPercent, GameFlags::all()),
    ("addrow", NumberOrPercent, GameFlags::all()),
    ("align", Align, GameFlags::all()),
    ("allow_outside", Boolean, GameFlags::all()),
    ("alpha", Number, GameFlags::all()),
    ("alwaystransparent", Boolean, GameFlags::all()),
    ("animate_negative_changes", Boolean, GameFlags::Vic3),
    ("animation_speed", CVector2f, GameFlags::all()),
    ("autoresize", Boolean, GameFlags::all()),
    ("autoresize_slider", Boolean, GameFlags::all()),
    ("autoresizescrollarea", Boolean, GameFlags::all()),
    ("autoresizeviewport", Boolean, GameFlags::all()),
    ("background_texture", Item(Item::File), GameFlags::all()),
    ("bezier", CVector4f, GameFlags::all()),
    ("blend_mode", Blendmode, GameFlags::all()),
    ("button_ignore", MouseButton(&["both", "none", "left", "right"]), GameFlags::Ck3),
    ("button_trigger", UncheckedValue, GameFlags::all()), // only example is "none"
    ("camera_fov_y_degrees", Integer, GameFlags::all()),
    ("camera_look_at", CVector3f, GameFlags::all()),
    ("camera_near_far", CVector2f, GameFlags::all()),
    ("camera_position", CVector3f, GameFlags::all()),
    ("camera_rotation_pitch_limits", CVector2f, GameFlags::Ck3),
    ("camera_translation_limits", CVector3f, GameFlags::Ck3),
    ("camera_zoom_limits", CVector2f, GameFlags::all()),
    ("checked", Boolean, GameFlags::all()),
    ("clicksound", ItemOrBlank(Item::Sound), GameFlags::all()),
    ("coat_of_arms", Item(Item::File), GameFlags::Ck3),
    ("coat_of_arms_mask", Item(Item::File), GameFlags::Ck3),
    ("coat_of_arms_slot", CVector4f, GameFlags::all()),
    ("color", Color, GameFlags::all()),
    ("constantbuffers", Datatype, GameFlags::all()),
    ("cursorcolor", Color, GameFlags::all()),
    ("datacontext", Datacontext, GameFlags::all()),
    ("datamodel", Datamodel, GameFlags::all()),
    ("datamodel_reuse_widgets", Boolean, GameFlags::Ck3),
    ("datamodel_wrap", Integer, GameFlags::all()),
    ("dec_button", Widget, GameFlags::all()),
    ("default_clicksound", ItemOrBlank(Item::Sound), GameFlags::Ck3),
    ("default_format", Format, GameFlags::all()),
    ("delay", Number, GameFlags::all()),
    ("direction", Choice(&["horizontal", "vertical"]), GameFlags::all()),
    ("disableframe", Integer, GameFlags::all()),
    ("distribute_visual_state", Boolean, GameFlags::all()),
    ("down", Boolean, GameFlags::all()),
    ("downframe", Integer, GameFlags::all()),
    ("downhoverframe", Integer, GameFlags::all()),
    ("downpressedframe", Integer, GameFlags::all()),
    ("drag_drop_args", CString, GameFlags::Ck3),
    ("drag_drop_base_type", Choice(&["icon", "coat_of_arms_icon"]), GameFlags::Ck3),
    ("drag_drop_data", Datacontext, GameFlags::all()),
    ("drag_drop_id", UncheckedValue, GameFlags::Ck3), // TODO what are the options?
    ("draggable_by", MouseButtonSet(&["left", "right", "middle"]), GameFlags::all()),
    ("droptarget", Boolean, GameFlags::all()),
    ("duration", Number, GameFlags::all()),
    ("effect", Datatype, GameFlags::all()),
    ("effectname", UncheckedValue, GameFlags::all()), // TODO validate effect names
    ("elide", Choice(&["right", "middle", "left"]), GameFlags::all()),
    ("enabled", Boolean, GameFlags::all()),
    ("entity_enable_sound", Boolean, GameFlags::Vic3),
    ("entity_instance", Item(Item::Entity), GameFlags::all()),
    ("even_row_widget", Widget, GameFlags::all()),
    ("filter_mouse", MouseButtonSet(&["all", "none", "left", "right", "wheel"]), GameFlags::all()),
    ("fittype", Choice(&["center", "centercrop", "fill", "end", "start"]), GameFlags::all()),
    ("flipdirection", Boolean, GameFlags::all()),
    ("focus_on_visible", Boolean, GameFlags::all()),
    ("focuspolicy", Choice(&["click", "all", "none"]), GameFlags::all()),
    ("font", Item(Item::Font), GameFlags::all()),
    ("fontcolor", Color, GameFlags::all()),
    ("fontsize", Integer, GameFlags::all()),
    ("fontsize_min", Integer, GameFlags::all()),
    ("fonttintcolor", Color, GameFlags::all()),
    ("fontweight", UncheckedValue, GameFlags::all()), // TODO: what are the options?
    ("force_data_properties_update", Boolean, GameFlags::Vic3),
    ("format_override", FormatOverride, GameFlags::all()),
    ("frame", Integer, GameFlags::all()),
    ("framesize", CVector2i, GameFlags::all()),
    ("from", CVector2f, GameFlags::all()),
    ("gfx_environment_file", Item(Item::File), GameFlags::all()),
    ("gfxtype", UncheckedValue, GameFlags::all()), // TODO: what are the options?
    ("glow_alpha", Number, GameFlags::Ck3),
    ("glow_alpha_mask", Integer, GameFlags::Ck3),
    ("glow_blur_passes", Integer, GameFlags::Ck3),
    ("glow_ignore_inside_pixels", Boolean, GameFlags::Ck3),
    ("glow_radius", Integer, GameFlags::Ck3),
    ("glow_texture_downscale", NumberF, GameFlags::Ck3),
    ("grayscale", Boolean, GameFlags::all()),
    ("grid_entity_name", Item(Item::Entity), GameFlags::all()),
    ("highlightchecked", Boolean, GameFlags::Ck3),
    ("header_height", Integer, GameFlags::all()),
    ("ignore_in_debug_draw", Boolean, GameFlags::all()),
    // middle and left are guesses
    ("ignore_unset_buttons", MouseButtonSet(&["right", "middle", "left"]), GameFlags::Vic3),
    ("ignoreinvisible", Boolean, GameFlags::all()),
    ("inc_button", Widget, GameFlags::all()),
    ("indent", Integer, GameFlags::all()),
    ("index", Integer, GameFlags::Ck3),
    ("inherit_data_context", Boolean, GameFlags::Ck3),
    ("inherit_visibility", Choice(&["yes", "no", "hidden"]), GameFlags::all()),
    ("inherit_visual_state", Boolean, GameFlags::all()),
    ("input_action", Item(Item::Shortcut), GameFlags::Vic3),
    ("intersectionmask", Boolean, GameFlags::all()),
    ("intersectionmask_texture", Item(Item::File), GameFlags::Ck3),
    ("invert_reticule_color", Boolean, GameFlags::all()),
    ("invertprogress", Boolean, GameFlags::all()),
    ("layer", Item(Item::GuiLayer), GameFlags::all()),
    ("layoutanchor", UncheckedValue, GameFlags::all()), // TODO: only example is "bottomleft"
    ("layoutpolicy_horizontal", ChoiceSet(LAYOUT_POLICIES), GameFlags::all()),
    ("layoutpolicy_vertical", ChoiceSet(LAYOUT_POLICIES), GameFlags::all()),
    ("layoutstretchfactor_horizontal", NumberOrInt32, GameFlags::all()),
    ("layoutstretchfactor_vertical", NumberOrInt32, GameFlags::all()),
    ("line_cap", Boolean, GameFlags::all()),
    ("line_feather_distance", Integer, GameFlags::all()),
    ("line_type", UncheckedValue, GameFlags::all()), // TODO: only example is "nodeline"
    ("loop", Boolean, GameFlags::all()),
    ("loopinterval", Number, GameFlags::all()),
    ("margin", TwoNumberOrPercent, GameFlags::all()),
    ("margin_bottom", NumberOrInt32, GameFlags::all()),
    ("margin_left", NumberOrInt32, GameFlags::all()),
    ("margin_right", NumberOrInt32, GameFlags::all()),
    ("margin_top", NumberOrInt32, GameFlags::all()),
    ("mask", Item(Item::File), GameFlags::all()),
    ("mask_uv_scale", CVector2f, GameFlags::all()),
    ("max", NumberOrInt32, GameFlags::all()),
    ("max_update_rate", Integer, GameFlags::all()),
    ("max_width", Integer, GameFlags::all()),
    ("maxcharacters", UnsignedInteger, GameFlags::all()),
    ("maxhorizontalslots", Integer, GameFlags::all()),
    ("maximumsize", TwoNumberOrPercent, GameFlags::all()),
    ("maxverticalslots", Integer, GameFlags::all()),
    ("min", NumberOrInt32, GameFlags::all()),
    ("min_dist_from_screen_edge", Integer, GameFlags::Ck3),
    ("min_width", Integer, GameFlags::all()),
    ("minimumsize", TwoNumberOrPercent, GameFlags::all()),
    ("mipmaplodbias", Integer, GameFlags::all()),
    ("mirror", Choice(&["horizontal", "vertical"]), GameFlags::all()),
    ("modal", Boolean, GameFlags::all()),
    ("modality", UncheckedValue, GameFlags::all()), // TODO: only example is "all"
    ("movable", Boolean, GameFlags::all()),
    ("multiline", Boolean, GameFlags::all()),
    ("name", UncheckedValue, GameFlags::all()),
    ("next", UncheckedValue, GameFlags::all()), // TODO: choices are states in the same widget
    ("noprogresstexture", Item(Item::File), GameFlags::all()),
    ("odd_row_widget", Widget, GameFlags::all()),
    ("on_finish", Datatype, GameFlags::all()),
    ("on_keyframe_move", Datatype, GameFlags::all()),
    ("on_start", Datatype, GameFlags::all()),
    ("onalt", Datatype, GameFlags::Vic3),
    ("onchangefinish", Datatype, GameFlags::all()),
    ("onchangestart", Datatype, GameFlags::all()),
    ("onclick", Datatype, GameFlags::all()),
    ("oncolorchanged", Datatype, GameFlags::Ck3),
    ("oncoloredited", Datatype, GameFlags::Ck3),
    ("oncreate", Datatype, GameFlags::all()),
    ("ondefault", Datatype, GameFlags::all()),
    ("ondoubleclick", Datatype, GameFlags::all()),
    ("oneditingfinished", Datatype, GameFlags::all()),
    ("oneditingfinished_with_changes", Datatype, GameFlags::all()),
    ("oneditingstart", Datatype, GameFlags::all()),
    ("onfocusout", Datatype, GameFlags::all()),
    ("onmousehierarchyenter", Datatype, GameFlags::all()),
    ("onmousehierarchyleave", Datatype, GameFlags::all()),
    ("onpressed", Datatype, GameFlags::all()),
    ("onreleased", Datatype, GameFlags::all()),
    ("onreturnpressed", Datatype, GameFlags::all()),
    ("onrightclick", Datatype, GameFlags::all()),
    ("onselectionchanged", Datatype, GameFlags::all()),
    ("onshift", Datatype, GameFlags::all()),
    ("ontextchanged", Datatype, GameFlags::all()),
    ("ontextedited", Datatype, GameFlags::all()),
    ("onvaluechanged", Datatype, GameFlags::all()),
    ("overframe", Integer, GameFlags::all()),
    ("oversound", ItemOrBlank(Item::Sound), GameFlags::all()),
    ("page", Integer, GameFlags::all()),
    ("pan_position", CVector2f, GameFlags::all()),
    ("parentanchor", Align, GameFlags::all()),
    ("password", Boolean, GameFlags::all()),
    ("plotpoints", Datatype, GameFlags::Vic3),
    ("points", Datatype, GameFlags::all()),
    ("pop_out", Boolean, GameFlags::all()),
    ("portrait_context", Datatype, GameFlags::all()),
    ("portrait_offset", CVector2f, GameFlags::all()),
    ("portrait_scale", CVector2f, GameFlags::all()),
    ("portrait_texture", Item(Item::File), GameFlags::all()),
    ("position", TwoNumberOrPercent, GameFlags::all()),
    ("position_x", Integer, GameFlags::all()),
    ("position_y", Integer, GameFlags::all()),
    ("preferscrollwidgetsize", Boolean, GameFlags::Vic3),
    ("progress_change_to_duration_curve", CVector4f, GameFlags::all()),
    ("progresstexture", Item(Item::File), GameFlags::all()),
    ("pseudo_localization_enabled", Boolean, GameFlags::all()),
    ("raw_text", RawText, GameFlags::all()),
    ("raw_tooltip", RawText, GameFlags::all()),
    ("realtime", Boolean, GameFlags::Vic3),
    ("reorder_on_mouse", UncheckedValue, GameFlags::all()), // TODO: only example is "presstop"
    ("recursive", Boolean, GameFlags::Ck3),
    ("resizable", Boolean, GameFlags::all()),
    ("resizeparent", Boolean, GameFlags::all()),
    ("restart_on_show", Boolean, GameFlags::Ck3),
    ("restrictparent_min", Boolean, GameFlags::all()),
    ("reuse_widgets", Boolean, GameFlags::all()),
    ("righttoleft", Boolean, GameFlags::all()),
    ("rotate_uv", Number, GameFlags::all()),
    ("row_height", Integer, GameFlags::all()),
    ("scale", Number, GameFlags::all()),
    ("scale_mode", UncheckedValue, GameFlags::all()), // TODO: only example is "fixedwidth"
    ("scissor", Boolean, GameFlags::all()),
    ("scrollbaralign_horizontal", Align, GameFlags::all()),
    ("scrollbaralign_vertical", Align, GameFlags::all()),
    // TODO: always_on is a guess
    (
        "scrollbarpolicy_horizontal",
        Choice(&["as_needed", "always_off", "always_on"]),
        GameFlags::all(),
    ),
    (
        "scrollbarpolicy_vertical",
        Choice(&["as_needed", "always_off", "always_on"]),
        GameFlags::all(),
    ),
    ("selectallonfocus", Boolean, GameFlags::all()),
    ("selectedindex", CVector2i, GameFlags::all()),
    ("selectioncolor", Color, GameFlags::all()),
    ("set_parent_size_to_minimum", Boolean, GameFlags::all()),
    ("setitemsizefromcell", Boolean, GameFlags::all()),
    ("shaderfile", ItemOrBlank(Item::File), GameFlags::all()),
    ("shortcut", Item(Item::Shortcut), GameFlags::all()),
    ("size", TwoNumberOrPercent, GameFlags::all()),
    ("skip_initial_animation", Boolean, GameFlags::Vic3),
    ("slider", Widget, GameFlags::all()),
    ("snap_to_pixels", Boolean, GameFlags::Ck3),
    ("soundeffect", Item(Item::Sound), GameFlags::all()),
    ("spacing", Integer, GameFlags::all()),
    ("spriteborder", CVector2f, GameFlags::all()),
    ("spriteborder_bottom", Integer, GameFlags::all()),
    ("spriteborder_left", Integer, GameFlags::all()),
    ("spriteborder_right", Integer, GameFlags::all()),
    ("spriteborder_top", Integer, GameFlags::all()),
    ("spritetype", UncheckedValue, GameFlags::all()), // TODO
    ("step", NumberOrInt32, GameFlags::all()),
    ("stackmode", UncheckedValue, GameFlags::Ck3), // TODO only example is "top"
    ("sticky", Boolean, GameFlags::all()),
    ("tabfocusroot", Boolean, GameFlags::all()),
    ("text", Text, GameFlags::all()),
    ("text_selectable", Boolean, GameFlags::all()),
    ("text_validator", Datatype, GameFlags::all()),
    ("texture", Item(Item::File), GameFlags::all()),
    ("texture_density", Number, GameFlags::all()),
    ("timeline_line_direction", UncheckedValue, GameFlags::all()), // TODO only example is "up"
    ("timeline_line_height", Integer, GameFlags::all()),
    ("timeline_time_points", Integer, GameFlags::all()),
    ("tintcolor", Color, GameFlags::all()),
    ("to", CVector2f, GameFlags::all()),
    ("tooltip", Text, GameFlags::all()),
    ("tooltip_enabled", Boolean, GameFlags::all()),
    ("tooltip_horizontalbehavior", Choice(&["mirror", "slide", "flip"]), GameFlags::all()),
    ("tooltip_offset", TwoNumberOrPercent, GameFlags::all()),
    ("tooltip_parentanchor", Align, GameFlags::all()),
    ("tooltip_type", Choice(&["mouse", "widget"]), GameFlags::all()),
    ("tooltip_verticalbehavior", Choice(&["mirror", "slide", "flip"]), GameFlags::all()),
    ("tooltip_widgetanchor", Align, GameFlags::all()),
    ("tooltipwidget", Widget, GameFlags::all()),
    ("track", Widget, GameFlags::all()),
    ("tracknavigation", UncheckedValue, GameFlags::all()), // TODO only example is "direct"
    ("translate_uv", CVector2f, GameFlags::all()),
    ("trigger_on_create", Boolean, GameFlags::all()),
    ("trigger_when", Boolean, GameFlags::all()),
    ("upframe", Integer, GameFlags::all()),
    ("uphoverframe", Integer, GameFlags::all()),
    ("uppressedframe", Integer, GameFlags::all()),
    ("useragent", UncheckedValue, GameFlags::Vic3),
    ("using", Template, GameFlags::all()),
    ("uv_scale", CVector2f, GameFlags::all()),
    ("value", NumberOrInt32, GameFlags::all()),
    ("video", Item(Item::File), GameFlags::all()),
    ("viewportwidget", Widget, GameFlags::all()),
    ("visible", Boolean, GameFlags::all()),
    ("visible_at_creation", Boolean, GameFlags::all()),
    ("wheelstep", NumberOrInt32, GameFlags::all()),
    ("widgetanchor", Align, GameFlags::all()),
    ("widgetid", UncheckedValue, GameFlags::all()),
    ("width", Number, GameFlags::all()),
    ("zoom", Number, GameFlags::all()),
    ("zoom_max", Number, GameFlags::all()),
    ("zoom_min", Number, GameFlags::all()),
    ("zoom_step", Number, GameFlags::all()),
    ("zoomwidget", Widget, GameFlags::all()),
];

/// Widget types that are defined by the game engine and don't need to be defined in gui script.
// TODO: apply GameFlags here too
const BUILTIN_TYPES: &[&str] = &[
    "active_item",
    "animation",
    "attachto",
    #[cfg(feature = "vic3")]
    "axis",
    #[cfg(feature = "vic3")]
    "axis_label",
    "background",
    #[cfg(feature = "vic3")]
    "button",
    "button_group",
    "buttontext",
    "cameracontrolwidget",
    "checkbutton",
    "click_modifiers",
    "colormap_picker",
    "colorpicker",
    "colorpicker_reticule_icon",
    "container",
    "contextmenu",
    #[cfg(feature = "ck3")]
    "datacontext_from_model",
    #[cfg(feature = "ck3")]
    "drag_drop_icon",
    #[cfg(feature = "ck3")]
    "drag_drop_target",
    "dockable_container",
    "dropdown",
    "dynamicgridbox",
    "end_sound",
    "editbox",
    "expand_item",
    "expandbutton",
    "fixedgridbox",
    "flowcontainer",
    #[cfg(feature = "ck3")]
    "game_button",
    #[cfg(feature = "ck3")]
    "glow",
    #[cfg(feature = "ck3")]
    "glow_generation_rules",
    "hbox",
    "icon",
    #[cfg(feature = "ck3")]
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
    #[cfg(feature = "vic3")]
    "rightclick_modifiers",
    "scrollarea",
    "scrollbar",
    "scrollbar_horizontal",
    "scrollbar_vertical",
    "scrollwidget",
    #[cfg(feature = "ck3")]
    "soundparam", // TODO: this contains name and value fields which refer to the parameter:/ sounds
    "start_sound",
    "state",
    "text_occluder",
    "textbox",
    "timeline_texts",
    "tools_dragdrop_widget",
    "tools_keyframe_button",
    "tools_keyframe_editor",
    "tools_keyframe_editor_lane",
    "tools_player_timeline",
    "tools_table",
    "tree",
    #[cfg(feature = "vic3")]
    "treemapchart",
    #[cfg(feature = "vic3")]
    "treemapslice",
    "vbox",
    #[cfg(feature = "vic3")]
    "webwindow",
    "widget",
    "window",
    "zoomarea",
];
