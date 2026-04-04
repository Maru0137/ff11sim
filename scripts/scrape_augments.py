#!/usr/bin/env python3
"""
Scrape augment data from FF11用語辞典 (wiki.ffo.jp) set pages and generate augments.json.

Usage:
    python3 scripts/scrape_augments.py

Each set page contains an augment table with columns for each body part (頭/胴/両手/両脚/両足).
Two table formats exist:
  - Single-table: [Rank, 頭, 胴, 両手, 両脚, 両足] (Atonement 3 sets)
  - Multi-table per Type: [Rank, Type:X] with sub-header [頭, 胴, 両手, 両脚, 両足] (Nyame)
"""

import json
import re
import time
import urllib.request
from html.parser import HTMLParser
from pathlib import Path

# Body part column name -> item name suffix mapping per set
# The set pages use 頭/胴/両手/両脚/両足 as column headers
SLOT_COLUMNS = ["頭", "胴", "両手", "両脚", "両足"]

# Set pages to scrape: (wiki_url, [item_name_ja for 頭, 胴, 両手, 両脚, 両足])
SET_PAGES = [
    # Ikenga set (Odyssey Atonement 3 - Xevioso)
    ("https://wiki.ffo.jp/html/38196.html", [
        "イケンガハット", "イケンガベスト", "イケンガグローブ",
        "イケンガトラウザ", "イケンガクロッグ",
    ]),
    # Gleti set (Odyssey Atonement 3 - Ngai)
    ("https://wiki.ffo.jp/html/38215.html", [
        "グレティマスク", "グレティキュイラス", "グレティガントレ",
        "グレティブリーチズ", "グレティブーツ",
    ]),
    # Mpaca set (Odyssey Atonement 3 - Arebati)
    ("https://wiki.ffo.jp/html/38216.html", [
        "ムパカキャップ", "ムパカダブレット", "ムパカグローブ",
        "ムパカホーズ", "ムパカブーツ",
    ]),
    # Bunzi set (Odyssey Atonement 3 - Mboze)
    ("https://wiki.ffo.jp/html/38228.html", [
        "ブンジハット", "ブンジローブ", "ブンジグローブ",
        "ブンジズボン", "ブンジサボ",
    ]),
    # Agwu set (Odyssey Atonement 3 - Ongo)
    ("https://wiki.ffo.jp/html/38229.html", [
        "アグゥキャップ", "アグゥローブ", "アグゥゲージ",
        "アグゥスロップス", "アグゥピガッシュ",
    ]),
    # Sakpata set (Odyssey Atonement 3 - Kalunga)
    ("https://wiki.ffo.jp/html/38230.html", [
        "サクパタヘルム", "サクパタブレスト", "サクパタガントレ",
        "サクパタクウィス", "サクパタレギンス",
    ]),
    # Nyame set (Odyssey Atonement 4 - Bumba)
    ("https://wiki.ffo.jp/html_2006/38283.html", [
        "ニャメヘルム", "ニャメメイル", "ニャメガントレ",
        "ニャメフランチャ", "ニャメソルレット",
    ]),
]


class SimpleTableParser(HTMLParser):
    """Parse all tables from HTML into lists of rows."""

    def __init__(self):
        super().__init__()
        self.in_table = False
        self.tables = []
        self.current_table = []
        self.current_row = []
        self.in_cell = False
        self.cell_text = ""

    def handle_starttag(self, tag, attrs):
        if tag == "table":
            self.in_table = True
            self.current_table = []
        elif tag == "tr" and self.in_table:
            self.current_row = []
        elif tag in ("td", "th") and self.in_table:
            self.in_cell = True
            self.cell_text = ""
        elif tag == "br" and self.in_cell:
            self.cell_text += "\n"

    def handle_endtag(self, tag):
        if tag == "table" and self.in_table:
            self.in_table = False
            self.tables.append(self.current_table)
        elif tag == "tr" and self.in_table:
            self.current_table.append(self.current_row)
        elif tag in ("td", "th") and self.in_cell:
            self.current_row.append(self.cell_text.strip())
            self.in_cell = False

    def handle_data(self, data):
        if self.in_cell:
            self.cell_text += data


def fetch_page(url):
    """Fetch a wiki page and return its HTML."""
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req) as resp:
        return resp.read().decode("utf-8", errors="replace")


def parse_augment_cell(cell_text):
    """Parse a single augment cell text into a clean Japanese stat string.

    Removes [1], [2], [3] markers and cleans up whitespace.
    """
    if not cell_text:
        return ""

    lines = []
    for line in cell_text.split("\n"):
        line = line.strip()
        if not line:
            continue
        line = re.sub(r"\[\d+\]", "", line).strip()
        line = re.sub(r"^\]+", "", line).strip()
        if line and line != "-" and line != "―":
            lines.append(line)

    return "\n".join(lines)


def parse_set_page(html):
    """Parse a set page and return augment data per body slot (index 0-4).

    Returns: dict mapping slot_index (0-4) to list of path dicts
             e.g. {0: [{"type": "Default", "ranks": [...]}], ...}

    Handles two formats:
    1. Single table: [Rank, 頭, 胴, 両手, 両脚, 両足]
    2. Multi-table (Nyame): multiple tables each with [Rank, Type:X] header
       and [頭, 胴, 両手, 両脚, 両足] sub-header
    """
    p = SimpleTableParser()
    p.feed(html)

    # Find augment tables (contain 'Rank' in first cell of first row)
    aug_tables = []
    for table in p.tables:
        if not table or len(table) < 2:
            continue
        first_row = table[0]
        if first_row and "Rank" in first_row[0]:
            aug_tables.append(table)

    if not aug_tables:
        return None

    # Check format by examining first augment table header
    first_header = aug_tables[0][0]

    # Format 1: Single table with slot columns
    # Header: [Rank, 頭, 胴, 両手, 両脚, 両足]
    if len(first_header) >= 6:
        return _parse_single_table(aug_tables[0])

    # Format 2: Multiple tables, one per Type
    # Each table header: [Rank, Type:X], second row: [頭, 胴, 両手, 両脚, 両足]
    if len(first_header) == 2 and "Type:" in first_header[1]:
        return _parse_multi_type_tables(aug_tables)

    return None


def _parse_single_table(table):
    """Parse single augment table with slot columns.

    Table format:
      Row 0: [Rank, 頭, 胴, 両手, 両脚, 両足]
      Row 1: meta row (skip)
      Row 2+: [rank_num, cell_頭, cell_胴, cell_両手, cell_両脚, cell_両足]
    """
    result = {}
    for row in table[1:]:
        if len(row) < 2:
            continue
        try:
            rank = int(row[0].strip())
        except ValueError:
            continue

        for slot_idx in range(5):
            col_idx = slot_idx + 1
            if col_idx >= len(row):
                break
            aug_text = parse_augment_cell(row[col_idx])
            if not aug_text:
                continue

            if slot_idx not in result:
                result[slot_idx] = {}
            if "Default" not in result[slot_idx]:
                result[slot_idx]["Default"] = []
            result[slot_idx]["Default"].append({"rank": rank, "text": aug_text})

    # Convert to output format
    output = {}
    for slot_idx, paths_dict in result.items():
        output[slot_idx] = [
            {"type": t, "ranks": ranks} for t, ranks in paths_dict.items()
        ]
    return output


def _parse_multi_type_tables(tables):
    """Parse multiple Type tables (Nyame format).

    Each table:
      Row 0: [Rank, Type:X]
      Row 1: [頭, 胴, 両手, 両脚, 両足]
      Row 2+: [rank_num, cell_頭, cell_胴, cell_両手, cell_両脚, cell_両足]
    """
    result = {}
    for table in tables:
        if len(table) < 3:
            continue
        header = table[0]
        if len(header) < 2 or "Type:" not in header[1]:
            continue
        type_name = header[1]

        for row in table[2:]:
            if len(row) < 2:
                continue
            try:
                rank = int(row[0].strip())
            except ValueError:
                continue

            for slot_idx in range(5):
                col_idx = slot_idx + 1
                if col_idx >= len(row):
                    break
                aug_text = parse_augment_cell(row[col_idx])
                if not aug_text:
                    continue

                if slot_idx not in result:
                    result[slot_idx] = {}
                if type_name not in result[slot_idx]:
                    result[slot_idx][type_name] = []
                result[slot_idx][type_name].append(
                    {"rank": rank, "text": aug_text}
                )

    # Convert to output format
    output = {}
    for slot_idx, paths_dict in result.items():
        output[slot_idx] = [
            {"type": t, "ranks": ranks} for t, ranks in paths_dict.items()
        ]
    return output


def load_items_json():
    """Load items.json and build a name -> id lookup."""
    items_path = Path(__file__).parent.parent / "web" / "data" / "items.json"
    with open(items_path) as f:
        data = json.load(f)
    items = data.get("items", [])
    name_to_id = {}
    for item in items:
        ja = item.get("ja", "")
        if ja:
            name_to_id[ja] = item["id"]
    return name_to_id


def main():
    name_to_id = load_items_json()
    augments = {}
    errors = []

    for url, item_names in SET_PAGES:
        set_name = item_names[0].rstrip("ハットヘルムマスクキャップ")
        print(f"Fetching: {set_name}装束 ({url})")

        try:
            html = fetch_page(url)
            slot_data = parse_set_page(html)

            if not slot_data:
                errors.append(f"  No augment table found: {set_name}装束")
                print(errors[-1])
                continue

            for slot_idx, paths in slot_data.items():
                if slot_idx >= len(item_names):
                    continue
                item_name = item_names[slot_idx]
                item_id = name_to_id.get(item_name)
                if not item_id:
                    errors.append(f"  Item not found in items.json: {item_name}")
                    print(errors[-1])
                    continue

                augments[str(item_id)] = {"paths": paths}
                total_ranks = sum(len(p["ranks"]) for p in paths)
                print(f"  {item_name} (ID {item_id}): "
                      f"{len(paths)} paths, {total_ranks} rank entries")

        except Exception as e:
            errors.append(f"  Error: {set_name}装束: {e}")
            print(errors[-1])

        time.sleep(1)

    # Write output
    output_path = Path(__file__).parent.parent / "web" / "data" / "augments.json"
    output = {"version": 1, "augments": augments}

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(output, f, ensure_ascii=False, indent=2)

    print(f"\nWritten {len(augments)} items to {output_path}")
    if errors:
        print(f"\nErrors ({len(errors)}):")
        for e in errors:
            print(e)


if __name__ == "__main__":
    main()
