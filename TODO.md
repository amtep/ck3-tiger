# TODO

## Bugs

* data_binding replacements can be chains, not just single codes
* GetAnimatedBookmarkPortrait takes 5 arguments but datatype.rs can only handle up to 4

## False positives

* ai_value_modifier can have `dread_modified_ai_boldness` that takes a block
* In geographical regions, the requirement to define regions before use seems to not be true anymore
* fields `graphical` and `color` for geographical regions
* New province modifier `travel_danger`
* New fields `travel_danger_color` and `travel_danger_score` for terrain
* `position` can take a variable
* Parse max_naval_distance in any_connected_county
* When squared_distance has a () argument, it produces a Value
* Parse `[Concept('A', 'B')|E]` in localization
* parse `local_template` in gui files
* parse `layer` in gui files
* Parse and validate #tooltip: directives in localization

## Features

* If a datafunction is not found, but it does exist with a case-insensitive search, offer that version as an info hint
* Check that relation flags actually belong to the relation they are used with
* The validations in IDEAS.md
* Hundreds of TODO comments in the code
