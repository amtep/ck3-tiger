use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::Block;
use crate::data::localization::parse::ValueParser;
use crate::data::localization::LocaValue;
use crate::datatype::{Code, CodeArg, CodeChain};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct DataBindings {
    bindings: FnvHashMap<String, DataBinding>,
}

impl DataBindings {
    fn load_macro(&mut self, block: Block) {
        let key;
        if let Some(def) = block.get_field_value("definition") {
            if let Some((splitdef, _)) = def.split_once('(') {
                key = splitdef;
            } else {
                key = def.clone();
            }
        } else {
            warn(block, ErrorKey::ParseError, "missing field `definition`");
            return;
        }
        if let Some(other) = self.bindings.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "data binding");
            }
        }
        self.bindings
            .insert(key.to_string(), DataBinding::new(key, block));
    }

    pub fn get(&self, key: &str) -> Option<&DataBinding> {
        self.bindings.get(key)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.bindings.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.bindings.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for DataBindings {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("data_binding")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, b) in block.iter_pure_definitions_warn() {
            if key.is("macro") {
                self.load_macro(b.clone());
            } else {
                let msg = format!("unexpected key {key} in data_binding");
                warn(key, ErrorKey::ParseError, &msg);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataBinding {
    key: Token,
    block: Block,
    params: Vec<Token>,
    replace: Option<Code>,
}

impl DataBinding {
    pub fn new(key: Token, block: Block) -> Self {
        let mut params = Vec::new();
        if let Some(def) = block.get_field_value("definition") {
            if let Some((_, paramsx)) = def.split_once('(') {
                if let Some((arguments, _)) = paramsx.split_once(')') {
                    for param in arguments.split(',') {
                        params.push(param);
                    }
                }
            }
        }
        let mut replace = None;
        if let Some(rep) = block.get_field_value("replace_with") {
            // TODO: restructure ValueParser to have a separate DatafunctionParser,
            // so that we don't have to synthesize these brackets.
            let open_bracket = Token::new("[".to_string(), rep.loc.clone());
            let close_bracket = Token::new("]".to_string(), rep.loc.clone());
            let to_parse = vec![&open_bracket, rep, &close_bracket];
            let valuevec = ValueParser::new(to_parse).parse_value();
            if valuevec.len() == 1 {
                if let LocaValue::Code(code, _) = &valuevec[0] {
                    if code.codes.len() == 1 {
                        replace = Some(code.codes[0].clone());
                    } else {
                        let msg = "macro replacement should be a single call";
                        error(rep, ErrorKey::Datafunctions, msg);
                    }
                } else {
                    let msg = "could not parse macro replacement";
                    error(rep, ErrorKey::Datafunctions, msg);
                }
            } else {
                let msg = "could not parse macro replacement";
                error(rep, ErrorKey::Datafunctions, msg);
            }
        }
        Self {
            key,
            block,
            params,
            replace,
        }
    }

    pub fn replace(&self, call: &Code) -> Option<Code> {
        if let Some(rep) = &self.replace {
            if call.arguments.len() != self.params.len() {
                let msg = "wrong number of arguments for macro";
                warn(&call.name, ErrorKey::Datafunctions, msg);
                return None;
            }
            let mut result = Code {
                name: rep.name.clone(),
                arguments: Vec::new(),
            };
            for arg in &rep.arguments {
                if let Some(replacement) = self.replace_param(arg, call) {
                    result.arguments.push(replacement);
                } else {
                    return None;
                }
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn replace_param(&self, arg: &CodeArg, call: &Code) -> Option<CodeArg> {
        match arg {
            CodeArg::Chain(chain) => {
                let mut result = CodeChain { codes: Vec::new() };
                for code in &chain.codes {
                    if code.arguments.is_empty() {
                        let mut found = false;
                        for (i, param) in self.params.iter().enumerate() {
                            if param.is(code.name.as_str()) {
                                found = true;
                                match &call.arguments[i] {
                                    CodeArg::Literal(token) => {
                                        if chain.codes.len() != 1 {
                                            let msg = "cannot substitute a literal here";
                                            error(token, ErrorKey::Datafunctions, msg);
                                            return None;
                                        }
                                        return Some(call.arguments[i].clone());
                                    }
                                    CodeArg::Chain(caller_chain) => {
                                        for caller_code in &caller_chain.codes {
                                            result.codes.push(caller_code.clone());
                                        }
                                    }
                                }
                                break;
                            }
                        }
                        if !found {
                            result.codes.push(code.clone());
                        }
                    } else {
                        let mut new_code = Code {
                            name: code.name.clone(),
                            arguments: Vec::new(),
                        };
                        for arg in &code.arguments {
                            if let Some(rep) = self.replace_param(arg, call) {
                                new_code.arguments.push(rep);
                            } else {
                                return None;
                            }
                        }
                        result.codes.push(new_code);
                    }
                }
                Some(CodeArg::Chain(result))
            }
            _ => Some(arg.clone()),
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.req_field("replace_with");
        vd.field_value("description");
        vd.field_value("definition");
        vd.field_value("replace_with");
    }
}
