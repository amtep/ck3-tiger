use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::data::localization::LocaValue;
use crate::datatype::{validate_datatypes, Datatype};
use crate::errorkey::ErrorKey;
use crate::errors::{advice, error, error_info, warn, warn_info};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::parse::localization::ValueParser;
use crate::pdxfile::PdxFile;
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
            self.files
                .insert(filename, vec![GuiWidget::new(key, block)]);
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
        for (k, cmp, bv) in block.iter_items() {
            match stage {
                Expecting::Type => {
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        let info = "did you forget the `type` keyword?";
                        warn_info(key, ErrorKey::ParseError, &msg, info);
                    } else if let Some(token) = bv.expect_value() {
                        if token.is("type") {
                            stage = Expecting::Header;
                        } else {
                            let msg = format!("unexpected token `{token}`");
                            error(token, ErrorKey::ParseError, &msg);
                        }
                    }
                }
                Expecting::Header => {
                    if let Some(key) = k {
                        if let Some(token) = bv.expect_value() {
                            stage = Expecting::Body(key, token);
                        } else {
                            stage = Expecting::Type;
                        }
                    } else {
                        error(bv, ErrorKey::ParseError, "expected type header");
                        stage = Expecting::Type;
                    }
                }
                Expecting::Body(name, base) => {
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        error(key, ErrorKey::ParseError, &msg);
                    } else if let Some(block) = bv.expect_block() {
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
        self.types
            .insert(key.to_string(), GuiType::new(key, base, block));
    }

    pub fn load_template(&mut self, key: Token, block: Block) {
        if let Some(other) = self.templates.get(key.as_str()) {
            dup_error(&key, &other.key, "gui template");
        }
        self.templates
            .insert(key.to_string(), GuiTemplate::new(key, block));
    }

    pub fn load_layer(&mut self, key: Token, block: Block) {
        if let Some(other) = self.layers.get(key.as_str()) {
            dup_error(&key, &other.key, "gui layer");
        }
        self.layers
            .insert(key.to_string(), GuiLayer::new(key, block));
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
    }
}

impl FileHandler for Gui {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("gui")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        #[derive(Copy, Clone, Debug)]
        enum Expecting<'a> {
            Widget,
            Types,
            TypesBody,
            Template,
            TemplateBody(&'a Token),
            Layer,
            LayerBody(&'a Token),
        }

        if !entry.filename().to_string_lossy().ends_with(".gui") {
            return;
        }

        let Some(block) = PdxFile::read_optional_bom(entry, fullpath) else { return };

        let mut expecting = Expecting::Widget;

        for (k, cmp, bv) in block.iter_items() {
            match expecting {
                Expecting::Widget => {
                    if let Some(key) = k {
                        if let Some(block) = bv.expect_block() {
                            self.load_widget(
                                PathBuf::from(entry.filename()),
                                key.clone(),
                                block.clone(),
                            );
                        }
                    } else if let Some(token) = bv.expect_value() {
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
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        let info = format!("After `Types {key}` there shouldn't be an `{cmp}`");
                        error_info(key, ErrorKey::ParseError, &msg, &info);
                        expecting = Expecting::Widget;
                    } else if let Some(_token) = bv.expect_value() {
                        expecting = Expecting::TypesBody;
                    } else {
                        expecting = Expecting::Widget;
                    }
                }
                Expecting::TypesBody => {
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        error(key, ErrorKey::ParseError, &msg);
                    } else if let Some(block) = bv.expect_block() {
                        self.load_types(block);
                    }
                    expecting = Expecting::Widget;
                }
                Expecting::Template => {
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        let info = format!("After `template {key}` there shouldn't be an `{cmp}`");
                        error_info(key, ErrorKey::ParseError, &msg, &info);
                        expecting = Expecting::Widget;
                    } else if let Some(token) = bv.expect_value() {
                        expecting = Expecting::TemplateBody(token);
                    } else {
                        expecting = Expecting::Widget;
                    }
                }
                Expecting::TemplateBody(token) => {
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        error(key, ErrorKey::ParseError, &msg);
                    } else if let Some(block) = bv.expect_block() {
                        self.load_template(token.clone(), block.clone());
                    }
                    expecting = Expecting::Widget;
                }
                Expecting::Layer => {
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        let info = format!("After `layer {key}` there shouldn't be an `{cmp}`");
                        error_info(key, ErrorKey::ParseError, &msg, &info);
                        expecting = Expecting::Widget;
                    } else if let Some(token) = bv.expect_value() {
                        expecting = Expecting::LayerBody(token);
                    } else {
                        expecting = Expecting::Widget;
                    }
                }
                Expecting::LayerBody(token) => {
                    if let Some(key) = k {
                        let msg = format!("unexpected assignment `{key} {cmp}`");
                        error(key, ErrorKey::ParseError, &msg);
                    } else if let Some(block) = bv.expect_block() {
                        self.load_layer(token.clone(), block.clone());
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
        validate_gui(&self.block, data);
    }
}

#[derive(Clone, Debug)]
struct GuiTemplate {
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
    key: Token,
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
    for (k, cmp, bv) in block.iter_items() {
        match expecting {
            Expecting::Field => {
                if let Some(key) = k {
                    if key.is("block") {
                        let msg = format!("`{key}` should not be followed by an `{cmp}`");
                        advice(key, ErrorKey::ParseError, &msg);
                        bv.expect_value();
                        expecting = Expecting::SubstBlockBody;
                    } else if key.is("blockoverride") {
                        let msg = format!("`{key}` should not be followed by an `{cmp}`");
                        advice(key, ErrorKey::ParseError, &msg);
                        bv.expect_value();
                        expecting = Expecting::BlockOverrideBody;
                    } else {
                        validate_field(key, bv, data);
                    }
                } else if let Some(token) = bv.expect_value() {
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
                if let Some(key) = k {
                    let msg = format!("unexpected assignment `{key} {cmp}`");
                    error(key, ErrorKey::ParseError, &msg);
                    expecting = Expecting::Field;
                } else if let Some(_token) = bv.expect_value() {
                    expecting = Expecting::SubstBlockBody;
                } else {
                    expecting = Expecting::Field;
                }
            }
            Expecting::SubstBlockBody => {
                if let Some(key) = k {
                    let msg = format!("unexpected assignment `{key} {cmp}`");
                    error(key, ErrorKey::ParseError, &msg);
                } else if let Some(block) = bv.expect_block() {
                    validate_gui(block, data);
                }
                expecting = Expecting::Field;
            }
            Expecting::BlockOverride => {
                if let Some(key) = k {
                    let msg = format!("unexpected assignment `{key} {cmp}`");
                    error(key, ErrorKey::ParseError, &msg);
                    expecting = Expecting::Field;
                } else if let Some(_token) = bv.expect_value() {
                    expecting = Expecting::BlockOverrideBody;
                } else {
                    expecting = Expecting::Field;
                }
            }
            Expecting::BlockOverrideBody => {
                if let Some(key) = k {
                    let msg = format!("unexpected assignment `{key} {cmp}`");
                    error(key, ErrorKey::ParseError, &msg);
                } else if let Some(block) = bv.expect_block() {
                    validate_gui(block, data);
                }
                expecting = Expecting::Field;
            }
        }
    }
}

fn validate_field(key: &Token, bv: &BV, data: &Everything) {
    if key.is("default_format") {
        bv.expect_value();
        return;
    } else if key.is("texture") {
        if let Some(token) = bv.expect_value() {
            // The editor_gui ones aren't in the CK3 installation but do appear
            // to be available.
            if !token.starts_with("[") && !token.starts_with("gfx/editor_gui/") {
                data.verify_exists(Item::File, token);
                return;
            }
        }
    } else if key.is("tooltip") || key.is("text") {
        if let Some(token) = bv.expect_value() {
            // The JOMINI_MULTIPLAYER_ ones are probably built in.
            if !token.starts_with("[") && !token.starts_with("JOMINI_MULTIPLAYER_") {
                data.verify_exists(Item::Localization, token);
                return;
            }
        }
    } else if key.is("raw_text") {
        if let Some(token) = bv.expect_value() {
            if !token.starts_with("[") {
                // Some raw_text fields do contain loca keys
                data.item_used(Item::Localization, token.as_str());
                return;
            }
        }
    }
    match bv {
        BV::Value(token) => {
            let mut valuevec = ValueParser::new(vec![token]).parse_value();
            let value = if valuevec.len() == 1 {
                std::mem::take(&mut valuevec[0])
            } else {
                LocaValue::Concat(valuevec)
            };
            if key.is("datacontext") {
                if let LocaValue::Code(chain, _) = value {
                    validate_datatypes(&chain, data, Datatype::Unknown, "", true);
                } else {
                    let msg = "expected whole field to be a single [ ] clause";
                    warn(token, ErrorKey::Validation, msg);
                }
            } else {
                validate_gui_loca(value, data);
            }
        }
        BV::Block(block) => {
            validate_gui(block, data);
        }
    }
}

fn validate_gui_loca(value: LocaValue, data: &Everything) {
    match value {
        LocaValue::Concat(v) => {
            for value in v {
                validate_gui_loca(value, data);
            }
        }
        LocaValue::Code(chain, _) => {
            validate_datatypes(&chain, data, Datatype::Unknown, "", false);
        }
        _ => (),
    }
}
