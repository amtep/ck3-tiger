use std::mem::take;
use std::path::PathBuf;

use crate::block::Block;
use crate::data::localization::LocaValue;
use crate::datatype::{Code, CodeArg, CodeChain};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TigerHashMap};
use crate::parse::localization::ValueParser;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug, Default)]
pub struct DataBindings {
    bindings: TigerHashMap<&'static str, DataBinding>,
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
            warn(ErrorKey::ParseError).msg("missing field `definition`").loc(block).push();
            return;
        }
        if let Some(other) = self.bindings.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(&key, &other.key, "data binding");
            }
        }
        self.bindings.insert(key.as_str(), DataBinding::new(key, block));
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

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            if key.is("macro") {
                self.load_macro(block);
            } else {
                let msg = format!("unexpected key {key} in data_binding");
                warn(ErrorKey::ParseError).msg(msg).loc(key).push();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataBinding {
    key: Token,
    block: Block,
    params: Vec<Token>,
    body: Option<CodeChain>,
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
        let mut body = None;
        if let Some(rep) = block.get_field_value("replace_with") {
            // TODO: restructure ValueParser to have a separate DatafunctionParser,
            // so that we don't have to synthesize these brackets.
            let open_bracket = Token::from_static_str("[", rep.loc);
            let close_bracket = Token::from_static_str("]", rep.loc);
            let to_parse = vec![&open_bracket, rep, &close_bracket];
            let value = ValueParser::new(to_parse).parse();
            if let LocaValue::Code(mut chain, _) = value {
                body = Some(take(&mut chain));
            } else {
                let msg = "could not parse macro replacement";
                err(ErrorKey::Datafunctions).msg(msg).loc(rep).push();
            }
        }
        Self { key, block, params, body }
    }

    pub fn replace(&self, call: &Code) -> Option<CodeChain> {
        if call.arguments.len() != self.params.len() {
            let msg = "wrong number of arguments for macro";
            err(ErrorKey::Datafunctions).msg(msg).loc(&call.name).push();
            return None;
        }
        match self.replace_chain(self.body.as_ref()?, call)? {
            CodeArg::Chain(chain) => Some(chain),
            CodeArg::Literal(token) => {
                let msg = "cannot substitute a literal here";
                err(ErrorKey::Datafunctions).msg(msg).loc(token).push();
                None
            }
        }
    }

    fn replace_chain(&self, body_chain: &CodeChain, call: &Code) -> Option<CodeArg> {
        let mut result = Vec::new();
        for body_code in &body_chain.codes {
            if body_code.arguments.is_empty() {
                // Check if body_code is a macro parameter.
                // Note: self.params and call.arguments have already been checked to be the same length, so
                // using the same index on both is ok.
                if let Some(idx) = self.params.iter().position(|param| param == &body_code.name) {
                    match &call.arguments[idx] {
                        CodeArg::Literal(caller_token) => {
                            // A literal can't be part of a chain, so accept it only if the "chain"
                            // is only one part.
                            if body_chain.codes.len() != 1 {
                                let msg = "cannot substitute a literal here";
                                err(ErrorKey::Datafunctions).msg(msg).loc(caller_token).push();
                                return None;
                            }
                            return Some(call.arguments[idx].clone());
                        }
                        CodeArg::Chain(caller_chain) => {
                            result.extend_from_slice(&caller_chain.codes);
                        }
                    }
                } else {
                    result.push(body_code.clone());
                }
            } else {
                result.push(Code {
                    name: body_code.name.clone(),
                    arguments: body_code
                        .arguments
                        .iter()
                        .map(|arg| self.replace_arg(arg, call))
                        .collect::<Option<Vec<_>>>()?,
                });
            }
        }
        Some(CodeArg::Chain(CodeChain { codes: result }))
    }

    fn replace_arg(&self, body_arg: &CodeArg, call: &Code) -> Option<CodeArg> {
        match body_arg {
            CodeArg::Chain(body_chain) => self.replace_chain(body_chain, call),
            CodeArg::Literal(_) => Some(body_arg.clone()),
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
