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
    ("Artifact.GetFeatureText", 1): "Arg1(IType(Item::ArtifactFeatureGroup))",
    ("Character.Custom2", 2): "Arg2(DType(CString), DType(AnyScope))",
    ("EqualTo_string", 2): "Arg2(DType(CString), DType(CString))",
    ("FaithDoctrine.GetName", 1): "Arg1(DType(Faith))",
    ("FaithDoctrineGroup.GetName", 1): "Arg1(DType(Faith))",
    ("GetAccoladeType", 1): "Arg1(IType(Item::AccoladeType))",
    ("GetActivityGuestInviteRule", 1): "Arg1(IType(Item::GuestInviteRule))",
    ("GetActivityIntent", 1): "Arg1(IType(Item::ActivityIntent))",
    ("GetActivityLocale", 1): "Arg1(IType(Item::ActivityLocale))",
    ("GetActivityPulseAction", 1): "Arg1(IType(Item::PulseAction))",
    ("GetActivityType", 1): "Arg1(IType(Item::ActivityType))",
    ("GetArtifactType", 1): "Arg1(IType(Item::ArtifactType))",
    ("GetArtifactVisualType", 1): "Arg1(IType(Item::ArtifactVisual))",
    ("GetBookmark", 1): "Arg1(IType(Item::Bookmark))",
    ("GetBookmarkGroup", 1): "Arg1(IType(Item::BookmarkGroup))",
    ("GetBuilding", 1): "Arg1(IType(Item::Building))",
    ("GetCasusBelliType", 1): "Arg1(IType(Item::CasusBelli))",
    ("GetCatalystType", 1): "Arg1(IType(Item::Catalyst))",
    ("GetCharacterInteraction", 1): "Arg1(IType(Item::Interaction))",
    ("GetCharacterInteractionCategory", 1): "Arg1(IType(Item::InteractionCategory))",
    ("GetDecisionWithKey", 1): "Arg1(IType(Item::Decision))",
    ("GetDoctrine", 1): "Arg1(IType(Item::Doctrine))",
    ("GetFaithByKey", 1): "Arg1(IType(Item::Faith))",
    ("GetFaithDoctrine", 1): "Arg1(IType(Item::Doctrine))",
    ("GetFocus", 1): "Arg1(IType(Item::Focus))",
    ("GetGeographicalRegion", 1): "Arg1(IType(Item::Region))",
    ("GetHolySite", 1): "Arg1(IType(Item::HolySite))",
    ("GetIllustration", 1): "Arg1(IType(Item::ScriptedIllustration))",
    ("GetLifestyle", 1): "Arg1(IType(Item::Lifestyle))",
    ("GetMaA", 1): "Arg1(IType(Item::MenAtArms))",
    ("GetModifier", 1): "Arg1(IType(Item::Modifier))",
    ("GetNickname", 1): "Arg1(IType(Item::Nickname))",
    ("GetPerk", 1): "Arg1(IType(Item::Perk))",
    ("GetRelation", 1): "Arg1(IType(Item::Relation))",
    ("GetReligionByKey", 1): "Arg1(IType(Item::Religion))",
    ("GetScheme", 1): "Arg1(IType(Item::Scheme))",
    ("GetSchemeType", 1): "Arg1(IType(Item::Scheme))",
    ("GetScriptedGui", 1): "Arg1(IType(Item::ScriptedGui))",
    ("GetScriptedRelation", 1): "Arg1(IType(Item::Relation))",
    ("GetSecretType", 1): "Arg1(IType(Item::Secret))",
    ("GetStaticModifier", 1): "Arg1(IType(Item::Modifier))",
    ("GetTerrain", 1): "Arg1(IType(Item::Terrain))",
    ("GetTitleByKey", 1): "Arg1(IType(Item::Title))",
    ("GetTrait", 1): "Arg1(IType(Item::Trait))",
    ("GetVassalStance", 1): "Arg1(IType(Item::VassalStance))",
    ("Scope.Var", 1): "Arg1(DType(CString))",
    ("TopScope.sC", 1): "Arg1(DType(CString))",
}

# This is for overriding argument types regardless of the datafunction's input type
ARGS_OVERRIDE_ANY = {
    ("Custom", 1): "Arg1(DType(CString))",
    ("GetName", 1): "Arg1(DType(Character))",
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
    "TopScope.sC": "Character",
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
        nargs = 0
        if "Arg0" in lines[0]:
            args = "Arg1(DType(Unknown))"
            nargs = 1
        if "Arg1" in lines[0]:
            args = "Arg2(DType(Unknown), DType(Unknown))"
            nargs = 2
        if "Arg2" in lines[0]:
            args = "Arg3(DType(Unknown), DType(Unknown), DType(Unknown))"
            nargs = 3
        if "Arg3" in lines[0]:
            args = "Arg4(DType(Unknown), DType(Unknown), DType(Unknown), DType(Unknown))"
            nargs = 4
        if "Arg4" in lines[0]:
            args = "Arg5(DType(Unknown), DType(Unknown), DType(Unknown), DType(Unknown), DType(Unknown))"
            nargs = 5
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
        if (name, nargs) in ARGS_OVERRIDE_ANY:
            args = ARGS_OVERRIDE_ANY[(name, nargs)]
        if "." in name:
            type, barename = name.split(".")
            if (barename, nargs) in ARGS_OVERRIDE_ANY:
                args = ARGS_OVERRIDE_ANY[(barename, nargs)]
        if (name, nargs) in ARGS_OVERRIDE:
            args = ARGS_OVERRIDE[(name, nargs)]

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
            if barename == "AccessSelf" or barename == "Self":
                rtype = type
            FUNCTIONS.append('    ("%s", %s, %s, %s),\n' % (barename, type, args, rtype))
        elif "\nDefinition type: Promote\n" in item:
            PROMOTES.append('    ("%s", %s, %s, %s),\n' % (barename, type, args, rtype))
        else:
            print(item)
            raise "unknown item"

with open(OUTDIR + "/datatypes.rs", "w") as outf:
    TYPES.sort()
    outf.write("#[derive(Copy, Clone, Debug, Eq, PartialEq, Display, EnumString)]\n")
    outf.write("pub enum Datatype {\n")
    outf.write("    Unknown,\n")
    outf.write("    AnyScope,\n")
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
