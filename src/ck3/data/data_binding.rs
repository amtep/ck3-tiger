use std::path::{Path, PathBuf};

use fnv::FnvHashMap;

use crate::block::validator::Validator;
use crate::block::Block;
use crate::data::localization::LocaValue;
use crate::datatype::{Code, CodeArg, CodeChain};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::parse::localization::ValueParser;
use crate::pdxfile::PdxFile;
use crate::report::{error, old_warn, ErrorKey};
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
            old_warn(block, ErrorKey::ParseError, "missing field `definition`");
            return;
        }
        if let Some(other) = self.bindings.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "data binding");
            }
        }
        self.bindings.insert(key.to_string(), DataBinding::new(key, block));
    }

    pub fn get(&self, key: &str) -> Option<&DataBinding> {
        self.bindings.get(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.bindings.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for DataBindings {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("data_binding")
    }

    fn load_file(&self, entry: &FileEntry, fullpath: &Path) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, fullpath)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            if key.is("macro") {
                self.load_macro(block);
            } else {
                let msg = format!("unexpected key {key} in data_binding");
                old_warn(key, ErrorKey::ParseError, &msg);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataBinding {
    key: Token,
    block: Block,
    params: Vec<Token>,
    replace: Option<CodeChain>,
}

impl DataBinding {
    fn new(key: Token, block: Block) -> Self {
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
            let open_bracket = Token::from_static_str("[", rep.loc.clone());
            let close_bracket = Token::from_static_str("]", rep.loc.clone());
            let to_parse = vec![&open_bracket, rep, &close_bracket];
            let valuevec = ValueParser::new(to_parse).parse_value();
            if valuevec.len() == 1 {
                if let LocaValue::Code(chain, _) = &valuevec[0] {
                    replace = Some(chain.clone());
                } else {
                    let msg = "could not parse macro replacement";
                    error(rep, ErrorKey::Datafunctions, msg);
                }
            } else {
                let msg = "could not parse macro replacement";
                error(rep, ErrorKey::Datafunctions, msg);
            }
        }
        Self { key, block, params, replace }
    }

    pub fn replace(&self, call: &Code) -> Option<CodeChain> {
        if call.arguments.len() != self.params.len() {
            let msg = "wrong number of arguments for macro";
            error(&call.name, ErrorKey::Datafunctions, msg);
            return None;
        }
        if let Some(replacement) = &self.replace {
            let mut result = CodeChain { codes: Vec::new() };
            for code in &replacement.codes {
                let mut new_code = Code { name: code.name.clone(), arguments: Vec::new() };
                for arg in &code.arguments {
                    if let Some(replacement) = self.replace_param(arg, call) {
                        new_code.arguments.push(replacement);
                    } else {
                        return None;
                    }
                }
                result.codes.push(new_code);
            }
            Some(result)
        } else {
            None
        }
    }

    fn replace_param(&self, arg: &CodeArg, call: &Code) -> Option<CodeArg> {
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
                        let mut new_code = Code { name: code.name.clone(), arguments: Vec::new() };
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
            CodeArg::Literal(_) => Some(arg.clone()),
        }
    }

    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);
        vd.req_field("replace_with");
        vd.field_value("description");
        vd.field_value("definition");
        vd.field_value("replace_with");
    }
}
