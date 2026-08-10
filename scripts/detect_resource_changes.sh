#!/usr/bin/env bash
#
# 上流 (Windower/Resources) のリソースに変化があるかを判定する。
#
# 使い方:
#   METADATA_URL=<配信済み _build_metadata.json の URL> scripts/detect_resource_changes.sh
#
# 出力:
#   key=value 形式で stdout に出す (進捗ログは stderr)。
#   GitHub Actions からは `scripts/detect_resource_changes.sh >> "$GITHUB_OUTPUT"` で使う。
#     blobs=<{"<path>": "<blob sha>", ...}>
#     changed=<true|false>
#     digest=<blobs の sha256 先頭 16 桁。CI のキャッシュキーに使う>
#
# 判定方法:
#   上流の代表値には blob SHA を使う (理由は upstream_common.sh を参照)。
#   前回ビルド時の blob SHA は配信済みサイトの _build_metadata.json に記録してある
#   (docs/adr/0003)。取得に失敗した場合は changed=true に倒し、
#   「判定できないならビルドする」安全側の挙動にする。
#
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/upstream_common.sh"

: "${METADATA_URL:?METADATA_URL is required}"

log() { echo "$@" >&2; }

# 比較前に両辺をキー順で正規化し、記録時のキー順に依存しないようにする。
NORMALIZE='to_entries | sort_by(.key) | from_entries'

current=$(resolve_upstream_blobs)
log "現在の blob SHA:"
jq . <<<"$current" >&2

deployed='{}'
if raw=$(curl -fsSL --max-time 30 "$METADATA_URL" 2>/dev/null); then
  deployed=$(jq -c ".upstream_sources.windower_resources.blobs // {} | $NORMALIZE" <<<"$raw" 2>/dev/null || echo '{}')
fi
current=$(jq -c "$NORMALIZE" <<<"$current")

if [ "$deployed" = '{}' ]; then
  log "配信済みメタデータを取得できなかった (初回 or Pages 未到達) → ビルドする"
  changed=true
elif [ "$deployed" = "$current" ]; then
  log "上流の内容に変化なし → スケジュール実行ならスキップ"
  changed=false
else
  log "上流が更新された:"
  log "  before: $deployed"
  log "  after : $current"
  changed=true
fi

# キャッシュキー用の短い指紋。blobs は JSON なのでそのままキーに使えない。
digest=$(printf '%s' "$current" | shasum -a 256 2>/dev/null | cut -c1-16 \
  || printf '%s' "$current" | sha256sum | cut -c1-16)

echo "blobs=$current"
echo "changed=$changed"
echo "digest=$digest"
