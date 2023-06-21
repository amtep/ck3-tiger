# TODO

## Bugs

## False positives

* With localization, duplicate keys take the earlier entry not the later entry.

## Features

* Check that relation flags actually belong to the relation they are used with
* When reporting missing loca, have just one message instead of one per language
* The validations in IDEAS.md
* Hundreds of TODO comments in the code

## Refactoring

* Validator::unknown_keys should be separated into unknown_key_blocks and unknown_key_values
* ScriptValue::validate_bv should be in its own module just like trigger and effect
