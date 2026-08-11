#!/usr/bin/env bash
#
# 上流由来のデータを一括で作る。CI とローカルで同じ結果になる。
#
# 主生成物の items.json は Rust が include_str! で埋め込むため、Web 専用ではなく
# cargo build / cargo test より前に実行する必要がある。
#
# 使い方:
#   scripts/build_data.sh
#
# 生成物:
#   build/items.json               装備データ本体 (約 8.5MB, .gitignore 済み)
#   web/data/_build_metadata.json  ビルドのメタ情報 (.gitignore 済み)
#   temp_resources/*.lua           上流からのダウンロードキャッシュ (.gitignore 済み)
#
# items.json を web/ の外に置くのは、ブラウザがこれを読まなくなったため
# (docs/adr/0009 / docs/adr/0010)。Rust が include_str! で埋め込むので、
# web/ に置くと同じデータを二重に配ることになる。
# _build_metadata.json は次回の変化判定で配信サイト経由で読むので web/ に残す。
#
# 環境変数:
#   UPSTREAM_BLOBS  detect_resource_changes.sh が出力した blobs (JSON)。
#                   未指定ならこのスクリプトが GitHub API から取得する。
#                   CI では detect ジョブの結果を渡し、API の重複呼び出しを避ける。
#   COMMIT          _build_metadata.json に記録する ff11sim 自身の commit。
#                   既定は git rev-parse HEAD。
#   MIN_ITEMS       items.json の件数下限 (validate_items.sh へ渡す)。
#
# メタデータをこのスクリプトで書く理由:
#   _build_metadata.json は「この生成物が何から作られたか」の記録である。
#   データを作る処理と同じ場所で書けば、生成物とメタデータがずれない (docs/adr/0003)。
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/upstream_common.sh"

REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CACHE_DIR="${CACHE_DIR:-$REPO_ROOT/temp_resources}"
BUILD_DIR="$REPO_ROOT/build"
WEB_DATA_DIR="$REPO_ROOT/web/data"
ITEMS_JSON="$BUILD_DIR/items.json"
# items.json の生成元を記録する指紋。上流の blob SHA と変換スクリプトのハッシュから作る。
# 一致していれば再生成を省ける。変換スクリプトを入れたのは、上流が同じでも
# パース結果が変わりうるため。
ITEMS_SIGNATURE="$BUILD_DIR/.items-signature"
METADATA_JSON="$WEB_DATA_DIR/_build_metadata.json"

mkdir -p "$CACHE_DIR" "$BUILD_DIR" "$WEB_DATA_DIR"

# --- 1. 上流の blob SHA を確定 -------------------------------------------------
if [ -n "${UPSTREAM_BLOBS:-}" ]; then
  echo "==> 上流 blob SHA: 呼び出し元から受け取った値を使う"
  blobs="$UPSTREAM_BLOBS"
else
  echo "==> 上流 blob SHA を取得"
  blobs=$(resolve_upstream_blobs)
fi
jq . <<<"$blobs"

# --- 2. items.json の再生成が必要か判定 ----------------------------------------
# 上流が変わらず変換スクリプトも同じなら、生成物は必ず同じになるので作り直さない。
# FORCE_REBUILD=1 で常に再生成する (CI の手動実行で使う)。
signature="$(printf '%s\n%s\n' "$blobs" "$(git hash-object "$SCRIPT_DIR/parse_lua_to_json.py")")"
if [ "${FORCE_REBUILD:-}" != "1" ] \
  && [ -f "$ITEMS_JSON" ] \
  && [ -f "$ITEMS_SIGNATURE" ] \
  && [ "$(cat "$ITEMS_SIGNATURE")" = "$signature" ]; then
  echo "==> items.json は最新のため再生成をスキップ"
  echo "    ($(jq -r '.item_count' "$ITEMS_JSON") 件)"
  skip_generate=1
else
  skip_generate=0
fi

# --- 3. 上流ファイルを取得 (キャッシュが最新ならスキップ) -----------------------
# GitHub の blob SHA は git のオブジェクト ID そのものなので、
# git hash-object でローカルファイルのハッシュを計算すれば一致判定ができる。
if [ "$skip_generate" = "0" ]; then
echo "==> 上流ファイルを取得"
for name in "${TRACKED_FILES[@]}"; do
  dest="$CACHE_DIR/$name"
  want=$(jq -r --arg k "$UPSTREAM_DIR/$name" '.[$k]' <<<"$blobs")
  if [ -f "$dest" ] && [ "$(git hash-object "$dest")" = "$want" ]; then
    echo "  $name: キャッシュが最新のためスキップ"
    continue
  fi
  echo "  $name: ダウンロード"
  curl -fsSL --max-time 300 -o "$dest" "$(upstream_raw_url "$name")"
done

# --- 4. items.json を生成 ------------------------------------------------------
echo "==> items.json を生成"
python3 "$SCRIPT_DIR/parse_lua_to_json.py" \
  --items "$CACHE_DIR/items.lua" \
  --descriptions "$CACHE_DIR/item_descriptions.lua" \
  --output "$ITEMS_JSON"

# --- 5. 検証 -------------------------------------------------------------------
echo "==> items.json を検証"
"$SCRIPT_DIR/validate_items.sh" "$ITEMS_JSON"

# 検証を通ったものだけ指紋を残す。落ちた生成物を「最新」と誤認しないため。
printf '%s' "$signature" > "$ITEMS_SIGNATURE"
fi

# --- 6. ビルドメタデータを書き出す (再生成の有無に関わらず毎回作る) -------------
echo "==> _build_metadata.json を生成"
commit="${COMMIT:-$(git -C "$REPO_ROOT" rev-parse HEAD)}"
jq -n \
  --arg built_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg commit "$commit" \
  --arg repo "$UPSTREAM_REPO" \
  --arg ref "$UPSTREAM_REF" \
  --argjson blobs "$blobs" \
  '{
    version: 1,
    built_at: $built_at,
    commit: $commit,
    upstream_sources: {
      windower_resources: {
        repo: $repo,
        ref: $ref,
        blobs: $blobs
      }
    }
  }' > "$METADATA_JSON"
cat "$METADATA_JSON"
