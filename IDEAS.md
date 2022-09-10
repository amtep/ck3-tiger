# Ideas for potential checks

## General
* root.set_variable doesn't work; an effect can't be chained to a scope. Has to be `root = { set_variable ... }`
* Some code (in scripted_lists and in lifestyles) is not allowed to use scripted content in its triggers

## Advice
* for `current_date >= 1200.1.1` suggest `current_year >= 1200`
* for `current_date < 1201.1.1` suggest `current_year < 1200`
* for both in one block, suggest current_year = 1200

## GUI
* ck3 crashes if you have multiple objects with `resizeparent` in one parent

## Tooltips
* `create_character` is not executed during tooltip generation, so the scope it creates won't exist yet.
* iterators should have a `custom =` to summarize the iterator. ("All infidel counties:" for example)
* character interaction can't show OR in is_valid_showing_failures_only, should change them to custom_description or trigger_if.

## Performance
* Doing `if = { limit = {` in an iterator is often redundant, you can do `limit = {` directly in the iterator instead.

## Maps
* Check the detailed requirements for pixels in `rivers.png`
* Check resolution and graphics format for all the pngs in map_data
* Check that `heightmap.png` is not newer than `indirection_heightmap.png` and `packed_heightmap.png`

## History
* In history files, setting `liege =` to a title that has no holder at that date will not set the vassal

## Gfx
* CK3 needs all of its bones to be oriented y+ in rest position

## Crashes
* If you try to use scripted triggers in music files it will crash on startup
* scripted_triggers in the dynasty legacy tracks too, though individual perks are fine
* gfx: if a template is in modifiers the string should be somewhere in the genes
