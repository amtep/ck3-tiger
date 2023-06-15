#!/usr/bin/python3

import os.path
import sys

OUTDIR = "src/tables/include"

SEPARATOR = "\n-----------------------\n\n"

# Vec4f is not listed as a datatype but there is a Select_vec4f function so it must exist.
TYPES = ['    Vec4f,\n']
GLOBAL_PROMOTES = []
GLOBAL_FUNCTIONS = []
FUNCTIONS = []
PROMOTES = []

# Experimentation shows that Root is accepted as well as ROOT
GLOBAL_PROMOTES.append('    ("Root", NoArgs, Scope),\n')

# Most promotes and functions have 'Unknown' arg types in the data_type logs.
# This dictionary replaces those with known arg types in specific cases.
ARGS_OVERRIDE = {
    "EqualTo_string": "Arg2(DType(CString), DType(CString))",
}

# UNARY_ARGS are functions that have their argument type in their name
UNARY_ARGS = ["Abs_", "GetString_", "Negate_"]
# BINARY_ARGS are binary functions that have their argument type in their name
BINARY_ARGS = [
    "Add_", "EqualTo_", "Divide_", "GetNumberAbove_", "GreaterThanOrEqualTo_",
    "GreaterThan_", "LessThanOrEqualTo_", "LessThan_", "Max_", "Multiply_",
    "NotEqualTo_", "Subtract_"
]


# Most functions have 'Unknown' return types in the data_type logs.
# This dictionary replaces those with known return types in specific cases.
RTYPE_OVERRIDE = {
    "GetNullCharacter": "Character",
}

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
            args = "Arg1(DType(Unknown))"
        if "Arg1" in lines[0]:
            args = "Arg2(DType(Unknown), DType(Unknown))"
        if "Arg2" in lines[0]:
            args = "Arg3(DType(Unknown), DType(Unknown), DType(Unknown))"
        if "Arg3" in lines[0]:
            args = "Arg4(DType(Unknown), DType(Unknown), DType(Unknown), DType(Unknown))"
        if "Arg4" in lines[0]:
            args = "Arg5(DType(Unknown), DType(Unknown), DType(Unknown), DType(Unknown), DType(Unknown))"
        for s in UNARY_ARGS:
            if name.startswith(s):
                type = name.split('_')[1]
                args = 'Arg1(DType(%s))' % type
        for s in BINARY_ARGS:
            if name.startswith(s):
                type = name.split('_')[1]
                args = 'Arg2(DType(%s), DType(%s))' % (type, type)
        if name.startswith("Select_"):
            type = name.split('_')[1]
            args = 'Arg3(DType(bool), DType(%s), DType(%s))' % (type, type)
        if name in ARGS_OVERRIDE:
            args = ARGS_OVERRIDE[name]

        if "Definition type: Global macro" in item:
            # macros are parsed directly from data_binding
            continue

        if "Definition type: Type" in item:
            typeline = '    %s,\n' % name
            if typeline not in TYPES:
                TYPES.append(typeline)
            continue

        rtype = lines[-1].split("Return type: ")[1].strip()
        if rtype == "[unregistered]":
            rtype = "Unknown"
        if rtype == "_null_type_":
            rtype = "void"
        if rtype == "Unknown" and name in RTYPE_OVERRIDE:
            rtype = RTYPE_OVERRIDE[name]

        if "\nDefinition type: Global promote\n" in item:
            GLOBAL_PROMOTES.append('    ("%s", %s, %s),\n' % (name, args, rtype))
        elif "\nDefinition type: Global function\n" in item:
            GLOBAL_FUNCTIONS.append('    ("%s", %s, %s),\n' % (name, args, rtype))
        elif "\nDefinition type: Function\n" in item:
            type, name = name.split(".")
            if name == "AccessSelf" or name == "Self":
                rtype = type
            FUNCTIONS.append('    ("%s", %s, %s, %s),\n' % (name, type, args, rtype))
        elif "\nDefinition type: Promote\n" in item:
            type, name = name.split(".")
            PROMOTES.append('    ("%s", %s, %s, %s),\n' % (name, type, args, rtype))
        else:
            print(item)
            raise "unknown item"

with open(OUTDIR + "/datatypes.rs", "w") as outf:
    TYPES.sort()
    outf.write("#[derive(Copy, Clone, Debug, Eq, PartialEq, Display, EnumString)]\n")
    outf.write("pub enum Datatype {\n")
    outf.write("    Unknown,\n")
    outf.write("".join(TYPES))
    outf.write("}\n")

with open(OUTDIR + "/data_global_promotes.rs", "w") as outf:
    GLOBAL_PROMOTES.sort()
    outf.write("&[\n")
    outf.write("".join(GLOBAL_PROMOTES))
    outf.write("]\n")

with open(OUTDIR + "/data_global_functions.rs", "w") as outf:
    GLOBAL_FUNCTIONS.sort()
    outf.write("&[\n")
    outf.write("".join(GLOBAL_FUNCTIONS))
    outf.write("]\n")

with open(OUTDIR + "/data_promotes.rs", "w") as outf:
    PROMOTES.sort()
    outf.write("&[\n")
    outf.write("".join(PROMOTES))
    outf.write("]\n")

with open(OUTDIR + "/data_functions.rs", "w") as outf:
    FUNCTIONS.sort()
    outf.write("&[\n")
    outf.write("".join(FUNCTIONS))
    outf.write("]\n")
