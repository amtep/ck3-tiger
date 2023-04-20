#!/usr/bin/python3

import os.path
import sys

OUTDIR = "src/tables/include"

SEPARATOR = "\n-----------------------\n\n"

GLOBAL_PROMOTES = ""
GLOBAL_FUNCTIONS = ""
GLOBAL_MACROS = ""
TYPES = ["Unknown"]
FUNCTIONS = ""
PROMOTES = ""

fnames = sys.argv[1:]

for fname in fnames:
    text = open(fname, encoding="windows-1252").read()
    items = text.split(SEPARATOR)

    for item in items:
        if not item:
            continue
        lines = item.splitlines()
        name = lines[0].split('(')[0]

        args = "NoArgs"
        if "Arg0" in lines[0]:
            args = "Arg(Unknown)"
        if "Arg1" in lines[0]:
            args = "Arg2(Unknown, Unknown)"
        if "Arg2" in lines[0]:
            args = "Arg3(Unknown, Unknown, Unknown)"
        if "Arg3" in lines[0]:
            args = "Arg4(Unknown, Unknown, Unknown, Unknown)"

        if "Definition type: Global macro" in item:
            GLOBAL_MACROS += item
            continue

        if "Definition type: Type" in item:
            if name not in TYPES:
                TYPES += [name]
            continue

        rtype = lines[-1].split("Return type: ")[1].strip()
        if rtype == "[unregistered]":
            rtype = "Unknown"

        if "\nDefinition type: Global promote\n" in item:
            GLOBAL_PROMOTES += '    ("%s", %s, %s),\n' % (name, args, rtype)
        elif "\nDefinition type: Global function\n" in item:
            GLOBAL_FUNCTIONS += '    ("%s", %s, %s),\n' % (name, args, rtype)
        elif "\nDefinition type: Function\n" in item:
            type, name = name.split(".")
            if name == "AccessSelf" or name == "Self":
                rtype = type
            FUNCTIONS += '    (%s, "%s", %s, %s),\n' % (type, name, args, rtype)
        elif "\nDefinition type: Promote\n" in item:
            type, name = name.split(".")
            PROMOTES += '    (%s, "%s", %s, %s),\n' % (type, name, args, rtype)
        else:
            print(item)
            raise "unknown item"

with open(OUTDIR + "/datatypes.rs", "w") as outf:
    outf.write("#[derive(Copy, Clone, Debug)]\n");
    outf.write("enum DataType {\n");
    outf.write("    " + ",\n    ".join(TYPES) + ",\n")
    outf.write("}\n");

with open(OUTDIR + "/data_global_promotes.rs", "w") as outf:
    outf.write("&[\n");
    outf.write(GLOBAL_PROMOTES)
    outf.write("]\n");

with open(OUTDIR + "/data_global_functions.rs", "w") as outf:
    outf.write("&[\n");
    outf.write(GLOBAL_FUNCTIONS)
    outf.write("]\n");

with open(OUTDIR + "/data_promotes.rs", "w") as outf:
    outf.write("&[\n");
    outf.write(PROMOTES)
    outf.write("]\n");

with open(OUTDIR + "/data_functions.rs", "w") as outf:
    outf.write("&[\n");
    outf.write(FUNCTIONS)
    outf.write("]\n");

# These need further processing
print("GLOBAL MACROS")
print(GLOBAL_MACROS)

