# Core concepts and terminology

## `Token` and `Loc`
A `Token` is a small piece of script from the game, together with a `Loc`: its location in the game code.
A token is the smallest piece of script produced by the parser, and is usually the smallest meaningful part, but there are methods to break a token up into subtokens; for example to handle scope chains like `root.primary_title.tier` which is one token that can be broken up into three smaller ones by the validator.

A `Loc` specifies the pathname to the file in which a `Token` appears (starting from the game directory or the mod directory), as well as the line number and column number within that file. It is mostly used for error reporting, both in the reports themselves and to control what gets reported.

Most tokens are directly from the script files, but there are also synthetic tokens that represent the logical value, not the literal value, of a piece of script. This occurs mostly in macro processing.

Two tokens compare equal if their strings are equal, regardless of their locs.

## Pdx script

Pdx script is what's in nearly all the `.txt` files in the game.
The language doesn't have an official name.
In fact, most of the terms in this section are made up for the validator and don't come from Pdx.

The language is mostly declarative, with some bits of imperative logic (the *effects*) and conditionals (the *triggers*).
Pdx script is parsed into Rust datatypes by the `parse::pdxfile` module.

### `Block`

The core of the representation of Pdx script is the `Block`.
Blocks are delimited by `{` and `}`.
An entire file is also a `Block`.
Blocks can contain a mix of these kinds of items:

* Assignments: `key = value`
* Definitions: `key = { ... }`
* Loose sub-blocks: `{ ... } { ... } ...`
* Loose values: `value value ...`
* Comparisons: `key < value` for a variety of comparators, including `=` for equality
* `key < { ... }` is accepted by the parser but is not used anywhere

The same key can occur multiple times in a block.
Sometimes they are added together into a list of some sort, sometimes they override each other (latest key wins).
Overriding is often an error and is reported by the validator.

Note that there is overlap between comparisons and assignments.
The parser cannot distinguish them and doesn't try.
They are handled the same and it's up to the validator to accept only `=` or also other comparators.

#### Tagged block
A block may be *tagged* by a special token in front of it, for example `color = hsv { 1.0 0.5 0.5 }`.
This is handled specially by the parser for a limited number of tags, so that it is treated as a definition of `color` rather than a `color = hsv` assignment followed by a loose block.

### `BlockItem`
A `Block` contains a vector of `BlockItem` to represent its contents.
A `BlockItem` represents the variations listed above. It is an `enum` that is either a keyed `Field`, a loose `Block`, or a loose `Value`.

### `Field`
A `Field` contains a key, a comparator, and either a block or a value.
It's handled this way, rather than distinguishing between assignments and definitions at this level, because the validator often needs to look up a key regardless of whether it is in front of a block or a value.

### `BV`
A `BV` is an `enum` that is either a block or a value.
It is returned when you look up a key without specifying whether you want only values or only blocks.
The code is careful to always call a variable of this type a "bv" and not something confusing like "value".
A `BV` contains convenience methods `expect_block` and `expect_value` to return the expected type and emit a warning if it doesn't contain the expected type.

### Keys and values
Both keys and values are represented by `Token`.
You will often see a `Value` being called "token" in the code rather than "value", because "value" is a bit vague.
Values (and sometimes keys) can be converted into numbers or dates if they are valid tokens of that type.

## Localization

**TODO**

## Items

Pdx script defines a huge variety of different kinds of database items: faiths, landed titles, characters, buildings, coats of arms, and a hundred more.
These are all categorized by the giant `Item` enum, which is used as a lookup key (together with a string or `Token` to identify the item) in many places.

Each validator in the `data` directory loads and validates one kind of item or a small group of related items.

Because of the name of the enum, the code does not make much distinction between "item" and "item type", though you will see "itype" used in some older parts of the code. This distinction may be worth improving.

## Scopes

**TODO**

# Parsing, loading and validation

**TODO**

# Coding style

## Formatting

Code is formatted with `cargo fmt` at the default settings except that `use_small_heuristics = "Max"` is set to make better use of the screen space.
The `rustfmt.toml` file in the repo root directory takes care of this.

Some of the huge tables could probably benefit from an exception to this formatting, to keep their rows as single lines, but that hasn't been done yet.

## Variable names

Short-term variables are often just named after their type or role.
If you have a key and its block, just call them "key" and "block"; no need to get creative.
A `Validator` is always called "vd", a `ScopeContext` is always called "sc", the `&Everything` reference is always called "data", etc.
Using consistent names for each type, and only using other names when it really matters, will lighten the cognitive load for the reader once they are familiar with these conventions.

## Function arguments

Function arguments are in a fairly consistent order: key, block or bv, data, sc, vd where applicable, followed by any booleans.
Lookup functions take an `Item` and then a `&str` or `Token`. Validator functions start with the field name.
(These last two rules sometimes conflict, giving us `vd.field_item(name, item)` and `data.verify_exists(item, name)`).

## Features

The validator defines a *feature* for each game it supports (`ck3` and `vic3`).
Normal `cargo` rules are that features must be additive, not conflicting, but we're breaking that rule.
You can turn on either `ck3` or `vic3` but not both.
This is done by deciding whether to build the `ck3-tiger` or the `vic3-tiger` crate. Each contains a binary and depends on the feature it needs.

The code should be buildable with either feature without emitting warnings. This can sometimes involve silencing warnings with `#[allow(...)]` attributes in the code.

## Imports

`use` statements go at the top of the file, with imports from `std` in a separate block at the top, then imports from other crates, then `crate::` imports from the validator itself.
Each block is sorted alphabetically.
Multiple items are combined into `{` lists `}` only when they come from the same module, with the exception of `data` modules which are combined to avoid repeating the `#[cfg(feature = ...)]` over and over.

