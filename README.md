# ck3-tiger
Pounces on bugs. Checks Crusader Kings 3 user mod files for common mistakes and warns about them. For example: missing localizations, or using a faith trigger on a character.

## Status
This project still in its beginning stages. It will warn about some things that are actually correct.

## Features
`ck3-tiger` will read the relevant vanilla files and your mod's files, and it will complain about everything in your mod that looks wrong. Where possible, it will tell you why it thinks the thing is wrong and (still in very few cases) what you should do instead.

* Syntax validation: are you using the right key = value pairs? No misspellings?
* Missing items: is every game object that you refer to actually defined somewhere?
* Missing loca: do you have all the localizations you need for your mod?
* Scope consistency checking: are you using culture effects on cultures and character effects on characters, etc?
* Special: rivers.png check

It doesn't load all the game files yet, but it hits the major ones (events, decisions, history, localization, scripted triggers and effects) and many minor ones.

## Contributions

I welcome contributions in the form of suggestions and ideas about what it should check! Also if you have trouble understanding the output messages, feel free to let me know. They could always be more clear. You can file an issue on github or contact me directly via email. The same goes for bug reports. In particular, you can submit bug reports about false positives in the program output.

Contributions in the form of code are also welcome. They should be made as github Pull Requests, and you should read and understand the project's copyright license before doing so. It may help to file an issue before starting to code, though, since I may prefer to solve the issue in a different way.

### License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the `LICENSE` file for more details.

## How to use
Run it from the command line:
<pre>
ck3-tiger <i>path/to/your/</i>descriptor.mod
</pre>
or
<pre>
ck3-tiger "<i>path/to/</i>Paradox Interactive/Crusader Kings III/mod/YourMod.mod"
</pre>

## How to configure
You can place a file `ck3-tiger.conf` in your mod directory. You can use it to select which languages to check localizations for, and to suppress messages about things you don't want to fix.

There is a sample `ck3-tiger.conf` file in the release, with an explanation of what goes in it.

## How to build
You can unpack the archive from the "Release" page on github and use it that way.

If you want to build it yourself, you will have to install the Rust programming language:
https://www.rust-lang.org/tools/install

Then run `cargo build --release` in the project's directory, then run the program as `target/release/ck3-tiger` .

## Sample output
<pre>
[CK3] events/dlc/fp1/fp1_shieldmaiden_events.txt:666:           any_scheme_agent = { this = scope:prospective_shieldmaiden }
[CK3] events/dlc/fp1/fp1_shieldmaiden_events.txt:666:           ^
[CK3] events/dlc/fp1/fp1_shieldmaiden_events.txt:666:3: WARNING: `any_scheme_agent` is for scheme but scope seems to be character
[CK3] events/dlc/fp1/fp1_shieldmaiden_events.txt:656:   has_trait = shieldmaiden
[CK3] events/dlc/fp1/fp1_shieldmaiden_events.txt:656:   ^
[CK3] events/dlc/fp1/fp1_shieldmaiden_events.txt:656:2: INFO: scope was deduced from `has_trait` here
</pre>
