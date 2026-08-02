---
status: accepted
date: 2026-08-02
decision-makers: Akira Maruoka
---

# 0003. 装備データ `items.json` は git 管理せず、CI で上流 Lua から生成する

## Context and Problem Statement

装備検索と装備セット機能には全アイテムのデータが要る。出所は Windower/Resources が
公開している `items.lua` / `item_descriptions.lua` で、変換後の `items.json` は
14,921 件・約 8.5MB になる。これはゲームアップデートのたびに更新される巨大な生成物であり、
リポジトリに置くか、置かずに都度生成するかを決める必要がある。

## Decision Drivers

* リポジトリのサイズと履歴を汚さないこと（8.5MB のバイナリ的差分が積み上がるのを避ける）
* 配信されるデータが上流に追随すること
* サーバーを持たない制約（[ADR 0001](0001-rust-wasm-static-site.md)）の下で配れること
* ローカル開発の手間が過大にならないこと

## Considered Options

1. `items.json` をリポジトリにコミットする
2. CI のビルド時に上流 Lua から生成し、配信成果物にのみ含める（git 管理外）
3. Supabase の `items` テーブルに格納し、クライアントが実行時に取得する

## Decision Outcome

選択: **2. CI のビルド時に生成し、git 管理外とする**（採用）。

`items.json` は加工可能な派生物であって、人間が編集する一次データではない。
一次データは上流の Lua であり、変換スクリプトはリポジトリにある。
したがってリポジトリに保持すべきは「生成方法」であって「生成物」ではない。

- 変換は `scripts/parse_lua_to_json.py` が担う。ジョブ・スロット・種族のビットマスクを
  展開し、説明文を結合して JSON 化する。
- `.github/workflows/deploy.yml` が `curl` で上流 Lua を取得し、スクリプトを実行して
  `web/data/items.json` を生成した上で、`web/` 全体を Pages にアップロードする。
- `web/data/items.json` は `.gitignore` に登録する。
- ローカルで装備検索を動かす場合は、同じスクリプトを手動で実行して生成する。

`items` テーブルは `supabase/schema.sql` に「CI でのインポート用」として定義されているが、
現在 CI はここへインポートしていない。選択肢 3 の検討痕跡であり、実際には使われていない。

### Consequences

* Good: リポジトリが軽く保たれ、8.5MB の生成物が履歴に蓄積しない。
* Good: デプロイのたびに上流の最新データが反映される。手動の追随作業が要らない。
* Bad: 上流 (Windower/Resources) の URL 変更やフォーマット変更でビルドが壊れる。
  外部リポジトリがビルドの必須依存になっている。
* Bad: ローカル開発では手動生成が必要で、生成しないと装備検索・装備セットが動かない。
* Bad: `web/test/*.test.js` は実際の `items.json` を読んで抽出結果を検証する作りのため、
  データが git にない以上そのままでは CI で走らせられない
  （現に CI に含まれていない — [ADR 0001](0001-rust-wasm-static-site.md) の Confirmation 参照）。
* Neutral: 配信された `items.json` の内容はデプロイ時点のスナップショットであり、
  次のデプロイまで更新されない。
* Neutral: `supabase/schema.sql` の `items` テーブルは未使用のまま残っている。

### Confirmation

* `.gitignore` に `web/data/items.json` が登録されており、生成物がコミットされない。
* `.github/workflows/deploy.yml` の "Download Windower Resources" と
  "Generate items JSON" ステップが、毎回のデプロイで上流取得と変換を実行する。
  ここで失敗すれば deploy ジョブに進まない。

検証されていないもの:

* 生成された `items.json` の件数やスキーマを検査するステップはない。
  `parse_lua_to_json.py` は件数を標準出力に print するだけで、閾値チェックや
  assert を行わないため、上流が壊れて 0 件になっても CI は成功しうる。
* 上流フォーマット変更を検知する仕組みはない（変換スクリプトが例外を投げた場合のみ止まる）。

## More Information

* 全体構成: [ADR 0001](0001-rust-wasm-static-site.md)
* リポジトリ内で管理するテーブルデータとの線引き: [ADR 0002](0002-shared-table-data-json.md)
* 上流に存在しないオーグメントデータの扱い: [ADR 0004](0004-augment-data-managed-separately.md)
* 上流: <https://github.com/Windower/Resources>
