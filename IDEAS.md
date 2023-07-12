# Ideas for potential checks

## General
* root.set_variable doesn't work; an effect can't be chained to a scope. Has to be `root = { set_variable ... }`
* Some code (in scripted_lists and in lifestyles) is not allowed to use scripted content in its triggers
* Many effects and triggers have a target field. It almost never makes sense to have who = this or target = this as the target. It's worth warning about especially in macros.
* When using a scripted trigger in an effect, don't just say unknown token. Say this is a trigger and it should be in a limit field.
* Certain script values are referenced in code; check that they are defined and have the right scopes.
* It should be possible to follow scope definitions through on_actions and events and make sure every scope: reference is one that was set in previous code.
* There are now a bunch of scopes that have "Execute Effects: no" in `event_scopes.log`. Check that those aren't used in effects (especially including the `set_variable` effect).
* Warn if a script value has if blocks or iterators that end up not changing anything about the value (that is, there's no `add` statement or similar)
* Also warn if a modifier has triggers but no `add` or `factor`

## Advice
* for `current_date >= 1200.1.1` suggest `current_year >= 1200`
* for `current_date < 1201.1.1` suggest `current_year < 1200`
* for both in one block, suggest current_year = 1200
* keep in mind that in empty scope, current_year doesn't work
* `50 = 0` in random_list is invalid, suggest `50 = {}` instead
* when a target is invalid, check if the user forgot to put a faith: or culture: prefix
* sometimes you want an any_ list with empty trigger. User might try any_... = yes, but it should be any_... = {}

## GUI
* ck3 crashes if you have multiple objects with `resizeparent` in one parent
* Can't do `datacontext = [Character.GetLiege]` if your datacontext is already `Character` (and more generally, writing the datacontext you're already in)

## Tooltips
* `create_character` is not executed during tooltip generation, so the scope it creates won't exist yet.
* iterators should have a `custom =` to summarize the iterator. ("All infidel counties:" for example)

## Performance
* Doing `if = { limit = {` in an iterator is often redundant, you can do `limit = {` directly in the iterator instead.
* Warn about nested AND in a trigger that's already AND, or a double nested OR.
* `every_living_character` with a limit that restricts it to rulers, or to players, can be made more performant by using a more restrictive iterator.

## Maps
* Check resolution and graphics format for all the pngs in map_data
* Check that `heightmap.png` is not newer than `indirection_heightmap.png` and `packed_heightmap.png`
* map dimensions should be multiple of 32x32

## History
* title history: Warn if one character gets multiple different lieges on the same day
* warn if a character is born twice or dies twice

## Gfx
* CK3 needs all of its bones to be oriented y+ in rest position

## Crashes
* If you try to use scripted triggers in music files it will crash on startup
* scripted_triggers in the dynasty legacy tracks too, though individual perks are fine
