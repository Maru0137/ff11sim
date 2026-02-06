#!/usr/bin/env python3
"""
Parse Windower/Resources Lua files and generate JSON for equipment search.

Usage:
    python parse_lua_to_json.py [--items ITEMS_LUA] [--descriptions DESC_LUA] [--output OUTPUT_JSON]
"""

import argparse
import json
import re
from pathlib import Path
from typing import Any

# FFXI Job bitmask definitions (22 jobs)
JOBS = {
    0: "WAR", 1: "MNK", 2: "WHM", 3: "BLM", 4: "RDM", 5: "THF",
    6: "PLD", 7: "DRK", 8: "BST", 9: "BRD", 10: "RNG", 11: "SAM",
    12: "NIN", 13: "DRG", 14: "SMN", 15: "BLU", 16: "COR", 17: "PUP",
    18: "DNC", 19: "SCH", 20: "GEO", 21: "RUN"
}

# FFXI Slot bitmask definitions
SLOTS = {
    0: "main", 1: "sub", 2: "range", 3: "ammo",
    4: "head", 5: "body", 6: "hands", 7: "legs",
    8: "feet", 9: "neck", 10: "waist", 11: "ear1",
    12: "ear2", 13: "ring1", 14: "ring2", 15: "back"
}

# FFXI Race bitmask definitions
RACES = {
    0: "Hum_M", 1: "Hum_F", 2: "Elv_M", 3: "Elv_F",
    4: "Tar_M", 5: "Tar_F", 6: "Mit_M", 7: "Mit_F",
    8: "Gal"
}


def bitmask_to_list(bitmask: int, mapping: dict[int, str]) -> list[str]:
    """Convert bitmask to list of values."""
    if bitmask is None:
        return []
    return [v for k, v in sorted(mapping.items()) if bitmask & (1 << k)]


def parse_lua_value(value_str: str) -> Any:
    """Parse a Lua value string to Python type."""
    value_str = value_str.strip()

    # String value
    if value_str.startswith('"') and value_str.endswith('"'):
        # Handle escaped quotes and special characters
        return value_str[1:-1].replace('\\"', '"')

    # Boolean
    if value_str == "true":
        return True
    if value_str == "false":
        return False

    # Number (int or float)
    try:
        if '.' in value_str:
            return float(value_str)
        return int(value_str)
    except ValueError:
        return value_str


def parse_lua_table(content: str) -> dict[int, dict[str, Any]]:
    """Parse Lua table format to Python dictionary."""
    items = {}

    # Match each table entry: [id] = {fields}
    # Handle multi-line entries by matching balanced braces
    pattern = r'\[(\d+)\]\s*=\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}'

    for match in re.finditer(pattern, content):
        item_id = int(match.group(1))
        fields_str = match.group(2)

        item = {'id': item_id}

        # Parse individual fields
        # Match: key=value or key="string with spaces"
        field_pattern = r'(\w+)\s*=\s*("(?:[^"\\]|\\.)*"|[^,}\]]+)'

        for field_match in re.finditer(field_pattern, fields_str):
            key = field_match.group(1)
            value = parse_lua_value(field_match.group(2))
            item[key] = value

        items[item_id] = item

    return items


def process_item(item: dict, descriptions: dict[int, dict]) -> dict:
    """Process a single item, expanding bitmasks and adding descriptions."""
    item_id = item.get('id')

    # Get description if available
    desc = descriptions.get(item_id, {})

    processed = {
        'id': item_id,
        'en': item.get('en', ''),
        'ja': item.get('ja', ''),
        'enl': item.get('enl', ''),
        'jal': item.get('jal', ''),
        'category': item.get('category', ''),
        'type': item.get('type'),
        'flags': item.get('flags'),
        'stack': item.get('stack', 1),
    }

    # Equipment-specific fields
    if 'level' in item:
        processed['level'] = item['level']

    if 'jobs' in item:
        processed['jobs'] = bitmask_to_list(item['jobs'], JOBS)

    if 'slots' in item:
        processed['slots'] = bitmask_to_list(item['slots'], SLOTS)

    if 'races' in item:
        processed['races'] = bitmask_to_list(item['races'], RACES)

    # Weapon-specific fields
    if 'damage' in item:
        processed['damage'] = item['damage']

    if 'delay' in item:
        processed['delay'] = item['delay']

    if 'skill' in item:
        processed['skill'] = item['skill']

    # Armor-specific fields
    if 'shield_size' in item:
        processed['shield_size'] = item['shield_size']

    # Description
    if desc:
        processed['description_en'] = desc.get('en', '')
        processed['description_ja'] = desc.get('ja', '')

    return processed


def main():
    parser = argparse.ArgumentParser(description='Parse Lua files to JSON')
    parser.add_argument('--items', type=Path, default=Path('items.lua'),
                        help='Path to items.lua')
    parser.add_argument('--descriptions', type=Path, default=Path('item_descriptions.lua'),
                        help='Path to item_descriptions.lua')
    parser.add_argument('--output', type=Path, default=Path('items.json'),
                        help='Output JSON path')
    args = parser.parse_args()

    print(f"Reading items from {args.items}...")
    with open(args.items, 'r', encoding='utf-8') as f:
        items_content = f.read()

    print(f"Reading descriptions from {args.descriptions}...")
    with open(args.descriptions, 'r', encoding='utf-8') as f:
        desc_content = f.read()

    print("Parsing items...")
    items = parse_lua_table(items_content)
    print(f"  Found {len(items)} items")

    print("Parsing descriptions...")
    descriptions = parse_lua_table(desc_content)
    print(f"  Found {len(descriptions)} descriptions")

    print("Processing items...")
    processed_items = []
    for item_id, item in sorted(items.items()):
        processed = process_item(item, descriptions)
        processed_items.append(processed)

    # Create output structure
    output = {
        'version': 1,
        'item_count': len(processed_items),
        'items': processed_items
    }

    print(f"Writing JSON to {args.output}...")
    with open(args.output, 'w', encoding='utf-8') as f:
        json.dump(output, f, ensure_ascii=False, separators=(',', ':'))

    print(f"Done! Generated {len(processed_items)} items")

    # Print some stats
    categories = {}
    for item in processed_items:
        cat = item.get('category', 'Unknown')
        categories[cat] = categories.get(cat, 0) + 1

    print("\nCategory breakdown:")
    for cat, count in sorted(categories.items(), key=lambda x: -x[1]):
        print(f"  {cat}: {count}")


if __name__ == '__main__':
    main()
