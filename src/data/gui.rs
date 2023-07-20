use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::{Block, BlockItem, Field, BV};
use crate::context::ScopeContext;
use crate::data::localization::LocaValue;
use crate::datatype::{validate_datatypes, Datatype};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::parse::localization::ValueParser;
use crate::pdxfile::PdxFile;
use crate::report::{error, error_info, old_warn, warn, warn_info, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Gui {
    files: FnvHashMap<PathBuf, Vec<GuiWidget>>,
    templates: FnvHashMap<String, GuiTemplate>,
    types: FnvHashMap<String, GuiType>,
    layers: FnvHashMap<String, GuiLayer>,
}

impl Gui {
    fn load_widget(&mut self, filename: PathBuf, key: Token, block: Block) {
        if let Some(guifile) = self.files.get_mut(&filename) {
            guifile.push(GuiWidget::new(key, block));
        } else {
            self.files.insert(filename, vec![GuiWidget::new(key, block)]);
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
        if let Some(other) = self.types.get(key.as_str()) {
            dup_error(&key, &other.key, "gui type");
        }
        self.types.insert(key.to_string(), GuiType::new(key, base, block));
    }

    pub fn load_template(&mut self, key: Token, block: Block) {
        if let Some(other) = self.templates.get(key.as_str()) {
            dup_error(&key, &other.key, "gui template");
        }
        self.templates.insert(key.to_string(), GuiTemplate::new(key, block));
    }

    pub fn load_layer(&mut self, key: Token, block: Block) {
        if let Some(other) = self.layers.get(key.as_str()) {
            dup_error(&key, &other.key, "gui layer");
        }
        self.layers.insert(key.to_string(), GuiLayer::new(key, block));
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
                        if let Some((key, block)) = field.expect_into_definition() {
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
    #[allow(dead_code)]
    key: Token,
    block: Block,
}

impl GuiWidget {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        validate_gui(&self.block, data);
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
    #[allow(dead_code)] // TODO
    base: Token,
    block: Block,
}

impl GuiType {
    pub fn new(key: Token, base: Token, block: Block) -> Self {
        Self { key, base, block }
    }

    pub fn validate(&self, data: &Everything) {
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
        // A reference to a game concept
        #[cfg(feature = "ck3")]
        LocaValue::Code(chain, Some(fmt))
            if fmt.as_str().contains('E') || fmt.as_str().contains('e') =>
        {
            if let Some(name) = chain.as_gameconcept() {
                data.verify_exists(Item::GameConcept, name);
            } else {
                let msg = format!("cannot figure out game concept for this |{fmt}");
                warn(ErrorKey::ParseError).weak().msg(msg).loc(fmt).push();
            }
        }
        LocaValue::Code(chain, _) => {
            // TODO: figure out the actual scope context here. Perhaps it should be a strict scope with no root and no names defined?
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), key);
            sc.set_strict_scopes(false);
            validate_datatypes(&chain, data, &mut sc, Datatype::Unknown, "", false);
        }
        _ => (),
    }
}
