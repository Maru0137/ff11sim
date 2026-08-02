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
  効果は日本語のテキストとして保持し、抽出は装備説明文と同じパーサ (`web/js/equip-stats.js`) に通す。
- `augments.json` は Web 専用データであり、`web/data/` 直下の実ファイルとして置く
  （[ADR 0002](0002-shared-table-data-json.md) の symlink 共有の対象外）。Rust 側は参照しない。
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

自動チェックは存在しない。具体的には以下がいずれも検証されていない:

* `augments.json` が最新の `items.json` に対して十分な網羅率を持つか。
* `augments.json` の `text` フィールドが `equip-stats.js` のパーサで解釈可能な表記か
  （解釈できない表記は黙って 0 として無視される）。
* `scripts/scrape_augments.py` が現在のスクレイピング対象サイトに対して動作するか。

`web/test/equip-stats-extraction.test.js` は装備の `description_en` からの抽出を
検証しているが、`augments.json` の内容そのものは対象にしていない。

フォローアップ候補: `augments.json` の全 `text` を `equip-stats.js` に通し、
1 つも stat を抽出できないエントリを検出するテストを追加する。

## More Information

* 装備データ本体の供給: [ADR 0003](0003-items-json-generated-in-ci.md)
* テーブルデータの共有方針（本 ADR は対象外とする）: [ADR 0002](0002-shared-table-data-json.md)
