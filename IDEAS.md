# Ideas for potential checks

## General
* root.set_variable doesn't work; an effect can't be chained to a scope. Has to be `root = { set_variable ... }`
* Some code (in scripted_lists and in lifestyles) is not allowed to use scripted content in its triggers
* Many effects and triggers have a target field. It almost never makes sense to have who = this or target = this as the target. It's worth warning about especially in macros.
* When using a scripted trigger in an effect, don't just say unknown token. Say this is a trigger and it should be in a limit field.
* Certain script values are referenced in code; check that they are defined and have the right scopes.

## Advice
* for `current_date >= 1200.1.1` suggest `current_year >= 1200`
* for `current_date < 1201.1.1` suggest `current_year < 1200`
* for both in one block, suggest current_year = 1200
* keep in mind that in empty scope, current_year doesn't work

## GUI
* ck3 crashes if you have multiple objects with `resizeparent` in one parent

## Tooltips
* `create_character` is not executed during tooltip generation, so the scope it creates won't exist yet.
* iterators should have a `custom =` to summarize the iterator. ("All infidel counties:" for example)
* character interaction can't show OR in is_valid_showing_failures_only, should change them to custom_description or trigger_if.

## Performance
* Doing `if = { limit = {` in an iterator is often redundant, you can do `limit = {` directly in the iterator instead.

## Maps
* Check resolution and graphics format for all the pngs in map_data
* Check that `heightmap.png` is not newer than `indirection_heightmap.png` and `packed_heightmap.png`

## History
* bookmarked characters need to have a static COA for their highest landed title, because if it's random it will be blank on the bookmark selection view
* title history: Warn if one character gets multiple different lieges on the same day

## Gfx
* CK3 needs all of its bones to be oriented y+ in rest position

## Crashes
* If you try to use scripted triggers in music files it will crash on startup
* scripted_triggers in the dynasty legacy tracks too, though individual perks are fine
* gfx: if a template is in modifiers the string should be somewhere in the genes
