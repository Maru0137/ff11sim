#!/usr/bin/env bash
#
# 上流 (Windower/Resources) に関する定数と共通処理。
# detect_resource_changes.sh と build_web_data.sh の両方から source する。
#
# 定数をここに集約するのは、追跡対象や ref が両スクリプトに分散して
# 片方だけ更新される事故を防ぐため。
#
UPSTREAM_REPO="Windower/Resources"
UPSTREAM_REF="master"
UPSTREAM_DIR="resources_data"
TRACKED_FILES=("items.lua" "item_descriptions.lua")

# 追跡対象ファイルの blob SHA (= ファイル内容のハッシュ) を
# {"<dir>/<name>": "<sha>", ...} 形式で stdout に出す。
#
# 代表値に commit SHA を使わない理由:
#   commit SHA は内容の代理でしかなく、
#     - 過去日付のコミットを含むブランチが後から master にマージされた場合
#     - 異なる 2 コミットの committer date が同秒だった場合
#   に変化を取りこぼす。blob SHA なら「内容が変わったときだけ変わる」ため、
#   取りこぼしも、内容不変な no-op コミットによる空振りビルドも起きない。
#
# ディレクトリ一覧エンドポイントを使うのでファイル内容は転送されない
# (メタデータのみ、約 34KB)。API 呼び出しも 1 回で済む。
resolve_upstream_blobs() {
  local tracked_json blobs count
  tracked_json=$(printf '%s\n' "${TRACKED_FILES[@]}" | jq -R . | jq -sc .)

  blobs=$(gh api "repos/${UPSTREAM_REPO}/contents/${UPSTREAM_DIR}?ref=${UPSTREAM_REF}" \
    | jq -c --argjson tracked "$tracked_json" --arg dir "$UPSTREAM_DIR" \
        "[.[]
          | select(.name as \$n | \$tracked | index(\$n))
          | {key: (\$dir + \"/\" + .name), value: .sha}]
         | sort_by(.key) | from_entries")

  count=$(jq -r 'length' <<<"$blobs")
  if [ "$count" -ne "${#TRACKED_FILES[@]}" ]; then
    echo "ERROR: 追跡対象 ${#TRACKED_FILES[@]} ファイルのうち ${count} 件しか見つからない。" >&2
    echo "上流でリネーム/削除された可能性がある。" >&2
    return 1
  fi

  echo "$blobs"
}

# 追跡対象ファイルの raw ダウンロード URL を返す。
upstream_raw_url() {
  echo "https://raw.githubusercontent.com/${UPSTREAM_REPO}/${UPSTREAM_REF}/${UPSTREAM_DIR}/$1"
}
