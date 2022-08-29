# ck3-mod-validator
Checks Crusader Kings 3 user mod files for common mistakes and warns about them.

## Status
This project still in its beginning stages. It might warn about things that are actually correct.

## How to use
Run it from the command line:
<pre>
ck3-mod-validator <i>path/to/your/</i>descriptor.mod
</pre>
or
<pre>
ck3-mod-validator "<i>path/to/</i>Paradox Interactive/Crusader Kings III/mod/YourMod.mod"
</pre>

## How to build
There's no ready-to-use package for this program yet.

You will have to install the Rust programming language:
https://www.rust-lang.org/tools/install

Then run `cargo build --release` in the project's directory, then run the program as `target/release/ck3-mod-validator` .
