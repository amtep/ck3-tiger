# Ideas for potential checks

## GUI
* ck3 crashes if you have multiple objects with `resizeparent` in one parent

## Tooltips
* `create_character` is not executed during tooltip generation, so the scope it creates won't exist yet.
* `title:k_burgundy = { is_title_created = yes }` gives a nicer tooltip than `exists = title:k_burgundy.holder`
* iterators should have a `custom =` to summarize the iterator. ("All infidel counties:" for example)

## Performance
* Doing `if = { limit = {` in an iterator is often redundant, you can do `limit = {` directly in the iterator instead.

## Maps
* Check the detailed requirements for pixels in `rivers.png`
* Check resolution and graphics format for all the pngs in map_data
* Check that `heightmap.png` is not newer than `indirection_heightmap.png` and `packed_heightmap.png`
