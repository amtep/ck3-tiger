//! A registry of all the script variables that have been defined somewhere.

use crate::block::{Block, Comparator, Eq::Single, Field, BV};
use crate::game::{Game, GameFlags};
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::report::{report, ErrorKey, Severity};
use crate::token::Token;

#[derive(Debug)]
pub struct Variables {
    names: TigerHashSet<&'static str>,
    // For hoi4: variables that were defined with @something at the end, here without the @ part.
    name_prefixes: TigerHashSet<&'static str>,
    // For hoi4: variables that look like they have a country tag at the end, without the country tag.
    name_speculative_prefixes: TigerHashSet<&'static str>,
    lists: TigerHashSet<&'static str>,
    list_prefixes: TigerHashSet<&'static str>,
    list_speculative_prefixes: TigerHashSet<&'static str>,

    // effect names to look for, mapped to the field inside them that contains the name.
    create_variable: TigerHashMap<&'static str, Extract>,
    create_list: TigerHashMap<&'static str, Extract>,
}

impl Variables {
    pub fn new() -> Self {
        Self {
            names: TigerHashSet::default(),
            name_prefixes: TigerHashSet::default(),
            name_speculative_prefixes: TigerHashSet::default(),
            lists: TigerHashSet::default(),
            list_prefixes: TigerHashSet::default(),
            list_speculative_prefixes: TigerHashSet::default(),
            create_variable: filter_table(CREATE_VARIABLE),
            create_list: filter_table(CREATE_LIST),
        }
    }

    pub fn register_variable(&mut self, name: &'static str) {
        if Game::is_hoi4() {
            if let Some((prefix, _)) = name.split_once('@') {
                self.name_prefixes.insert(remove_qualifiers(prefix));
            } else {
                let name = remove_qualifiers(name);
                if let Some(prefix) = remove_suffix_tag(name) {
                    self.name_speculative_prefixes.insert(prefix);
                }
                self.names.insert(name);
            }
        } else {
            self.names.insert(name);
        }
    }

    /// Just like `register_variable` but for lists.
    pub fn register_list(&mut self, name: &'static str) {
        if Game::is_hoi4() {
            if let Some((prefix, _)) = name.split_once('@') {
                self.list_prefixes.insert(remove_qualifiers(prefix));
            } else {
                let name = remove_qualifiers(name);
                if let Some(prefix) = remove_suffix_tag(name) {
                    self.list_speculative_prefixes.insert(prefix);
                }
                self.lists.insert(name);
            }
        } else {
            self.lists.insert(name);
        }
    }
    /// Recursively scan a block for effects that set a variable.
    pub fn scan(&mut self, block: &Block) {
        for Field(key, cmp, bv) in block.iter_fields() {
            if !matches!(cmp, Comparator::Equals(Single)) {
                continue;
            }
            if let Some(extract) = self.create_variable.get(key.as_str()) {
                if let Some(name) = extract.extract(bv) {
                    self.register_variable(name);
                }
            } else if let Some(extract) = self.create_list.get(key.as_str()) {
                if let Some(name) = extract.extract(bv) {
                    self.register_list(name);
                }
            } else if let Some(block) = bv.get_block() {
                self.scan(block);
            }
        }
    }

    /// Check if a variable name has been previously registered.
    /// This takes a bare variable name that did not have an `@` suffix (in Hoi4).
    #[allow(dead_code)]
    pub fn verify_variable_exists(&self, name: &Token, sev: Severity) {
        if let Some(prefix) = remove_suffix_tag(name.as_str()) {
            if !self.name_prefixes.contains(prefix) && !self.names.contains(name.as_str()) {
                let msg = format!("variable `{name}` or `{name}@TAG` was not set anywhere");
                report(ErrorKey::Variables, sev).msg(msg).loc(name).push();
            }
        } else if !self.names.contains(name.as_str()) {
            let msg = format!("variable `{name}` was not set anywhere");
            report(ErrorKey::Variables, sev).msg(msg).loc(name).push();
        }
    }

    /// Check if a variable name has been previously registered.
    /// This takes a bare variable name from which an `@` suffix was removed.
    /// This logic is specific to hoi4.
    #[cfg(feature = "hoi4")]
    pub fn verify_variable_prefix_exists(&self, prefix: &Token, sev: Severity) {
        if !self.name_prefixes.contains(prefix.as_str())
            && !self.name_speculative_prefixes.contains(prefix.as_str())
        {
            let msg = format!("a variable with prefix `{prefix}` was not set anywhere");
            report(ErrorKey::Variables, sev).msg(msg).loc(prefix).push();
        }
    }

    /// Check if a variable list name has been previously registered.
    /// This takes a bare name that did not have an `@` suffix (in Hoi4).
    #[allow(dead_code)]
    pub fn verify_list_exists(&self, name: &Token, sev: Severity) {
        let thing = if Game::is_hoi4() { "array" } else { "variable list" };
        if let Some(prefix) = remove_suffix_tag(name.as_str()) {
            if !self.list_prefixes.contains(prefix) && !self.lists.contains(name.as_str()) {
                let msg = format!("{thing} `{name}` or `{name}@TAG` was not created anywhere");
                report(ErrorKey::Variables, sev).msg(msg).loc(name).push();
            }
        } else if !self.lists.contains(name.as_str()) {
            let msg = format!("{thing} `{name}` was not created anywhere");
            report(ErrorKey::Variables, sev).msg(msg).loc(name).push();
        }
    }

    /// Check if a variable list name has been previously registered.
    /// This takes a bare name from which an `@` suffix was removed.
    /// This logic is specific to hoi4.
    /// Hoi4 calls them `arrays` but the function uses `list` for consistency with the other functions.
    #[cfg(feature = "hoi4")]
    pub fn verify_list_prefix_exists(&self, prefix: &Token, sev: Severity) {
        if !self.list_prefixes.contains(prefix.as_str())
            && !self.list_speculative_prefixes.contains(prefix.as_str())
        {
            let msg = format!("an array with prefix `{prefix}` was not set anywhere");
            report(ErrorKey::Variables, sev).msg(msg).loc(prefix).push();
        }
    }
}

/// Create a map tuned for the current game.
fn filter_table(
    table: &[(&'static str, Extract, GameFlags)],
) -> TigerHashMap<&'static str, Extract> {
    let game = GameFlags::game();
    table
        .iter()
        .filter(|(_, _, gameflags)| gameflags.contains(game))
        .map(|(effect, extract, _)| (*effect, *extract))
        .collect()
}

/// Return the variable name with any preceding `FROM.` etc removed.
fn remove_qualifiers(name: &str) -> &str {
    if let Some((_, name)) = name.rsplit_once('.') {
        name
    } else {
        name
    }
}

/// If the variable name has a country tag at the end, return it with that tag removed.
/// Otherwise return `None`.
fn remove_suffix_tag(name: &str) -> Option<&str> {
    (name.len() > 3 && name.chars().rev().take(3).all(|c| c.is_ascii_uppercase()))
        .then(|| &name[..name.len() - 3])
}

#[derive(Debug, Clone, Copy)]
enum Extract {
    Field(&'static str),
    AssignOrField(&'static str),
    InternalAssignOrField(&'static str),
}

impl Extract {
    pub fn extract(&self, bv: &BV) -> Option<&'static str> {
        match self {
            Self::Field(field) => {
                if let Some(block) = bv.get_block() {
                    if let Some(name) = block.get_field_value(field) {
                        return Some(name.as_str());
                    }
                }
            }
            Self::AssignOrField(field) => match bv {
                BV::Value(name) => {
                    return Some(name.as_str());
                }
                BV::Block(block) => {
                    if let Some(name) = block.get_field_value(field) {
                        return Some(name.as_str());
                    }
                }
            },
            Self::InternalAssignOrField(field) => {
                if let Some(block) = bv.get_block() {
                    if let Some(name) = block.get_field_value(field) {
                        return Some(name.as_str());
                    } else if block.num_items() == 1 {
                        if let Some((name, _)) = block.iter_assignments().next() {
                            return Some(name.as_str());
                        }
                    }
                }
            }
        }
        None
    }
}

// TODO: treat local variables and temp variables like named scopes instead.
const CREATE_VARIABLE: &[(&str, Extract, GameFlags)] = &[
    ("set_dead_character_variable", Extract::Field("name"), GameFlags::Ck3),
    ("set_global_variable", Extract::AssignOrField("name"), GameFlags::jomini()),
    ("set_local_variable", Extract::AssignOrField("name"), GameFlags::jomini()),
    ("set_temp_variable", Extract::InternalAssignOrField("var"), GameFlags::Hoi4),
    ("set_temp_variable_to_random", Extract::AssignOrField("var"), GameFlags::Hoi4),
    ("set_variable", Extract::AssignOrField("name"), GameFlags::jomini()),
    ("set_variable", Extract::InternalAssignOrField("var"), GameFlags::Hoi4),
    ("set_variable_to_random", Extract::AssignOrField("var"), GameFlags::Hoi4),
];

const CREATE_LIST: &[(&str, Extract, GameFlags)] = &[
    ("add_to_array", Extract::InternalAssignOrField("array"), GameFlags::Hoi4),
    ("add_to_global_variable_list", Extract::Field("name"), GameFlags::jomini()),
    ("add_to_local_variable_list", Extract::Field("name"), GameFlags::jomini()),
    ("add_to_temp_array", Extract::InternalAssignOrField("array"), GameFlags::Hoi4),
    ("add_to_variable_list", Extract::Field("name"), GameFlags::jomini()),
];
