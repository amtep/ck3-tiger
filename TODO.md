# TODO

## Bugs

## False positives

## Features

* Check that relation flags actually belong to the relation they are used with
* The validations in IDEAS.md
* Hundreds of TODO comments in the code

## Refactoring

* The munch_data_types.py script has an ever-growing list of overrides. It should instead be smart enough to parse the existing Rust tables and only add/remove the functions that changed. That way, the overrides can be done directly in Rust code.
* Make each `Block` carry a copy of its key so that we don't have to pass key parameters around everywhere.
* Make `Validator` able to take a `BV` or `Token` and validate values as well as blocks, for more uniform code.
* the effects table has a lot of Special entries, which then necessitate another string lookup to handle them. Maybe it would be better if the effects table contained function pointers in the first place.
