#!/usr/bin/env bash
#
# 生成された items.json の健全性を検査する。
#
# 使い方:
#   scripts/validate_items.sh [items.json のパス]
#   MIN_ITEMS=14000 scripts/validate_items.sh web/data/items.json
#
# なぜ変換スクリプトと分けるか:
#   parse_lua_to_json.py の責務は Lua → JSON の変換であり、
#   どの件数を許容するかは上流データの性質ではなく運用ポリシーだから (docs/adr/0003)。
#
# なぜ必要か:
#   items.json は git 管理外 (docs/adr/0003) なので、壊れた生成物を差分レビューで
#   検出できない。加えて schedule 実行は無人である。上流の構造変更でパースが
#   空振りしても変換スクリプトは例外を投げずに「0 件生成して正常終了」しうるため、
#   ここで下限を検査して CI を止める。
#
set -euo pipefail

ITEMS_JSON="${1:-web/data/items.json}"
MIN_ITEMS="${MIN_ITEMS:-14000}"

if [ ! -f "$ITEMS_JSON" ]; then
  echo "ERROR: $ITEMS_JSON が存在しない。" >&2
  exit 1
fi

count=$(jq -r '.item_count' "$ITEMS_JSON")
echo "generated $count items (minimum: $MIN_ITEMS)"

if [ "$count" -lt "$MIN_ITEMS" ]; then
  echo "ERROR: expected at least $MIN_ITEMS items." >&2
  echo "Upstream data may be broken or its format may have changed." >&2
  exit 1
fi
