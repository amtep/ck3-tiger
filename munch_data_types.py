#!/usr/bin/python3

import os.path
import sys

SEPARATOR = "\n-----------------------\n\n"

# Vec4f is not listed as a datatype but there is a Select_vec4f function so it must exist.
TYPES = ['    Vec4f,\n']
GLOBAL_PROMOTES = []
GLOBAL_FUNCTIONS = []
FUNCTIONS = []
PROMOTES = []

# Experimentation shows that Root is accepted as well as ROOT
GLOBAL_PROMOTES.append('    ("Root", Args(&[]), Scope),\n')

# Most promotes and functions have 'Unknown' arg types in the data_type logs.
# This dictionary replaces those with known arg types in specific cases.
ARGS_OVERRIDE = {
    ("Artifact.GetFeatureText", 1): ["IType(Item::ArtifactFeatureGroup)"],
    ("Character.Custom2", 2): ["DType(CString)", "DType(AnyScope)"],
    ("EqualTo_string", 2): ["DType(CString)", "DType(CString)"],
    ("FaithDoctrine.GetName", 1): ["DType(Faith)"],
    ("FaithDoctrineGroup.GetName", 1): ["DType(Faith)"],
    ("GetAccoladeType", 1): ["IType(Item::AccoladeType)"],
    ("GetActivityGuestInviteRule", 1): ["IType(Item::GuestInviteRule)"],
    ("GetActivityIntent", 1): ["IType(Item::ActivityIntent)"],
    ("GetActivityLocale", 1): ["IType(Item::ActivityLocale)"],
    ("GetActivityPulseAction", 1): ["IType(Item::PulseAction)"],
    ("GetActivityType", 1): ["IType(Item::ActivityType)"],
    ("GetArtifactType", 1): ["IType(Item::ArtifactType)"],
    ("GetArtifactVisualType", 1): ["IType(Item::ArtifactVisual)"],
    ("GetBookmark", 1): ["IType(Item::Bookmark)"],
    ("GetBookmarkGroup", 1): ["IType(Item::BookmarkGroup)"],
    ("GetBuilding", 1): ["IType(Item::Building)"],
    ("GetCasusBelliType", 1): ["IType(Item::CasusBelli)"],
    ("GetCatalystType", 1): ["IType(Item::Catalyst)"],
    ("GetCharacterInteraction", 1): ["IType(Item::Interaction)"],
    ("GetCharacterInteractionCategory", 1): ["IType(Item::InteractionCategory)"],
    ("GetDecisionWithKey", 1): ["IType(Item::Decision)"],
    ("GetDoctrine", 1): ["IType(Item::Doctrine)"],
    ("GetFaithByKey", 1): ["IType(Item::Faith)"],
    ("GetFaithDoctrine", 1): ["IType(Item::Doctrine)"],
    ("GetFocus", 1): ["IType(Item::Focus)"],
    ("GetGeographicalRegion", 1): ["IType(Item::Region)"],
    ("GetHolySite", 1): ["IType(Item::HolySite)"],
    ("GetIllustration", 1): ["IType(Item::ScriptedIllustration)"],
    ("GetLifestyle", 1): ["IType(Item::Lifestyle)"],
    ("GetMaA", 1): ["IType(Item::MenAtArms)"],
    ("GetModifier", 1): ["IType(Item::Modifier)"],
    ("GetNickname", 1): ["IType(Item::Nickname)"],
    ("GetPerk", 1): ["IType(Item::Perk)"],
    ("GetRelation", 1): ["IType(Item::Relation)"],
    ("GetReligionByKey", 1): ["IType(Item::Religion)"],
    ("GetScheme", 1): ["IType(Item::Scheme)"],
    ("GetSchemeType", 1): ["IType(Item::Scheme)"],
    ("GetScriptedGui", 1): ["IType(Item::ScriptedGui)"],
    ("GetScriptedRelation", 1): ["IType(Item::Relation)"],
    ("GetSecretType", 1): ["IType(Item::Secret)"],
    ("GetStaticModifier", 1): ["IType(Item::Modifier)"],
    ("GetTerrain", 1): ["IType(Item::Terrain)"],
    ("GetTitleByKey", 1): ["IType(Item::Title)"],
    ("GetTrait", 1): ["IType(Item::Trait)"],
    ("GetVassalStance", 1): ["IType(Item::VassalStance)"],
    ("Scope.Var", 1): ["DType(CString)"],
    ("TopScope.sC", 1): ["DType(CString)"],

# For Vic3
    ("VariableSystem.Set", 1): ["DType(CString)", "DType(Unknown)"],
}

# This is for overriding argument types regardless of the datafunction's input type
ARGS_OVERRIDE_ANY = {
    ("Custom", 1): ["DType(CString)"],
    ("GetName", 1): ["DType(Character)"],
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
    "Scope.Accolade": "Accolade",
    "Scope.Activity": "Activity",
    "Scope.ActivityType": "ActivityType",
    "Scope.Army": "Army",
    "Scope.Artifact": "Artifact",
    "Scope.CasusBelli": "ActiveCasusBelli",
    "Scope.Char": "Character",
    "Scope.CharacterMemory": "CharacterMemory",
    "Scope.Combat": "Combat",
    "Scope.CombatSide": "CombatSide",
    "Scope.CouncilTask": "ActiveCouncilTask",
    "Scope.Culture": "Culture",
    "Scope.CulturePillar": "CulturePillar",
    "Scope.CultureTradition": "CultureTradition",
    "Scope.DecisionType": "Decision",
    "Scope.Faction": "Faction",
    "Scope.Faith": "Faith",
    "Scope.FaithDoctrine": "FaithDoctrine",
    "Scope.GetCharacter": "Character",
    "Scope.GetLandedTitle": "Title",
    "Scope.GetProvince": "Province",
    "Scope.GetScheme": "Scheme",
    "Scope.GovernmentType": "GovernmentType",
    "Scope.GreatHolyWar": "GreatHolyWar",
    "Scope.HolyOrder": "HolyOrder",
    "Scope.House": "DynastyHouse",
    "Scope.Inspiration": "Inspiration",
    "Scope.MercenaryCompany": "MercenaryCompany",
    "Scope.Province": "Province",
    "Scope.Religion": "Religion",
    "Scope.Scheme": "Scheme",
    "Scope.ScriptValue": "CFixedPoint", # this is a guess
    "Scope.Secret": "Secret",
    "Scope.Story": "Story",
    "Scope.Struggle": "Struggle",
    "Scope.Title": "Title",
    "Scope.Trait": "Trait",
    "Scope.TravelPlan": "TravelPlan",
    "Scope.VassalContractType": "VassalContractType",
    "Scope.War": "War",
    "TopScope.sC": "Character",
}

game = sys.argv[1]
fnames = sys.argv[2:]

if game == "ck3":
    OUTDIR = "src/ck3/tables/include"
elif game == "vic3":
    OUTDIR = "src/vic3/tables/include"
else:
    print("unknown game {}", game)

for fname in fnames:
    text = open(fname, encoding="windows-1252").read()
    items = text.split(SEPARATOR)

    for item in items:
        if not item:
            continue
        lines = item.splitlines()
        name = lines[0].split('(')[0]

        args = []
        nargs = 0
        while True:
            arg = "Arg{}".format(nargs)
            if arg not in lines[0]:
                break
            nargs = nargs + 1
            args.append("DType(Unknown)")

        for s in UNARY_ARGS:
            if name.startswith(s):
                type = name.split('_')[1]
                args = ["DType(%s)" % type]
        for s in BINARY_ARGS:
            if name.startswith(s):
                type = name.split('_')[1]
                args = ["DType(%s)" % type, "DType(%s)" % type]
        if name.startswith("Select_"):
            type = name.split('_')[1]
            args = ["DType(bool)", "DType(%s)" % type, "DType(%s)" % type]
        if (name, nargs) in ARGS_OVERRIDE_ANY:
            args = ARGS_OVERRIDE_ANY[(name, nargs)]
        if "." in name:
            type, barename = name.split(".")
            if (barename, nargs) in ARGS_OVERRIDE_ANY:
                args = ARGS_OVERRIDE_ANY[(barename, nargs)]
        if (name, nargs) in ARGS_OVERRIDE:
            args = ARGS_OVERRIDE[(name, nargs)]

        args = "Args(&[" + ", ".join(args) + "])"

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
