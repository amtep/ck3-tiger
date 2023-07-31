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
use crate::helpers::dup_error;
use crate::helpers::stringify_choices;
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
                vd.req_tokens_numbers_exactly(2);
            }
        },
        GuiValidation::CVector2i => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector2i, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_integers_exactly(2);
            }
        },
        GuiValidation::CVector3f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector3f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(3);
            }
        },
        GuiValidation::CVector4f => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector4f, key, bv, data, false);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(4);
            }
        },
        GuiValidation::Color => match bv {
            BV::Value(_) => {
                validate_datatype_field(Datatype::CVector4f, key, bv, data, false);
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
        GuiValidation::SoundBlock => {
            if let Some(block) = bv.expect_block() {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                if let Some(value) = vd.field_value("soundeffect") {
                    if value.starts_with("[") {
                        // TODO: need a way to express "stringable" datatype
                        validate_datatype_field(Datatype::Unknown, key, bv, data, false);
                    } else {
                        data.verify_exists(Item::Sound, value);
                    }
                }
                vd.field_block("soundparam"); // TODO
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
    for (name, validation) in GUI_FIELDS {
        if key_lc == *name {
            validate_known_field(key, bv, data, *validation);
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
                    // TODO: validate fmt
                    LocaValue::Code(chain, _) => {
                        validate_datatypes(chain, data, &mut sc, dtype, "", allow_promote);
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

            let mut sc = ScopeContext::new(Scopes::None, key);
            validate_datatypes(&chain, data, &mut sc, Datatype::Unknown, "", false);
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
    /// A block containing a `soundeffect` field and optionally a `soundparam` block
    SoundBlock,
    /// Some text. May contain datatypes or loca keys, but it's optional.
    RawText,
    /// Some text. May contain datatypes or loca keys.
    Text,
}

const GUI_FIELDS: &[(&str, GuiValidation)] = &[
    ("accept_tabs", Boolean),
    ("addcolumn", NumberOrPercent),
    ("addrow", NumberOrPercent),
    ("align", Align),
    ("allow_outside", Boolean),
    ("alpha", Number),
    ("alwaystransparent", Boolean),
    ("animation_speed", CVector2f),
    ("autoresize", Boolean),
    ("autoresize_slider", Boolean),
    ("autoresizescrollarea", Boolean),
    ("autoresizeviewport", Boolean),
    ("background_texture", Item(Item::File)),
    ("bezier", CVector4f),
    ("blend_mode", Blendmode),
    ("button_ignore", MouseButton(&["both", "none", "left", "right"])),
    ("button_trigger", UncheckedValue), // only example is "none"
    ("camera_fov_y_degrees", Integer),
    ("camera_look_at", CVector3f),
    ("camera_near_far", CVector2f),
    ("camera_position", CVector3f),
    ("camera_rotation_pitch_limits", CVector2f),
    ("camera_translation_limits", CVector3f),
    ("camera_zoom_limits", CVector2f),
    ("checked", Boolean),
    ("clicksound", ItemOrBlank(Item::Sound)),
    ("coat_of_arms", Item(Item::File)),
    ("coat_of_arms_mask", Item(Item::File)),
    ("coat_of_arms_slot", CVector4f),
    ("color", Color),
    ("constantbuffers", Datatype),
    ("cursorcolor", Color),
    ("datacontext", Datacontext),
    ("datamodel", Datamodel),
    ("datamodel_reuse_widgets", Boolean),
    ("datamodel_wrap", Integer),
    ("dec_button", Widget),
    ("default_clicksound", ItemOrBlank(Item::Sound)),
    ("default_format", Format),
    ("delay", Number),
    ("direction", Choice(&["horizontal", "vertical"])),
    ("disableframe", Integer),
    ("distribute_visual_state", Boolean),
    ("down", Boolean),
    ("downframe", Integer),
    ("downhoverframe", Integer),
    ("downpressedframe", Integer),
    ("drag_drop_args", CString),
    ("drag_drop_base_type", Choice(&["icon", "coat_of_arms_icon"])),
    ("drag_drop_data", Datacontext),
    ("drag_drop_id", UncheckedValue), // TODO what are the options?
    ("draggable_by", MouseButtonSet(&["left", "right", "middle"])),
    ("droptarget", Boolean),
    ("duration", Number),
    ("effect", Datatype),
    ("effectname", UncheckedValue), // TODO validate effect names
    ("elide", Choice(&["right", "middle", "left"])),
    ("enabled", Boolean),
    ("end_sound", SoundBlock),
    ("entity_instance", Item(Item::Entity)),
    ("even_row_widget", Widget),
    ("filter_mouse", MouseButtonSet(&["all", "none", "left", "right", "wheel"])),
    ("fittype", Choice(&["center", "centercrop", "fill", "end", "start"])),
    ("flipdirection", Boolean),
    ("focus_on_visible", Boolean),
    ("focuspolicy", Choice(&["click", "all", "none"])),
    ("format_override", FormatOverride),
    ("font", Item(Item::Font)),
    ("fontcolor", Color),
    ("fontsize", Integer),
    ("fontsize_min", Integer),
    ("fontweight", UncheckedValue), // TODO: what are the options?
    ("frame", Integer),
    ("framesize", CVector2i),
    ("from", CVector2f),
    ("fonttintcolor", Color),
    ("gfx_environment_file", Item(Item::File)),
    ("gfxtype", UncheckedValue), // TODO: what are the options?
    ("glow_alpha", Number),
    ("glow_alpha_mask", Integer),
    ("glow_blur_passes", Integer),
    ("glow_ignore_inside_pixels", Boolean),
    ("glow_radius", Integer),
    ("glow_texture_downscale", NumberF),
    ("grayscale", Boolean),
    ("grid_entity_name", Item(Item::Entity)),
    ("highlightchecked", Boolean),
    ("header_height", Integer),
    ("ignore_in_debug_draw", Boolean),
    ("ignoreinvisible", Boolean),
    ("inc_button", Widget),
    ("indent", Integer),
    ("index", Integer),
    ("inherit_data_context", Boolean),
    ("inherit_visibility", Choice(&["yes", "no", "hidden"])),
    ("inherit_visual_state", Boolean),
    ("invert_reticule_color", Boolean),
    ("invertprogress", Boolean),
    ("intersectionmask", Boolean),
    ("intersectionmask_texture", Item(Item::File)),
    ("layer", Item(Item::GuiLayer)),
    ("layoutanchor", UncheckedValue), // TODO: only example is "bottomleft"
    ("layoutpolicy_horizontal", ChoiceSet(LAYOUT_POLICIES)),
    ("layoutpolicy_vertical", ChoiceSet(LAYOUT_POLICIES)),
    ("layoutstretchfactor_horizontal", NumberOrInt32),
    ("layoutstretchfactor_vertical", NumberOrInt32),
    ("line_cap", Boolean),
    ("line_feather_distance", Integer),
    ("line_type", UncheckedValue), // TODO: only example is "nodeline"
    ("loop", Boolean),
    ("loopinterval", Number),
    ("margin", TwoNumberOrPercent),
    ("margin_bottom", NumberOrInt32),
    ("margin_left", NumberOrInt32),
    ("margin_right", NumberOrInt32),
    ("margin_top", NumberOrInt32),
    ("mask", Item(Item::File)),
    ("mask_uv_scale", CVector2f),
    ("max", NumberOrInt32),
    ("max_update_rate", Integer),
    ("max_width", Integer),
    ("maxcharacters", UnsignedInteger),
    ("maxhorizontalslots", Integer),
    ("maxverticalslots", Integer),
    ("maximumsize", TwoNumberOrPercent),
    ("min", NumberOrInt32),
    ("min_dist_from_screen_edge", Integer),
    ("min_width", Integer),
    ("minimumsize", TwoNumberOrPercent),
    ("mipmaplodbias", Integer),
    ("mirror", Choice(&["horizontal", "vertical"])),
    ("modal", Boolean),
    ("modality", UncheckedValue), // TODO: only example is "all"
    ("movable", Boolean),
    ("multiline", Boolean),
    ("name", UncheckedValue),
    ("next", UncheckedValue), // TODO: choices are states in the same widget
    ("noprogresstexture", Item(Item::File)),
    ("odd_row_widget", Widget),
    ("on_finish", Datatype),
    ("on_keyframe_move", Datatype),
    ("on_start", Datatype),
    ("onchangefinish", Datatype),
    ("onchangestart", Datatype),
    ("onclick", Datatype),
    ("oncolorchanged", Datatype),
    ("oncoloredited", Datatype),
    ("oncreate", Datatype),
    ("ondefault", Datatype),
    ("ondoubleclick", Datatype),
    ("oneditingstart", Datatype),
    ("oneditingfinished", Datatype),
    ("oneditingfinished_with_changes", Datatype),
    ("onfocusout", Datatype),
    ("onmousehierarchyenter", Datatype),
    ("onmousehierarchyleave", Datatype),
    ("onpressed", Datatype),
    ("onreleased", Datatype),
    ("onreturnpressed", Datatype),
    ("onrightclick", Datatype),
    ("onselectionchanged", Datatype),
    ("onshift", Datatype),
    ("ontextchanged", Datatype),
    ("ontextedited", Datatype),
    ("onvaluechanged", Datatype),
    ("overframe", Integer),
    ("oversound", ItemOrBlank(Item::Sound)),
    ("page", Integer),
    ("pan_position", CVector2f),
    ("parentanchor", Align),
    ("password", Boolean),
    ("points", Datatype),
    ("pop_out", Boolean),
    ("portrait_context", Datatype),
    ("portrait_offset", CVector2f),
    ("portrait_scale", CVector2f),
    ("portrait_texture", Item(Item::File)),
    ("position", TwoNumberOrPercent),
    ("position_x", Integer),
    ("position_y", Integer),
    ("progress_change_to_duration_curve", CVector4f),
    ("progresstexture", Item(Item::File)),
    ("pseudo_localization_enabled", Boolean),
    ("raw_text", RawText),
    ("raw_tooltip", RawText),
    ("reorder_on_mouse", UncheckedValue), // TODO: only example is "presstop"
    ("recursive", Boolean),
    ("resizable", Boolean),
    ("resizeparent", Boolean),
    ("restart_on_show", Boolean),
    ("restrictparent_min", Boolean),
    ("reuse_widgets", Boolean),
    ("righttoleft", Boolean),
    ("rotate_uv", Number),
    ("row_height", Integer),
    ("scale", Number),
    ("scale_mode", UncheckedValue), // TODO: only example is "fixedwidth"
    ("scissor", Boolean),
    ("scrollbaralign_horizontal", Align),
    ("scrollbaralign_vertical", Align),
    // TODO: always_on is a guess
    ("scrollbarpolicy_horizontal", Choice(&["as_needed", "always_off", "always_on"])),
    ("scrollbarpolicy_vertical", Choice(&["as_needed", "always_off", "always_on"])),
    ("selectedindex", CVector2i),
    ("selectallonfocus", Boolean),
    ("selectioncolor", Color),
    ("set_parent_size_to_minimum", Boolean),
    ("setitemsizefromcell", Boolean),
    ("shaderfile", ItemOrBlank(Item::File)),
    ("shortcut", Item(Item::Shortcut)),
    ("size", TwoNumberOrPercent),
    ("slider", Widget),
    ("snap_to_pixels", Boolean),
    ("spacing", Integer),
    ("spriteborder", CVector2f),
    ("spriteborder_bottom", Integer),
    ("spriteborder_left", Integer),
    ("spriteborder_right", Integer),
    ("spriteborder_top", Integer),
    ("spritetype", UncheckedValue), // TODO
    ("start_sound", SoundBlock),
    ("step", NumberOrInt32),
    ("stackmode", UncheckedValue), // TODO only example is "top"
    ("sticky", Boolean),
    ("tabfocusroot", Boolean),
    ("text", Text),
    ("text_selectable", Boolean),
    ("text_validator", Datatype),
    ("texture", Item(Item::File)),
    ("texture_density", Number),
    ("timeline_line_height", Integer),
    ("timeline_line_direction", UncheckedValue), // TODO only example is "up"
    ("timeline_time_points", Integer),
    ("tintcolor", Color),
    ("to", CVector2f),
    ("tooltip", Text),
    ("tooltip_enabled", Boolean),
    ("tooltip_horizontalbehavior", Choice(&["mirror", "slide", "flip"])),
    ("tooltip_verticalbehavior", Choice(&["mirror", "slide", "flip"])),
    ("tooltip_offset", TwoNumberOrPercent),
    ("tooltip_parentanchor", Align),
    ("tooltip_type", Choice(&["mouse", "widget"])),
    ("tooltip_widgetanchor", Align),
    ("tooltipwidget", Widget),
    ("track", Widget),
    ("tracknavigation", UncheckedValue), // TODO only example is "direct"
    ("translate_uv", CVector2f),
    ("trigger_on_create", Boolean),
    ("trigger_when", Boolean),
    ("upframe", Integer),
    ("uphoverframe", Integer),
    ("uppressedframe", Integer),
    ("using", Template),
    ("uv_scale", CVector2f),
    ("value", NumberOrInt32),
    ("video", Item(Item::File)),
    ("viewportwidget", Widget),
    ("visible", Boolean),
    ("visible_at_creation", Boolean),
    ("wheelstep", NumberOrInt32),
    ("widgetanchor", Align),
    ("widgetid", UncheckedValue),
    ("width", Number),
    ("zoom", Number),
    ("zoom_max", Number),
    ("zoom_min", Number),
    ("zoom_step", Number),
    ("zoomwidget", Widget),
];

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
