#!/usr/bin/python3

import os.path
import sys

fname = sys.argv[1]

text = open(fname).read()

SEPARATOR = "\n--------------------\n\n"

items = text.split(SEPARATOR)
print( len(items))

if fname.endswith("event_targets.log"):
    header = items[0]
    footer = items[-1]
    del items[0]
    del items[-1]
    items.sort()
    items.insert(0, header)
    items.append(footer)
    print(SEPARATOR.join(items))

elif fname.endswith("triggers.log"):
    header = items[0:2]
    del items[0:1]

    items.sort()

    items.insert(0, header[1])
    items.insert(0, header[0])

    print(SEPARATOR.join(items))
