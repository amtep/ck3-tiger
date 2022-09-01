# Ideas for potential checks

## General
* replace_path doesn't work recursively. It only works on the specific directory. (Warn about replace_path = "history" for example)
* replace_path on the province mapping directory will crash the game
* root.set_variable doesn't work; an effect can't be chained to a scope. Has to be root = { set_variable ... }

## GUI
* ck3 crashes if you have multiple objects with `resizeparent` in one parent

## Tooltips
* `create_character` is not executed during tooltip generation, so the scope it creates won't exist yet.
* `title:k_burgundy = { is_title_created = yes }` gives a nicer tooltip than `exists = title:k_burgundy.holder`
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
