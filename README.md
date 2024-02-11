# ![Tiger Banner](banner.png)

[![GitHub Release](https://img.shields.io/github/v/release/amtep/ck3-tiger)](https://github.com/amtep/ck3-tiger/releases)
[![GitHub License](https://img.shields.io/github/license/amtep/ck3-tiger)](https://github.com/amtep/ck3-tiger/blob/main/LICENSE)
[![tiger-lib docs.rs](https://img.shields.io/docsrs/tiger-lib?label=tiger-lib%20docs)](https://docs.rs/tiger-lib/latest/tiger_lib/)
[![Discord](https://img.shields.io/discord/1137432658067062784?logo=discord&label=discord&labelColor=1137432658067062784&color=royalblue)](https://discord.gg/3uQVCJ8uVf)

Tiger pounces on bugs. Checks Crusader Kings 3 user mod files for mistakes and warns about them. For example: missing localizations, or using a faith trigger on a character.

Crusader Kings 3 is a grand strategy game made by Paradox Interactive, and user mods are made by the players to enhance or change their game experience. This tool is for the people who make the mods.

`ck3-tiger` now also comes with `vic3-tiger`, which does the same thing for Victoria 3.

## Status

This project is maturing but not yet stable. It will warn about some things that are actually correct.

`vic3-tiger` is somewhat younger than `ck3-tiger` and will have less accurate warnings.

## Features

`ck3-tiger` (or `vic3-tiger`) will read the relevant vanilla files and your mod's files, and it will complain about everything in your mod that looks wrong. Where possible, it will tell you why it thinks the thing is wrong and (still in very few cases) what you should do instead.

* Syntax validation: are you using the right key = value pairs? No misspellings?
* Missing items: is every game object that you refer to actually defined somewhere?
* Missing localizations: do you have all the localizations you need for your mod?
* Scope consistency checking: are you using culture effects on cultures and character effects on characters, etc.?
* History (for CK3): Are spouses, employers, and lieges alive on the relevant dates? Is no one their own grandfather?
* Special: rivers.png check

`ck3-tiger` loads and checks nearly all the game files.
`vic3-tiger` still has gaps in its coverage of the game item types.

## Sample output

![Sample Output](example.png)

![Sample Output](example2.png)

![Sample Output](example3.png)

## How to use

### `ck3-tiger`

Download a release package from [GitHub](https://github.com/amtep/ck3-tiger/releases). Unpack it somewhere.

On Windows, if everything works out, you can then just double-click on `ck3-tiger-auto` and it will try its best.

Otherwise, run the tool from the command prompt:
<pre>
<i>path/to/</i>ck3-tiger <i>path/to/your/</i>descriptor.mod
</pre>
or
<pre>
<i>path/to/</i>ck3-tiger "<i>path/to/</i>Paradox Interactive/Crusader Kings III/mod/YourMod.mod"
</pre>

(Note that the quote marks around the path are important because of the spaces in it.)

If you want the output in a file, you can redirect it like this:
<pre>
ck3-tiger <i>path/to/your/</i>descriptor.mod ><i>filename</i>
</pre>

### `vic3-tiger`

Download a release package from [GitHub](https://github.com/amtep/ck3-tiger/releases). Unpack it somewhere.

On Windows, if everything works out, you can then just double-click on `vic3-tiger-auto` and it will try its best.

Otherwise, run the tool from the command prompt:
<pre>
<i>path/to/</i>vic3-tiger <i>path/to/your/mod</i>
</pre>
or
<pre>
<i>path/to/</i>vic3-tiger "<i>path/to/</i>Paradox Interactive/Victoria 3/mod/YourMod/"
</pre>

(Note that the quote marks around the path are important because of the spaces in it.)

If you want the output in a file, you can redirect it like this:
<pre>
vic3-tiger <i>path/to/your/mod</i> ><i>filename</i>
</pre>

## How to configure

You can place a file `ck3-tiger.conf` (or `vic3-tiger.conf`) in your mod directory. You can use it to select which languages to check localizations for, and to suppress messages about things you don't want to fix.

There is a sample [`ck3-tiger.conf`](ck3-tiger.conf) file and [`vic3-tiger.conf`](vic3-tiger.conf) file in the release, with an explanation of what goes in it. There is also a [guide](filter.md).

## How to build

You can unpack the archive from the "Release" page on GitHub and use it that way.

If you want to build it yourself, you will have to [install the Rust programming language](https://www.rust-lang.org/tools/install).

For `ck3-tiger`, run `cargo build --release -p ck3-tiger` in the project's directory, then run the program as `cargo run --release -p ck3-tiger`.
For `vic3-tiger`, run `cargo build --release -p vic3-tiger` in the project's directory, then run the program as `cargo run --release -p vic3-tiger`.

## Visual Studio Code extension

User unLomTrois has made a [VS Code extension](https://github.com/unLomTrois/ck3tiger-for-vscode) for `ck3-tiger`.
It enables you to view the reports directly in the Problems tab.

## Contributions

I welcome contributions and collaborations! Some forms that contributions can take:

* Suggestions and ideas about what things tiger should check
* Telling me which of the output messages are confusing or hard to understand
* Reporting cases where tiger complains about a problem that's not real (false positives)
* Filing an issue on GitHub about a problem you have, or sending me email directly
* Starting up the game to verify something that's marked "TODO: verify" in the code

Contributions in the form of code are also welcome. They should be made as GitHub pull requests, and you should read and understand the project's copyright license before doing so. It may help to file a GitHub issue before starting to code, though, since I may prefer to solve the problem in a different way.

Some ideas for code contributions:

* Adding a new check and its error report
* Adding a validator for a new item type
* Updating a validator to a new game version
* Solving one of the hundreds of TODO comments in the code
* Solving one of the issues marked in the [TODO](https://github.com/amtep/ck3-tiger/wiki/Todo) or [IDEAS](https://github.com/amtep/ck3-tiger/wiki/Ideas) wiki pages
* Speed or memory use improvements; opportunities are all over the place

See the [CODING](https://github.com/amtep/ck3-tiger/wiki/Overview-for-coders) wiki page for an overview of the code and coding style.

### License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the [`LICENSE`](LICENSE) file for more details.
