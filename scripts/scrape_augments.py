#!/usr/bin/env python3
"""
Scrape augment data from FF11用語辞典 (wiki.ffo.jp) and generate augments.json.

Usage:
    python3 scripts/scrape_augments.py

Supports multiple page formats:
  - Set pages (装束): single table or multi-Type tables with body part columns
  - Individual item pages: simple [Rank, 追加性能] table
  - Limbus set pages: [Rank, オーグメント, 必要RP, Total RP] with per-slot values (Rank 30 only)
"""

import json
import re
import time
import urllib.request
from html.parser import HTMLParser
from pathlib import Path

# ---------------------------------------------------------------------------
# Data: pages to scrape
# ---------------------------------------------------------------------------

# Set pages: (url, [item_names for 頭, 胴, 両手, 両脚, 両足])
SET_PAGES = [
    # Ikenga (Odyssey AT3 - Xevioso)
    ("https://wiki.ffo.jp/html/38196.html", [
        "イケンガハット", "イケンガベスト", "イケンガグローブ",
        "イケンガトラウザ", "イケンガクロッグ",
    ]),
    # Gleti (Odyssey AT3 - Ngai)
    ("https://wiki.ffo.jp/html/38215.html", [
        "グレティマスク", "グレティキュイラス", "グレティガントレ",
        "グレティブリーチズ", "グレティブーツ",
    ]),
    # Mpaca (Odyssey AT3 - Arebati)
    ("https://wiki.ffo.jp/html/38216.html", [
        "ムパカキャップ", "ムパカダブレット", "ムパカグローブ",
        "ムパカホーズ", "ムパカブーツ",
    ]),
    # Bunzi (Odyssey AT3 - Mboze)
    ("https://wiki.ffo.jp/html/38228.html", [
        "ブンジハット", "ブンジローブ", "ブンジグローブ",
        "ブンジズボン", "ブンジサボ",
    ]),
    # Agwu (Odyssey AT3 - Ongo)
    ("https://wiki.ffo.jp/html/38229.html", [
        "アグゥキャップ", "アグゥローブ", "アグゥゲージ",
        "アグゥスロップス", "アグゥピガッシュ",
    ]),
    # Sakpata (Odyssey AT3 - Kalunga)
    ("https://wiki.ffo.jp/html/38230.html", [
        "サクパタヘルム", "サクパタブレスト", "サクパタガントレ",
        "サクパタクウィス", "サクパタレギンス",
    ]),
    # Nyame (Odyssey AT4 - Bumba)
    ("https://wiki.ffo.jp/html_2006/38283.html", [
        "ニャメヘルム", "ニャメメイル", "ニャメガントレ",
        "ニャメフランチャ", "ニャメソルレット",
    ]),
]

# Limbus set pages: (url, [[NQ names], [HQ1 names], [HQ2 names], ...])
# Augment table has per-slot values in [2][3] columns; only Rank 30 is extracted.
LIMBUS_SET_PAGES = [
    # ホープ装束
    ("https://wiki.ffo.jp/html/39949.html", [
        ["ホープマスク", "ホーププレート", "ホープガントレ", "ホープブレー", "ホープサバトン"],
        ["パーフェクマスク", "パーフェクプレート", "パーフェクガントレ", "パーフェクブレー", "パーフェクサバトン"],
        ["レベレマスク", "レベレプレート", "レベレガントレ", "レベレブレー", "レベレサバトン"],
    ]),
    # ジャスト装束
    ("https://wiki.ffo.jp/html/39950.html", [
        ["ジャストクラウン", "ジャストシクラス", "ジャストガントレ", "ジャストフランチャ", "ジャストソルレット"],
    ]),
    # トラスト装束
    ("https://wiki.ffo.jp/html/39951.html", [
        ["トラストクラウン", "トラストプレート", "トラストガントレ", "トラストブレー", "トラストサバトン"],
    ]),
    # 慈悲装束
    ("https://wiki.ffo.jp/html/39952.html", [
        ["慈悲総面", "慈悲腹巻", "慈悲篭手", "慈悲膝甲", "慈悲脛当"],
    ]),
]

# Individual item pages: (url, item_name_ja)
INDIVIDUAL_PAGES = [
    # Odyssey AT1
    ("https://wiki.ffo.jp/html/38138.html", "ヘスペリデ"),
    ("https://wiki.ffo.jp/html/38140.html", "エピタフサシェ"),
    ("https://wiki.ffo.jp/html/38141.html", "Ｎ．ストリンガー"),
    ("https://wiki.ffo.jp/html/38139.html", "コイストボダー"),
    # Odyssey AT2
    ("https://wiki.ffo.jp/html/38142.html", "アクロンティカ"),
    ("https://wiki.ffo.jp/html/38145.html", "ベーシルリング"),
    ("https://wiki.ffo.jp/html/38143.html", "鶴"),
    ("https://wiki.ffo.jp/html/38144.html", "シェレピアス"),
    ("https://wiki.ffo.jp/html/38146.html", "テレンベルト"),
    ("https://wiki.ffo.jp/html/38147.html", "オブシテナサッシュ"),
    # RMEA - Relic Weapons
    ("https://wiki.ffo.jp/html/1444.html", "スファライ"),
    ("https://wiki.ffo.jp/html/1478.html", "マンダウ"),
    ("https://wiki.ffo.jp/html/1479.html", "エクスカリバー"),
    ("https://wiki.ffo.jp/html/30953.html", "ラグナロク"),
    ("https://wiki.ffo.jp/html/1615.html", "ガトラー"),
    ("https://wiki.ffo.jp/html/1616.html", "ブラビューラ"),
    ("https://wiki.ffo.jp/html/3075.html", "アポカリプス"),
    ("https://wiki.ffo.jp/html/2398.html", "グングニル"),
    ("https://wiki.ffo.jp/html/3076.html", "鬼哭"),
    ("https://wiki.ffo.jp/html/3077.html", "天の村雲"),
    ("https://wiki.ffo.jp/html/2402.html", "ミョルニル"),
    ("https://wiki.ffo.jp/html/3078.html", "クラウストルム"),
    ("https://wiki.ffo.jp/html/3079.html", "与一の弓"),
    ("https://wiki.ffo.jp/html/3080.html", "アナイアレイター"),
    # RMEA - Mythic Weapons
    ("https://wiki.ffo.jp/html/15177.html", "グランツファウスト"),
    ("https://wiki.ffo.jp/html/15189.html", "乾坤圏"),
    ("https://wiki.ffo.jp/html/15185.html", "ヴァジュラ"),
    ("https://wiki.ffo.jp/html/15186.html", "カルンウェナン"),
    ("https://wiki.ffo.jp/html/15190.html", "テルプシコラー"),
    ("https://wiki.ffo.jp/html/15184.html", "ミュルグレス"),
    ("https://wiki.ffo.jp/html/15179.html", "ブルトガング"),
    ("https://wiki.ffo.jp/html/15176.html", "ティソーナ"),
    ("https://wiki.ffo.jp/html/15183.html", "アイムール"),
    ("https://wiki.ffo.jp/html/15173.html", "コンカラー"),
    ("https://wiki.ffo.jp/html/15180.html", "リベレーター"),
    ("https://wiki.ffo.jp/html/15174.html", "竜の髭"),
    ("https://wiki.ffo.jp/html/15181.html", "凪"),
    ("https://wiki.ffo.jp/html/15175.html", "小鴉丸"),
    ("https://wiki.ffo.jp/html/15182.html", "ヤグルシュ"),
    ("https://wiki.ffo.jp/html/15178.html", "レーヴァテイン"),
    ("https://wiki.ffo.jp/html/15188.html", "ニルヴァーナ"),
    ("https://wiki.ffo.jp/html/15191.html", "トゥプシマティ"),
    ("https://wiki.ffo.jp/html/15187.html", "ガストラフェテス"),
    ("https://wiki.ffo.jp/html/483.html", "デスペナルティ"),
    # RMEA - Empyrean Weapons
    ("https://wiki.ffo.jp/html/19984.html", "ウルスラグナ"),
    ("https://wiki.ffo.jp/html/20291.html", "トゥワシュトラ"),
    ("https://wiki.ffo.jp/html/20292.html", "アルマス"),
    ("https://wiki.ffo.jp/html/20294.html", "カラドボルグ"),
    ("https://wiki.ffo.jp/html/20295.html", "ファルシャ"),
    ("https://wiki.ffo.jp/html/20296.html", "ウコンバサラ"),
    ("https://wiki.ffo.jp/html/20297.html", "リデンプション"),
    ("https://wiki.ffo.jp/html/20298.html", "ロンゴミアント"),
    ("https://wiki.ffo.jp/html/20299.html", "神無"),
    ("https://wiki.ffo.jp/html/5237.html", "正宗"),
    ("https://wiki.ffo.jp/html/20300.html", "ガンバンテイン"),
    ("https://wiki.ffo.jp/html/20301.html", "フヴェルゲルミル"),
    ("https://wiki.ffo.jp/html/20383.html", "ガーンデーヴァ"),
    ("https://wiki.ffo.jp/html/20384.html", "アルマゲドン"),
    # RMEA - Ergon Weapons
    ("https://wiki.ffo.jp/html/31311.html", "エピオラトリー"),
    ("https://wiki.ffo.jp/html/31326.html", "イドリス"),
    # RMEA - Aeonic Weapons
    ("https://wiki.ffo.jp/html/35345.html", "ゴッドハンド"),
    ("https://wiki.ffo.jp/html/35349.html", "エーネアス"),
    ("https://wiki.ffo.jp/html/35346.html", "セクエンス"),
    ("https://wiki.ffo.jp/html/12434.html", "ライオンハート"),
    ("https://wiki.ffo.jp/html/35358.html", "トライエッジ"),
    ("https://wiki.ffo.jp/html/35347.html", "シャンゴル"),
    ("https://wiki.ffo.jp/html/35350.html", "トリシューラ"),
    ("https://wiki.ffo.jp/html/35348.html", "アングータ"),
    ("https://wiki.ffo.jp/html/35351.html", "丙子椒林剣"),
    ("https://wiki.ffo.jp/html/35352.html", "童子切安綱"),
    ("https://wiki.ffo.jp/html/35354.html", "ティシュトライヤ"),
    ("https://wiki.ffo.jp/html/35353.html", "カトヴァンガ"),
    ("https://wiki.ffo.jp/html/35357.html", "アキヌフォート"),
    ("https://wiki.ffo.jp/html/35356.html", "フォーマルハウト"),
    # Unity Wanted - 色褪せた鱗 (CL119/122)
    ("https://wiki.ffo.jp/html/33864.html", "峨嵋刺改"),
    ("https://wiki.ffo.jp/html/32194.html", "ブラアーム+1"),
    ("https://wiki.ffo.jp/html/32497.html", "丹頂改"),
    ("https://wiki.ffo.jp/html/32612.html", "国宗改"),
    ("https://wiki.ffo.jp/html/32499.html", "ポウェンワ+1"),
    ("https://wiki.ffo.jp/html/32215.html", "エヴァラック+1"),
    ("https://wiki.ffo.jp/html/32500.html", "メンガド+1"),
    ("https://wiki.ffo.jp/html/17935.html", "ウィングカッター+1"),
    ("https://wiki.ffo.jp/html/33293.html", "リファイグリップ+1"),
    ("https://wiki.ffo.jp/html/32204.html", "聖帝羽虫の髪飾り+1"),
    ("https://wiki.ffo.jp/html/33296.html", "アゴニジャーキン+1"),
    ("https://wiki.ffo.jp/html/32692.html", "ルーグラクローク+1"),
    ("https://wiki.ffo.jp/html/32489.html", "ロゼトジャズラン+1"),
    ("https://wiki.ffo.jp/html/32222.html", "マカブルガントレ+1"),
    ("https://wiki.ffo.jp/html/32224.html", "時雨手甲改"),
    ("https://wiki.ffo.jp/html/32622.html", "ゾアサブリガ+1"),
    ("https://wiki.ffo.jp/html/32493.html", "アシドゥイズボン+1"),
    ("https://wiki.ffo.jp/html/32228.html", "オーグリクウィス+1"),
    ("https://wiki.ffo.jp/html/32695.html", "ヒポメネソックス+1"),
    ("https://wiki.ffo.jp/html/32235.html", "リーガルパンプス+1"),
    ("https://wiki.ffo.jp/html/32623.html", "アンムーヴカラー+1"),
    ("https://wiki.ffo.jp/html/32259.html", "カントネックレス+1"),
    ("https://wiki.ffo.jp/html/32494.html", "神術帯+1"),
    ("https://wiki.ffo.jp/html/32263.html", "アキュイテベルト+1"),
    ("https://wiki.ffo.jp/html/32262.html", "セールフィベルト+1"),
    ("https://wiki.ffo.jp/html/32697.html", "ルーグラピアス+1"),
    ("https://wiki.ffo.jp/html/32495.html", "ナーリシュピアス+1"),
    ("https://wiki.ffo.jp/html/32496.html", "アレテデルルナ+1"),
    ("https://wiki.ffo.jp/html/32267.html", "ハンドラーピアス+1"),
    ("https://wiki.ffo.jp/html/33894.html", "ゼラチナスリング+1"),
    ("https://wiki.ffo.jp/html/32696.html", "アペリエリング+1"),
    ("https://wiki.ffo.jp/html/32624.html", "メタモルリング+1"),
    # Unity Wanted - 色褪せた皮 (CL125/128)
    ("https://wiki.ffo.jp/html/33285.html", "フューリーフィスト+1"),
    ("https://wiki.ffo.jp/html/35435.html", "クスタウィ+1"),
    ("https://wiki.ffo.jp/html/33286.html", "ターニオンダガー+1"),
    ("https://wiki.ffo.jp/html/32672.html", "アナセマハルパー+1"),
    ("https://wiki.ffo.jp/html/32606.html", "ユーゴククリ+1"),
    ("https://wiki.ffo.jp/html/32191.html", "サンガリアス+1"),
    ("https://wiki.ffo.jp/html/32192.html", "プクラトムージュ+1"),
    ("https://wiki.ffo.jp/html/33879.html", "デマサルデーゲン+1"),
    ("https://wiki.ffo.jp/html/35436.html", "ウシェンジ+1"),
    ("https://wiki.ffo.jp/html/33301.html", "クラデネツ+1"),
    ("https://wiki.ffo.jp/html/33882.html", "ンドモアクス+1"),
    ("https://wiki.ffo.jp/html/33287.html", "ペルーン+1"),
    ("https://wiki.ffo.jp/html/33288.html", "アイズコラブージ+1"),
    ("https://wiki.ffo.jp/html/33289.html", "ビヘッダー+1"),
    ("https://wiki.ffo.jp/html/32673.html", "トリスカサイズ+1"),
    ("https://wiki.ffo.jp/html/35437.html", "ピクイズパン+1"),
    ("https://wiki.ffo.jp/html/33290.html", "ゲイダーグ+1"),
    ("https://wiki.ffo.jp/html/33883.html", "雷鳥改"),
    ("https://wiki.ffo.jp/html/34639.html", "則房改"),
    ("https://wiki.ffo.jp/html/34640.html", "ロクソテクメイス+1"),
    ("https://wiki.ffo.jp/html/32280.html", "メイジスマッシャ+1"),
    ("https://wiki.ffo.jp/html/32614.html", "マランスタッフ+1"),
    ("https://wiki.ffo.jp/html/32196.html", "アバブリニ+1"),
    ("https://wiki.ffo.jp/html/33298.html", "エージャックス+1"),
    ("https://wiki.ffo.jp/html/33299.html", "デリベレンス+1"),
    ("https://wiki.ffo.jp/html/33292.html", "ガストリタスラム+1"),
    ("https://wiki.ffo.jp/html/34644.html", "シーズボムレット+1"),
    ("https://wiki.ffo.jp/html/33291.html", "ポローマボウ+1"),
    ("https://wiki.ffo.jp/html/35438.html", "イマーティ+1"),
    ("https://wiki.ffo.jp/html/32618.html", "リガレスグリップ+1"),
    ("https://wiki.ffo.jp/html/34646.html", "ブリスタサリット+1"),
    ("https://wiki.ffo.jp/html/32463.html", "アドーンドヘルム+1"),
    ("https://wiki.ffo.jp/html/32619.html", "スティンガヘルム+1"),
    ("https://wiki.ffo.jp/html/33294.html", "ヒーケカット+1"),
    ("https://wiki.ffo.jp/html/33295.html", "アルハゼンハット+1"),
    ("https://wiki.ffo.jp/html/33895.html", "コーホトクローク+1"),
    ("https://wiki.ffo.jp/html/32490.html", "エメットハーネス+1"),
    ("https://wiki.ffo.jp/html/32491.html", "比売胴丸改"),
    ("https://wiki.ffo.jp/html/32620.html", "唱門師浄衣改"),
    ("https://wiki.ffo.jp/html/32693.html", "徒武者の篭手改"),
    ("https://wiki.ffo.jp/html/33902.html", "アステリアミトン+1"),
    ("https://wiki.ffo.jp/html/33903.html", "ラマスミトン+1"),
    ("https://wiki.ffo.jp/html/35439.html", "ガズブレスレット+1"),
    ("https://wiki.ffo.jp/html/32694.html", "ハイゲアクロッグ+1"),
    ("https://wiki.ffo.jp/html/32236.html", "ジュートブーツ+1"),
    ("https://wiki.ffo.jp/html/32492.html", "ウォーダチャーム+1"),
    ("https://wiki.ffo.jp/html/33907.html", "バーシチョーカー+1"),
    ("https://wiki.ffo.jp/html/33913.html", "ケンタークベルト+1"),
    ("https://wiki.ffo.jp/html/33910.html", "ズワゾピアス+1"),
    ("https://wiki.ffo.jp/html/35440.html", "オノワイヤリング+1"),
    ("https://wiki.ffo.jp/html/34650.html", "カコエシクリング+1"),
    ("https://wiki.ffo.jp/html/33297.html", "メフィタスリング+1"),
    ("https://wiki.ffo.jp/html/32212.html", "グラウンドマント+1"),
    ("https://wiki.ffo.jp/html/33912.html", "フィフォレケープ+1"),
    ("https://wiki.ffo.jp/html/34649.html", "オリストケープ+1"),
    # Unity Wanted - 色褪せた羽 (CL135/145)
    ("https://wiki.ffo.jp/html/35102.html", "カマッパンス+1"),
    ("https://wiki.ffo.jp/html/35103.html", "タンモガイ+1"),
    ("https://wiki.ffo.jp/html/35104.html", "フリッサ+1"),
    ("https://wiki.ffo.jp/html/34637.html", "コンバスター+1"),
    ("https://wiki.ffo.jp/html/35105.html", "モンタント+1"),
    ("https://wiki.ffo.jp/html/34638.html", "ナリス+1"),
    ("https://wiki.ffo.jp/html/35106.html", "ハビリテイター+1"),
    ("https://wiki.ffo.jp/html/35107.html", "セプトプテック+1"),
    ("https://wiki.ffo.jp/html/35108.html", "コンテムプレータ+1"),
    ("https://wiki.ffo.jp/html/34642.html", "マリソン+1"),
    ("https://wiki.ffo.jp/html/34643.html", "アンチテイル+1"),
    ("https://wiki.ffo.jp/html/34641.html", "フォフェンド+1"),
    ("https://wiki.ffo.jp/html/34645.html", "ロースバルブータ+1"),
    ("https://wiki.ffo.jp/html/34647.html", "オビエキュイラス+1"),
    ("https://wiki.ffo.jp/html/35109.html", "楯無腹巻改"),
    ("https://wiki.ffo.jp/html/35110.html", "楯無篭手改"),
    ("https://wiki.ffo.jp/html/35112.html", "楯無佩楯改"),
    ("https://wiki.ffo.jp/html/35113.html", "楯無脛当改"),
    ("https://wiki.ffo.jp/html/34648.html", "ロリケートトルク+1"),
    ("https://wiki.ffo.jp/html/35114.html", "ヴィムトルク+1"),
    ("https://wiki.ffo.jp/html/34878.html", "ドミネンスピアス+1"),
]

# ---------------------------------------------------------------------------
# HTML Parser
# ---------------------------------------------------------------------------

SLOT_NAMES = ["頭", "胴", "両手", "両脚", "両足"]


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


def parse_tables(html):
    """Parse all tables from HTML."""
    p = SimpleTableParser()
    p.feed(html)
    return p.tables


def parse_augment_cell(cell_text):
    """Remove [1], [2], [3] markers and clean up."""
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


# ---------------------------------------------------------------------------
# Set page parsers (Odyssey AT3/AT4)
# ---------------------------------------------------------------------------

def parse_set_page(html):
    """Parse a set page → dict mapping slot_index (0-4) to paths list."""
    tables = parse_tables(html)
    aug_tables = [t for t in tables if t and t[0] and "Rank" in t[0][0]]

    if not aug_tables:
        return None

    first_header = aug_tables[0][0]

    # Single table: [Rank, 頭, 胴, 両手, 両脚, 両足]
    if len(first_header) >= 6:
        return _parse_single_table(aug_tables[0])

    # Multi-table per Type: [Rank, Type:X]
    if len(first_header) == 2 and "Type:" in first_header[1]:
        return _parse_multi_type_tables(aug_tables)

    return None


def _parse_single_table(table):
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

    return {si: [{"type": t, "ranks": r} for t, r in pd.items()]
            for si, pd in result.items()} or None


def _parse_multi_type_tables(tables):
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

    return {si: [{"type": t, "ranks": r} for t, r in pd.items()]
            for si, pd in result.items()} or None


# ---------------------------------------------------------------------------
# Individual item page parser (AT1/AT2, RMEA, Unity Wanted)
# ---------------------------------------------------------------------------

def parse_individual_page(html):
    """Parse individual item page → paths list or None.

    Table formats:
      - [Rank, 追加性能] (AT1/AT2, Unity Wanted)
      - [Rank, オーグメント, 必要RP, Total RP] (RMEA)
    """
    tables = parse_tables(html)

    for table in tables:
        if not table or len(table) < 2:
            continue
        header = table[0]
        if not header or "Rank" not in header[0]:
            continue

        # Augment data is in column 1 (追加性能 or オーグメント)
        # For RMEA 4-col format, columns 1-3 may each have augment lines
        ranks = []
        for row in table[1:]:
            if len(row) < 2:
                continue
            try:
                rank = int(row[0].strip())
            except ValueError:
                continue

            # For multi-column augment format (RMEA), combine all aug columns
            if len(header) >= 4 and "RP" in header[2]:
                # Columns 1..N-2 are augment data (skip RP columns)
                parts = []
                for col_idx in range(1, len(row) - 2):
                    cell = parse_augment_cell(row[col_idx])
                    if cell:
                        parts.append(cell)
                aug_text = "\n".join(parts)
            else:
                aug_text = parse_augment_cell(row[1])

            if aug_text:
                ranks.append({"rank": rank, "text": aug_text})

        if ranks:
            return [{"type": "Default", "ranks": ranks}]

    return None


# ---------------------------------------------------------------------------
# Limbus set page parser (Rank 30 only, per-slot values in [2][3])
# ---------------------------------------------------------------------------

def _parse_limbus_slot_values(cell_text):
    """Parse '頭+7% 胴+8% 両手+6% 両脚+7% 両足+6%' → {0: '+7%', 1: '+8%', ...}."""
    result = {}
    for i, slot_name in enumerate(SLOT_NAMES):
        m = re.search(rf"{slot_name}([+\-][\d?]+%?)", cell_text)
        if m:
            result[i] = m.group(1)
    return result


def parse_limbus_set_page(html):
    """Parse Limbus set page → dict mapping slot_index (0-4) to paths list.

    Only extracts Rank 30 data. [2] and [3] contain per-slot values.
    """
    tables = parse_tables(html)

    # Find the last Rank table (some pages have multiple)
    aug_table = None
    for table in tables:
        if not table or len(table) < 2:
            continue
        if table[0] and "Rank" in table[0][0]:
            aug_table = table

    if not aug_table:
        return None

    # Find Rank 30 row
    rank30_row = None
    for row in aug_table:
        if row and row[0].strip() == "30":
            rank30_row = row
            break

    if not rank30_row or len(rank30_row) < 4:
        return None

    # [1] is common to all slots, [2] and [3] have per-slot values
    common_text = parse_augment_cell(rank30_row[1])

    result = {}
    for slot_idx in range(5):
        parts = []
        if common_text:
            parts.append(common_text)

        # Parse [2] and [3] for this slot's specific value
        for col_idx in [2, 3]:
            if col_idx >= len(rank30_row):
                break
            cell = rank30_row[col_idx]
            if not cell:
                continue
            # First line is the stat name, subsequent lines have slot-specific values
            cell_lines = cell.split("\n")
            stat_name = cell_lines[0].strip()
            if not stat_name:
                continue
            # Try to find slot-specific value
            slot_values = _parse_limbus_slot_values(cell)
            if slot_idx in slot_values:
                val = slot_values[slot_idx]
                parts.append(f"{stat_name}{val}")

        if parts:
            aug_text = "\n".join(parts)
            result[slot_idx] = [{"type": "Default",
                                 "ranks": [{"rank": 30, "text": aug_text}]}]

    return result or None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def load_items_json():
    """Load items.json and build a name -> id lookup."""
    items_path = Path(__file__).parent.parent / "web" / "data" / "items.json"
    with open(items_path) as f:
        data = json.load(f)
    name_to_id = {}
    for item in data.get("items", []):
        ja = item.get("ja", "")
        if ja:
            name_to_id[ja] = item["id"]
    return name_to_id


def register_item(augments, name_to_id, item_name, paths, errors):
    """Register augment data for an item. Returns True on success."""
    item_id = name_to_id.get(item_name)
    if not item_id:
        errors.append(f"  Item not found in items.json: {item_name}")
        print(errors[-1])
        return False
    augments[str(item_id)] = {"paths": paths}
    total_ranks = sum(len(p["ranks"]) for p in paths)
    print(f"  {item_name} (ID {item_id}): "
          f"{len(paths)} paths, {total_ranks} rank entries")
    return True


def main():
    name_to_id = load_items_json()
    augments = {}
    errors = []

    # --- Set pages (Odyssey AT3/AT4) ---
    for url, item_names in SET_PAGES:
        set_label = item_names[0][:3]
        print(f"Fetching set: {set_label}... ({url})")
        try:
            html = fetch_page(url)
            slot_data = parse_set_page(html)
            if not slot_data:
                errors.append(f"  No augment table: {set_label}")
                print(errors[-1])
                continue
            for slot_idx, paths in slot_data.items():
                if slot_idx < len(item_names):
                    register_item(augments, name_to_id,
                                  item_names[slot_idx], paths, errors)
        except Exception as e:
            errors.append(f"  Error: {set_label}: {e}")
            print(errors[-1])
        time.sleep(1)

    # --- Limbus set pages (Rank 30 only) ---
    for url, name_groups in LIMBUS_SET_PAGES:
        set_label = name_groups[0][0][:3]
        print(f"Fetching Limbus set: {set_label}... ({url})")
        try:
            html = fetch_page(url)
            slot_data = parse_limbus_set_page(html)
            if not slot_data:
                errors.append(f"  No Rank 30 data: {set_label}")
                print(errors[-1])
                continue
            for name_group in name_groups:
                for slot_idx, paths in slot_data.items():
                    if slot_idx < len(name_group):
                        register_item(augments, name_to_id,
                                      name_group[slot_idx], paths, errors)
        except Exception as e:
            errors.append(f"  Error: {set_label}: {e}")
            print(errors[-1])
        time.sleep(1)

    # --- Individual item pages (AT1/AT2, RMEA) ---
    for url, item_name in INDIVIDUAL_PAGES:
        print(f"Fetching: {item_name} ({url})")
        try:
            html = fetch_page(url)
            paths = parse_individual_page(html)
            if not paths:
                errors.append(f"  No augment table: {item_name}")
                print(errors[-1])
                continue
            register_item(augments, name_to_id, item_name, paths, errors)
        except Exception as e:
            errors.append(f"  Error: {item_name}: {e}")
            print(errors[-1])
        time.sleep(1)

    # --- Write output ---
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
