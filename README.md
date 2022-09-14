# ck3-tiger
Pounces on bugs. Checks Crusader Kings 3 user mod files for common mistakes and warns about them.

## Status
This project still in its beginning stages. It will warn about many things that are actually correct.

## Features
`ck3-tiger` will read the relevant vanilla files and your mod's files, and it will complain about everything in your mod that looks wrong. Where possible, it will tell you why it thinks the thing is wrong and (still in very few cases) what you should do instead.

* Syntax validation: are you using the right key = value pairs? No misspellings?
* Missing items: is every game object that you refer to actually defined somewhere?
* Missing loca: do you have all the localizations you need for your mod?
* Scope consistency checking: are you using culture effects on cultures and character effects on characters, etc?
* Special: rivers.png check

It doesn't load all the game files yet, but it hits the major ones (events, decisions, scripted triggers and effects, script values, localization) and many minor ones.

## Contributions

I welcome contributions in the form of suggestions and ideas about what it should check! Also if you have trouble understanding the output messages, feel free to let me know. They could always be more clear. You can file an issue on github or contact me directly via email.

Contributions in the form of code are also welcome. They should be made as github Pull Requests, and you should read and understand the project's copyright license before doing so. It may help to file an issue before starting to code, though, since I may prefer to solve the issue in a different way.

## How to use
Run it from the command line:
<pre>
ck3-tiger<i>path/to/your/</i>descriptor.mod
</pre>
or
<pre>
ck3-tiger "<i>path/to/</i>Paradox Interactive/Crusader Kings III/mod/YourMod.mod"
</pre>

## How to build
You can unpack the archive from the "Release" page on github and use it that way.

If you want to build it yourself, you will have to install the Rust programming language:
https://www.rust-lang.org/tools/install

Then run `cargo build --release` in the project's directory, then run the program as `target/release/ck3-tiger` .
