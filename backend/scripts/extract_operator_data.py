#!/usr/bin/env python3
"""
Extract operator data from ArknightsDpsCompare for test generation.

This script parses the damage_formulas.py and JsonReader.py files to extract:
- Operator class names
- Character IDs
- Available skills and modules

Output: JSON file with operator configurations for DPS tests.
"""

import os
import sys
import re
import json

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BACKEND_DIR = os.path.dirname(SCRIPT_DIR)
ARKDPS_PATH = os.path.join(BACKEND_DIR, 'external', 'ArknightsDpsCompare')


def parse_id_dict():
    """Parse the id_dict from JsonReader.py to get operator name -> char_id mapping."""
    json_reader_path = os.path.join(ARKDPS_PATH, 'Database', 'JsonReader.py')

    with open(json_reader_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Find the id_dict definition (it spans multiple lines)
    match = re.search(r'id_dict\s*=\s*\{([^}]+)\}', content, re.DOTALL)
    if not match:
        print("Error: Could not find id_dict in JsonReader.py")
        return {}

    dict_content = match.group(1)

    # Parse the dictionary entries
    id_dict = {}
    # Match patterns like 'Name': 'char_xxx_yyy' or "Name": "char_xxx_yyy"
    pattern = r"['\"]([^'\"]+)['\"]\s*:\s*['\"]([^'\"]+)['\"]"
    for match in re.finditer(pattern, dict_content):
        name, char_id = match.groups()
        id_dict[name] = char_id

    return id_dict


def parse_damage_formulas():
    """Parse damage_formulas.py to extract operator class definitions."""
    damage_formulas_path = os.path.join(ARKDPS_PATH, 'damagecalc', 'damage_formulas.py')

    with open(damage_formulas_path, 'r', encoding='utf-8') as f:
        content = f.read()

    operators = {}

    # Find class definitions and their super().__init__ calls
    # Pattern: class OperatorName(Operator): ... super().__init__("Name", pp, [skills], [modules], ...)
    class_pattern = r'class\s+(\w+)\s*\(\s*Operator\s*\)\s*:'

    for class_match in re.finditer(class_pattern, content):
        class_name = class_match.group(1)
        class_start = class_match.end()

        # Find the next class definition or end of file
        next_class = re.search(r'\nclass\s+\w+\s*\(', content[class_start:])
        class_end = class_start + next_class.start() if next_class else len(content)
        class_body = content[class_start:class_end]

        # Find super().__init__ call
        init_pattern = r'super\(\)\.__init__\s*\(\s*["\']([^"\']+)["\']\s*,\s*pp\s*,\s*\[([^\]]*)\]\s*,\s*\[([^\]]*)\]'
        init_match = re.search(init_pattern, class_body)

        if init_match:
            display_name = init_match.group(1)
            skills_str = init_match.group(2).strip()
            modules_str = init_match.group(3).strip()

            # Parse skills list
            if skills_str:
                skills = [int(s.strip()) for s in skills_str.split(',') if s.strip()]
            else:
                skills = []

            # Parse modules list
            if modules_str:
                modules = [int(m.strip()) for m in modules_str.split(',') if m.strip()]
            else:
                modules = []

            # Skip template classes
            if class_name in ['NewBlueprint', 'NewBlueprintAoe', 'NewBlueprintArts']:
                continue

            operators[class_name] = {
                'class_name': class_name,
                'display_name': display_name,
                'skills': skills,
                'modules': modules,
            }

    return operators


def to_upper_camel_case(name):
    """Capitalize the first letter to match Rust struct naming (e.g., twelveF -> TwelveF)."""
    if not name:
        return name
    return name[0].upper() + name[1:]


def to_snake_case(name):
    """Convert PascalCase or camelCase to snake_case."""
    # Handle special cases
    if name == '12F':
        return 'twelve_f'
    if name == 'W':
        return 'w'

    result = []
    for i, c in enumerate(name):
        if c.isupper():
            if i > 0:
                prev = name[i-1]
                if not prev.isupper() and prev != '_':
                    result.append('_')
            result.append(c.lower())
        elif c.isdigit():
            if i > 0 and name[i-1].isalpha():
                result.append('_')
            result.append(c)
        else:
            result.append(c)

    return ''.join(result)


def main():
    # Parse data from ArknightsDpsCompare
    print("Parsing JsonReader.py for id_dict...", file=sys.stderr)
    id_dict = parse_id_dict()
    print(f"  Found {len(id_dict)} operator IDs", file=sys.stderr)

    print("Parsing damage_formulas.py for operator classes...", file=sys.stderr)
    operators = parse_damage_formulas()
    print(f"  Found {len(operators)} operator classes", file=sys.stderr)

    # Merge data
    result = []
    for class_name, op_data in sorted(operators.items()):
        display_name = op_data['display_name']

        # Try to find character ID
        char_id = None
        # Try display_name first, then class_name
        for name in [display_name, class_name]:
            if name in id_dict:
                char_id = id_dict[name]
                break

        if not char_id:
            print(f"  Warning: No char_id found for {class_name} ({display_name})", file=sys.stderr)
            continue

        # Skip operators with no skills (like 12F variants)
        if not op_data['skills']:
            # Use skill 0 as base
            op_data['skills'] = [0]

        # Determine default module
        default_module = op_data['modules'][0] if op_data['modules'] else -1

        result.append({
            'class_name': to_upper_camel_case(class_name),
            'display_name': display_name,
            'char_id': char_id,
            'rust_module': to_snake_case(class_name),
            'skills': op_data['skills'],
            'modules': op_data['modules'],
            'default_module': default_module,
        })

    print(f"Generated data for {len(result)} operators", file=sys.stderr)

    # Output as JSON
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
