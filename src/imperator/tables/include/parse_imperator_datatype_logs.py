def parse_and_format_input(input_string):
    lines = input_string.split("\n")

    current_class = ""
    output_list = []

    for line in lines:
        line = line.strip()

        if line.endswith("= {"):
            current_class = line.replace(" =", "").replace("{", "").strip()
        elif "->" in line:
            method, return_type = line.replace("}", "").split(" -> ")
            output_list.append(
                f'("{method}", {current_class}, Args(&[]), {return_type})'
            )

    return "\n".join(output_list)


input_string = """
AISettingsMenu = {
    AccessSelf -> Unknown
    Close -> void
    GetItems -> Unknown
    Reset -> void
    Save -> void
    Self -> Unknown
}

AISettingsMenuItem = {
    AccessSelf -> Unknown
    GetText -> CString
    GetTooltip -> CString
    GetValue -> bool
    Self -> Unknown
    SetValue -> void
}
"""

print(parse_and_format_input(input_string))
