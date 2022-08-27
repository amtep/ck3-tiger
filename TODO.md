# TODO

## Bugs

### Display

* The caret placement for error messages assumes 1 byte of the source string = 1 space, which is inaccurate both for tabs and for multibyte characters.
* The column numbers reported for pdxscript files are wrong.
* There are some non-Unicode files in CK3. The error reporting tries to load them as Unicode anyway.
* The line count for events/court_events/court_events_general.txt is wrong somehow. It agrees with errors.rs but not with vi or less.

## Features

* The validations in IDEAS.md
* Warn if vanilla redefines something from the mod. Tricky because logging is turned off for vanilla items.
