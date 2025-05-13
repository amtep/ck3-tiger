#!/bin/bash

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "usage: $0 NEWVERSION" 1>&2
    exit 2
fi

for toml in Cargo.toml */Cargo.toml; do
    sed -i -e "s/^version = .*/version = \"$VERSION\"/" \
           -e 's/^\(tiger-.*, version = \)"[.0-9]*"/\1'"\"$VERSION\"/" \
           "$toml"
done
