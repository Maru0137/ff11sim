---
status: accepted
date: 2026-08-02
decision-makers: Akira Maruoka
---

# 0004. オーグメントデータは独立の `augments.json` として管理し、手動スクレイピングで更新する

## Context and Problem Statement

オデシー装束やマスター武器などは、同じアイテム ID のままオーグメント（強化）の
ランクによって性能が変わる。しかし [ADR 0003](0003-items-json-generated-in-ci.md) で
一次データとしている Windower/Resources にはオーグメント情報が含まれていない。
装備セットの計算をオーグメント込みで正しく行うには、このデータをどこから調達し、
どう管理するかを別途決める必要がある。

## Decision Drivers

* オーグメント込みの計算ができること（未対応だと主要装備の数値が実態と合わない）
* [ADR 0003](0003-items-json-generated-in-ci.md) のビルドを外部サイトへ増やさないこと
* データの誤りを差分レビューで見つけられること
* 網羅できない装備があっても機能が破綻しないこと

## Considered Options

1. データを持たず、ユーザーがカスタム欄に手入力する
2. `augments.json` を用意し、装備ごとにランクをドロップダウンで選ばせる
    - 2.1. CI のビルド時に毎回スクレイピングして生成する
    - 2.2. 手元でスクレイピングし、生成結果をコミットする
3. `items.json` 生成時にマージし、単一の装備データに統合する

## Decision Outcome

選択: **2 + 2.2. `augments.json` をコミットして持ち、ランク選択式にする**（採用）。
併せて **1. カスタム入力欄** も残し、データ未整備の装備を補えるようにする。

CI での毎回スクレイピング（2.1）は、外部サイトへの依存と負荷をデプロイのたびに発生させ、
サイト構造の変更でビルドが壊れる。オーグメントデータは `items.json` と違って
更新頻度が低く、手元生成＋コミットで十分追随できる。

- `scripts/scrape_augments.py` で生成し、結果を `web/data/augments.json` にコミットする。
  このスクリプトは CI では実行しない。
- データ形式は `{"version": 1, "augments": {"<item_id>": {"paths": [{"type", "ranks": [{"rank", "text"}]}]}}}`。
  効果は日本語のテキストとして保持し、JA→EN 変換を経て装備説明文と同じパーサに通す
  (現在は `rust/src/equip_stats.rs`。移植の経緯は [ADR 0010](0010-equipment-interpretation-in-rust.md))。
- `augments.json` は `web/data/` 直下の実ファイルとして置く
  （[ADR 0002](0002-shared-table-data-json.md) の symlink 共有の対象外）。
  当初は Web 専用データで Rust 側は参照しない想定だったが、
  [ADR 0009](0009-embed-item-data-in-binary.md) により Rust が `include_str!` で
  埋め込むようになった。`items.json` と違いリポジトリ管理下なのでビルド順の制約は受けない。
- オーグメント定義がない装備は、ドロップダウンを disabled にして選べないようにする。
  ユーザーはカスタム欄に自分で記述できる。

### Consequences

* Good: デプロイが外部サイトのスクレイピングに依存しない。ビルドが安定する。
* Good: データ変更が差分レビューに載る。誤りを追跡・修正しやすい。
* Good: 未整備の装備でもカスタム欄で計算でき、機能が完全に止まらない。
* Bad: データの鮮度が手動更新に依存する。新装備が追加されても誰かがスクリプトを
  実行してコミットするまで反映されない。
* Bad: `items.json` は CI で毎回最新化されるのに `augments.json` は固定であるため、
  両者の対象装備がずれうる（新しい装備 ID にオーグメント定義がない状態）。
* Neutral: 効果を構造化データではなく日本語テキストで持つため、抽出精度は
  `equip-stats.js` のパーサの精度に完全に依存する。

### Confirmation

[ADR 0009](0009-embed-item-data-in-binary.md) で `augments.json` を Rust に
埋め込んだ際に、いくつか自動チェックが入った。

* `items::tests::augments_are_loaded` — 読み込めること。
* `items::tests::augments_by_item_id_resolves_known_item` — 経路とランクを引けること。
  ランクが昇順で `text` が空でないことも見る。
* `items::tests::augment_target_items_exist_in_items` — **`augments.json` が参照する
  アイテム ID がすべて `items.json` に存在すること。** 「`augments.json` が古くなる」
  という本 ADR の弱点を検出する。現状は全件一致。
* `rust/tests/augment_ja_to_en.rs` — 日本語表記が英語表記へ正しく変換されること。
  移植時に全 1,646 件で JS 実装と一致を確認した（現在は期待値 JSON を用意しないと走らない）。

検証されていないもの:

* `augments.json` が最新の `items.json` に対して十分な網羅率を持つか
  （参照先が存在することは見ているが、逆に「あるべき装備の定義が無い」ことは見ていない）。
* `augments.json` の `text` が抽出パーサで解釈可能な表記か
  （解釈できない表記は黙って 0 として無視される）。
* `scripts/scrape_augments.py` が現在のスクレイピング対象サイトに対して動作するか。

フォローアップ候補: `augments.json` の全 `text` を JA→EN 変換と抽出に通し、
1 つも stat を取り出せないエントリを検出するテストを追加する。
`rust/src/equip_stats.rs` の関数を直接呼べるので、移植後は書きやすくなっている。

## Pros and Cons of the Options

### 1. データを持たず、ユーザーがカスタム欄に手入力する

* Good: メンテナンスコストがゼロ。データの陳腐化が起きない。
* Good: 未収録の装備や、まだ情報が出回っていないオーグメントにも対応できる。
* Bad: 主要装備のたびにユーザーが自分で調べて入力することになり、実用に耐えない。
* Bad: 入力ミスが計算結果に直結し、ユーザーからはシミュレータ側の誤りに見える。

### 2. `augments.json` を用意し、ランクをドロップダウンで選ばせる（採用）

* Good: ユーザーはランクを選ぶだけでよく、入力ミスが起きない。
* Good: データが 1 箇所にあるため、誤りを見つけたら全ユーザーぶんまとめて直せる。
* Bad: 上流にないデータを自前で維持する責任を負う。
* Bad: 対象装備の網羅は原理的に終わらない。

#### 2.1. CI のビルド時に毎回スクレイピングして生成する

* Good: データが常に最新に保たれ、手動更新の手間がない。
* Bad: デプロイのたびに外部サイトへアクセスし、負荷をかける。
* Bad: サイト構造の変更でデプロイが壊れる。
  [ADR 0003](0003-items-json-generated-in-ci.md) の上流 Lua に続いて 2 つ目の外部依存になる。
* Bad: スクレイピング結果が差分レビューに載らず、誤ったデータが黙って配信されうる。

#### 2.2. 手元でスクレイピングし、生成結果をコミットする（採用）

* Good: デプロイが外部サイトに依存せず、ビルドが安定する。
* Good: データ変更が差分レビューに載り、誤りを検出・追跡できる。
* Good: オーグメントは `items.json` ほど更新頻度が高くないため、手動でも追随できる。
* Bad: 鮮度が手動更新に依存する。更新を忘れると新装備が反映されない。
* Bad: CI で毎回最新化される `items.json` との間にずれが生じる。

### 3. `items.json` 生成時にマージし、単一の装備データに統合する

* Good: Web 側が読むデータが 1 つになり、参照が単純になる。
* Bad: [ADR 0003](0003-items-json-generated-in-ci.md) により `items.json` は git 管理外のため、
  手で整備したオーグメントデータまで git 管理外になってしまい、レビューできなくなる。
* Bad: マージ処理を `parse_lua_to_json.py` に足すことになり、
  上流由来のデータと自前データの境界が曖昧になる。
* Bad: 自前データを 1 件直すためだけに `items.json` 全体の再生成が要る。

## More Information

* 装備データ本体の供給: [ADR 0003](0003-items-json-generated-in-ci.md)
* テーブルデータの共有方針（本 ADR は対象外とする）: [ADR 0002](0002-shared-table-data-json.md)
