# TODO

## Bugs

## False positives

* compare_modifier can take other targets than `Character`. It should evaluate its value in the same scope type as the target.
* custom loca should be checked on demand, so that the existence of the keys can be verified for the language it's used in, and custom loca called for other reasons than localization don't have to be checked. This should also allow checking the custom loca's scope type at the call site.

## Features

* Check that relation flags actually belong to the relation they are used with
* The validations in IDEAS.md
* Hundreds of TODO comments in the code
