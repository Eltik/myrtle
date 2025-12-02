#!/usr/bin/env python3
"""
Auto-generate FlatBufferToJson implementations for all FlatBuffer types.

This script parses the generated Rust FlatBuffer code and creates
impl FlatBufferToJson blocks for each struct, mimicking Python's
dynamic reflection approach but at compile time.

Usage:
    python3 generate_fb_json_impls.py
"""

import re
from pathlib import Path
from dataclasses import dataclass
from typing import Optional

GENERATED_DIR = Path(__file__).parent / "src" / "generated_fbs"
OUTPUT_FILE = Path(__file__).parent / "src" / "fb_json_auto.rs"

@dataclass
class Field:
    name: str  # Rust method name (e.g., "skillId")
    return_type: str  # Return type (e.g., "Option<&'a str>", "i32", etc.)
    is_option: bool
    is_vector: bool
    is_enum: bool
    is_nested: bool
    element_type: Optional[str] = None  # For vectors, the element type

@dataclass
class Struct:
    name: str  # Full struct name (e.g., "clz_Torappu_SkillDataBundle")
    fields: list[Field]
    is_dict: bool  # Has Key/Value pattern
    is_root: bool  # Has GetRootAs function

def parse_struct_fields(content: str, struct_name: str) -> list[Field]:
    """Extract field accessor methods from a struct impl block."""
    fields = []

    # Find the impl block for this struct
    impl_pattern = rf"impl<'a>\s+{re.escape(struct_name)}<'a>\s*\{{"
    impl_match = re.search(impl_pattern, content)
    if not impl_match:
        return fields

    # Find all pub fn methods in this impl block
    start = impl_match.end()
    brace_count = 1
    end = start

    while brace_count > 0 and end < len(content):
        if content[end] == '{':
            brace_count += 1
        elif content[end] == '}':
            brace_count -= 1
        end += 1

    impl_content = content[start:end]

    # Extract pub fn methods that look like field accessors
    # Pattern: pub fn fieldName(&self) -> ReturnType {
    method_pattern = r'pub fn (\w+)\(&self\)\s*->\s*([^{]+?)\s*\{'

    for match in re.finditer(method_pattern, impl_content):
        method_name = match.group(1)
        return_type = match.group(2).strip()

        # Skip non-field methods
        skip_patterns = ['VT_', 'init_from_table', 'create', 'key_compare', 'unpack']
        if any(p in method_name for p in skip_patterns):
            continue
        if method_name.startswith('_'):
            continue
        if method_name.endswith('Length'):
            continue

        # Determine field characteristics
        is_option = return_type.startswith('Option<')
        is_vector = 'Vector<' in return_type
        is_enum = return_type.startswith('enum__')
        is_nested = (not is_enum and
                    'clz_' in return_type or
                    'dict__' in return_type)

        # Extract element type for vectors
        element_type = None
        if is_vector:
            # Extract inner type from Vector<'a, ForwardsUOffset<T>>
            inner_match = re.search(r'ForwardsUOffset<([^>]+)>', return_type)
            if inner_match:
                element_type = inner_match.group(1).strip()
                if element_type == "&'a str" or element_type == "&str":
                    element_type = "string"
                elif 'clz_' in element_type or 'dict__' in element_type:
                    element_type = "nested"
                elif 'enum__' in element_type:
                    element_type = "enum"

        fields.append(Field(
            name=method_name,
            return_type=return_type,
            is_option=is_option,
            is_vector=is_vector,
            is_enum=is_enum,
            is_nested=is_nested and not is_vector,
            element_type=element_type
        ))

    return fields

def parse_file(filepath: Path) -> list[Struct]:
    """Parse a generated Rust file and extract struct information."""
    content = filepath.read_text()
    structs = []

    # Find all struct definitions: pub struct name<'a> { pub _tab: ... }
    # Include clz_, dict__, list_, and kvp__ prefixed types
    struct_pattern = r"pub struct ((?:clz_|dict__|list_|kvp__)\w+)<'a>\s*\{\s*pub _tab:"

    for match in re.finditer(struct_pattern, content):
        struct_name = match.group(1)
        fields = parse_struct_fields(content, struct_name)

        # Check if it's a dict (has Key and Value methods)
        field_names = {f.name for f in fields}
        is_dict = 'key' in field_names and 'value' in field_names

        # Check if it has a root function
        root_pattern = rf"pub unsafe fn root_as_{struct_name.lower()}_unchecked"
        is_root = bool(re.search(root_pattern, content, re.IGNORECASE))

        # Include ALL structs, even empty ones (they're referenced by other types)
        structs.append(Struct(
            name=struct_name,
            fields=fields,
            is_dict=is_dict,
            is_root=is_root
        ))

    return structs

def generate_impl(struct: Struct, module_name: str) -> str:
    """Generate impl FlatBufferToJson for a struct."""
    lines = []
    lines.append(f"impl FlatBufferToJson for {module_name}::{struct.name}<'_> {{")
    lines.append("    fn to_json(&self) -> Value {")

    if struct.is_dict:
        # Dict pattern: return key-value pair
        # Find key and value fields to check their optionality
        key_field = next((f for f in struct.fields if f.name == 'key'), None)
        value_field = next((f for f in struct.fields if f.name == 'value'), None)

        lines.append("        let mut map = Map::new();")

        # Handle key field
        if key_field:
            if key_field.is_option:
                lines.append('        if let Some(k) = self.key() {')
                lines.append('            map.insert("key".to_string(), json!(k));')
                lines.append('        }')
            else:
                lines.append('        map.insert("key".to_string(), json!(self.key()));')

        # Handle value field
        if value_field:
            if value_field.is_option:
                if value_field.is_nested:
                    lines.append('        if let Some(v) = self.value() {')
                    lines.append('            map.insert("value".to_string(), v.to_json());')
                    lines.append('        }')
                elif value_field.is_vector:
                    if value_field.element_type == "nested":
                        lines.append('        if let Some(vec) = self.value() {')
                        lines.append('            let arr: Vec<Value> = (0..vec.len()).map(|i| vec.get(i).to_json()).collect();')
                        lines.append('            map.insert("value".to_string(), json!(arr));')
                        lines.append('        }')
                    else:
                        lines.append('        if let Some(vec) = self.value() {')
                        lines.append('            let arr: Vec<Value> = vec.iter().map(|v| json!(v)).collect();')
                        lines.append('            map.insert("value".to_string(), json!(arr));')
                        lines.append('        }')
                else:
                    lines.append('        if let Some(v) = self.value() {')
                    lines.append('            map.insert("value".to_string(), json!(v));')
                    lines.append('        }')
            else:
                # Non-optional value
                if value_field.is_nested:
                    lines.append('        map.insert("value".to_string(), self.value().to_json());')
                elif value_field.is_enum:
                    lines.append('        map.insert("value".to_string(), self.value().to_json_value());')
                else:
                    lines.append('        map.insert("value".to_string(), json!(self.value()));')

        lines.append("        Value::Object(map)")
    else:
        lines.append("        let mut map = Map::new();")

        for field in struct.fields:
            if field.name in ('key', 'value'):  # Skip if we're not a dict but have these
                continue

            pascal_name = field.name[0].upper() + field.name[1:]

            if field.is_vector:
                if field.element_type == "string":
                    lines.append(f'        if let Some(vec) = self.{field.name}() {{')
                    lines.append('            let arr: Vec<Value> = (0..vec.len()).map(|i| json!(vec.get(i))).collect();')
                    lines.append(f'            map.insert("{pascal_name}".to_string(), json!(arr));')
                    lines.append('        }')
                elif field.element_type == "nested":
                    lines.append(f'        if let Some(vec) = self.{field.name}() {{')
                    lines.append('            let arr: Vec<Value> = (0..vec.len()).map(|i| vec.get(i).to_json()).collect();')
                    lines.append(f'            map.insert("{pascal_name}".to_string(), json!(arr));')
                    lines.append('        }')
                elif field.element_type == "enum":
                    lines.append(f'        if let Some(vec) = self.{field.name}() {{')
                    lines.append('            let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();')
                    lines.append(f'            map.insert("{pascal_name}".to_string(), json!(arr));')
                    lines.append('        }')
                else:
                    # Unknown vector type - try as scalars
                    lines.append(f'        if let Some(vec) = self.{field.name}() {{')
                    lines.append('            let arr: Vec<Value> = vec.iter().map(|v| json!(v)).collect();')
                    lines.append(f'            map.insert("{pascal_name}".to_string(), json!(arr));')
                    lines.append('        }')
            elif field.is_enum:
                if field.is_option:
                    lines.append(f'        if let Some(e) = self.{field.name}() {{')
                    lines.append(f'            map.insert("{pascal_name}".to_string(), e.to_json_value());')
                    lines.append('        }')
                else:
                    lines.append(f'        map.insert("{pascal_name}".to_string(), self.{field.name}().to_json_value());')
            elif field.is_nested:
                lines.append(f'        if let Some(nested) = self.{field.name}() {{')
                lines.append(f'            map.insert("{pascal_name}".to_string(), nested.to_json());')
                lines.append('        }')
            elif field.is_option:
                # Optional string or other
                if "&'a str" in field.return_type or "&str" in field.return_type:
                    lines.append(f'        if let Some(s) = self.{field.name}() {{')
                    lines.append(f'            map.insert("{pascal_name}".to_string(), json!(s));')
                    lines.append('        }')
                else:
                    lines.append(f'        if let Some(v) = self.{field.name}() {{')
                    lines.append(f'            map.insert("{pascal_name}".to_string(), json!(v));')
                    lines.append('        }')
            else:
                # Scalar
                lines.append(f'        map.insert("{pascal_name}".to_string(), json!(self.{field.name}()));')

        lines.append("        Value::Object(map)")

    lines.append("    }")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)

def generate_enum_impl(enum_name: str, module_name: str) -> str:
    """Generate EnumToJson impl for an enum."""
    return f"""impl EnumToJson for {module_name}::{enum_name} {{
    fn to_json_value(&self) -> Value {{
        match self.variant_name() {{
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{{}}", self.0)),
        }}
    }}
}}
"""

def main():
    print("Scanning generated FlatBuffer files...")

    all_structs = {}  # module_name -> list of Struct
    all_enums = {}    # module_name -> list of enum names

    for filepath in sorted(GENERATED_DIR.glob("*_generated.rs")):
        module_name = filepath.stem  # e.g., "skill_table_generated"
        content = filepath.read_text()

        # Parse structs
        structs = parse_file(filepath)
        if structs:
            all_structs[module_name] = structs
            print(f"  {module_name}: {len(structs)} structs")

        # Find enums (both i32 and u8 underlying types)
        enum_pattern = r"pub struct (enum__\w+)\(pub (?:i32|u8)\)"
        enums = re.findall(enum_pattern, content)
        if enums:
            all_enums[module_name] = enums
            print(f"  {module_name}: {len(enums)} enums")

    # Collect all modules (those with enums OR structs)
    all_modules = set(all_enums.keys()) | set(all_structs.keys())

    # Generate output
    output_lines = [
        "//! Auto-generated FlatBufferToJson implementations",
        "//! Generated by generate_fb_json_impls.py",
        "//!",
        "//! DO NOT EDIT - regenerate with: python3 generate_fb_json_impls.py",
        "",
        "#![allow(unused_imports)]",
        "",
        "use crate::fb_json_macros::{FlatBufferToJson, EnumToJson};",
        "use serde_json::{json, Map, Value};",
        "",
        "// Import all generated modules",
    ]

    # Add all module imports at the top
    for module_name in sorted(all_modules):
        output_lines.append(f"use crate::generated_fbs::{module_name};")
    output_lines.append("")

    # Generate enum impls
    output_lines.append("// ============ Enum Implementations ============")
    output_lines.append("")
    for module_name, enums in sorted(all_enums.items()):
        output_lines.append(f"// From {module_name}")
        for enum_name in enums:
            output_lines.append(generate_enum_impl(enum_name, module_name))

    # Generate struct impls
    output_lines.append("")
    output_lines.append("// ============ Struct Implementations ============")
    output_lines.append("")
    for module_name, structs in sorted(all_structs.items()):
        output_lines.append(f"// From {module_name}")
        for struct in structs:
            output_lines.append(generate_impl(struct, module_name))

    # Write output
    OUTPUT_FILE.write_text("\n".join(output_lines))
    print(f"\nGenerated {OUTPUT_FILE}")
    print(f"  {sum(len(s) for s in all_structs.values())} struct impls")
    print(f"  {sum(len(e) for e in all_enums.values())} enum impls")

if __name__ == "__main__":
    main()
