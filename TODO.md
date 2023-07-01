# TODO

## Bugs

## False positives

## Features

* Check that relation flags actually belong to the relation they are used with
* The validations in IDEAS.md
* Hundreds of TODO comments in the code

## Refactoring

* ScriptValue::validate_bv should be in its own module just like trigger and effect
* The munch_data_types.py script has an ever-growing list of overrides. It should instead be smart enough to parse the existing Rust tables and only add/remove the functions that changed. That way, the overrides can be done directly in Rust code.
