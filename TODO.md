# TODO

## Bugs

* data_binding replacements can be chains, not just single codes

## False positives

* Event themes also have a `transition` field
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
* The validations in IDEAS.md
* Hundreds of TODO comments in the code
