---
status: accepted
date: 2026-08-10
decision-makers: Akira Maruoka
---

# 0010. 装備の解釈と検索を Rust に移し、段階的に移行する

## Context and Problem Statement

装備の説明文 (`description_en`) から数値を取り出しているのは JavaScript の
`web/js/equip-stats.js` であり、Rust には合算済みの `BonusStats`（HP/STR/DEF … の
数値だけ）が渡る。装備の検索も `web/js/item-search.js` が `items.json` を読んで行っている。

[ADR 0009](0009-embed-item-data-in-binary.md) で装備データを WASM バイナリに埋め込み、
JS 側の `items.json` 読み込みを廃止すると決めた。JS はもとデータを持たなくなるため、
現在の JS 実装はそのままでは成立しない。何をどこまで Rust に移し、検索をどう扱い、
移行をどう進めるかを決める必要がある。

## Decision Drivers

* 装備の解釈が 1 箇所にあること。現在は JS の実装と `rust/src/wasm.rs` の手書き転記が
  並存し、後者が黙って腐る
* 装備検索 UI の応答性を損なわないこと
* 移植量が個人開発として現実的であること
* 移行の途中で壊れたまま放置されないこと。既存の検証資産を使えること

## Considered Options

1. 解釈と検索の両方を Rust に移し、すべて WASM 経由にする
2. 解釈だけ Rust に移し、検索用の軽量インデックスを別途配信して JS が検索を続ける
3. ビルド時に全装備の stats を抽出しておき、実行時の解釈を減らす

1 と 2 は「検索をどちらに置くか」で分かれる。3 は 1 / 2 と排他ではない補助案。

## Decision Outcome

選択: **1. 解釈と検索の両方を Rust に移す**（採用）。移行は段階的に行う。

データが WASM 側にある以上、検索も同じ場所に置くのが素直である。選択肢 2 は
検索用インデックスという第 2 のデータソースを生み、埋め込みデータとの同期ずれという
新しい問題を作る。[ADR 0009](0009-embed-item-data-in-binary.md) で「配布物から
データファイルを消す」と決めた直後に別のデータファイルを足すのは一貫性を欠く。

検索の応答性については、WASM 境界を越えるのはキーストロークごとに 1 回・
クエリ文字列を渡して結果集合を受け取るだけで、アイテムごとの往復は発生しない。
現時点で問題になると判断する材料はないため、実測して問題が出たら選択肢 2 に退避する。

**移植の範囲**

- 装備解釈: `web/js/equip-stats.js`（455 行、正規表現 59 個、抽出項目 26 種）
- JA→EN 変換: `web/js/constants.js` の `AUGMENT_JA_TO_EN`（88 行）と
  `web/js/utils.js` の `convertAugmentJaToEn`
- 装備検索: `web/js/item-search.js`（418 行）。カタカナ→ひらがな・全角→半角の
  正規化を含む
- ユーザーのカスタム入力（`custom_description`）は実行時にしか解釈できないため、
  JS は文字列をそのまま WASM に渡す

**移行の順序**

一度にすべてを移すと動作確認の粒度が粗くなるため、次の順で進める。

1. `items.json` / `augments.json` を Rust に埋め込み、参照 API を用意する（JS は現状のまま）
2. 装備解釈を移植し、`web/test/equip-stats-extraction.test.js` の 76 アサーションを
   適合性テストとして Rust 側へ持ち込む
3. JS の `equip-stats.js` を WASM 呼び出しに置き換える
4. 装備検索を移植し、`item-search.js` を WASM 呼び出しに置き換える
5. JS 側の `items.json` fetch を削除し、配信物からも除外する。CI のステップ順を入れ替える

手順 2 から 4 の間は解釈が JS と Rust の 2 実装で並存する。ここは移行期間として許容し、
`web/test/*.test.js` を突き合わせに使う。手順 5 が完了するまで
[ADR 0009](0009-embed-item-data-in-binary.md) の「二重に配らない」前提は満たされない。

**2026-08-10 時点で手順 1〜5 はすべて完了した。** 実績は Confirmation を参照。

### この決定を見直す条件

- 検索の応答性が実測で問題になったとき。選択肢 2（検索用インデックスを別途配信）へ移る
- 移植の途中で 2 実装の食い違いが収束しないとき

### Consequences

* Good: 装備解釈が Rust の型で表現され、[ADR 0001](0001-rust-wasm-static-site.md) の
  1 番目のドライバ（整数と実数を型で区別）の恩恵が解釈部分にも及ぶ。
* Good: 実装が 1 箇所になる。CLI・WASM・テストが同じ解釈を共有する。
* Good: `rust/src/wasm.rs` の手書き転記を実データ参照に置き換えられる。
* Good: 既存の `web/test/equip-stats-extraction.test.js`（76 アサーション）を
  移植の適合性テストとして使える。移行が「動くかどうか分からない」状態にならない。
* Bad: 移植量が大きい。合計 960 行超に加え、Rust へ `fancy-regex` の依存が増える。
  JS 側が後読み/先読みを 91 箇所で使っており `regex` crate では移植できないため。
  unicode 機能は切っている (日本語リテラルと文字クラス範囲は扱えることを実測で確認)。
  この依存で WASM が gzip 約 +339KB 増えた ([ADR 0009](0009-embed-item-data-in-binary.md)
  の Confirmation 参照)。
* Bad: 移行期間中は 2 実装が並存し、食い違いが起こりうる。
* Bad: 検索がキーストロークごとに WASM 境界を越えるようになる。
  現時点で問題になる材料はないが、未計測である。
* Bad: `web/test/*.test.js` は `items.json` を直接読む作りのため書き換えが要った。
  → 3 本とも Rust へ移植 (2 本) または削除 (1 本、全件突き合わせが上位互換) して解消。
  結果として JS 時代 CI に入っていなかったこれらが `cargo test` で走るようになった。
* Neutral: ブラウザでの解釈は WASM の初期化完了を待つことになる。
  ステータス計算自体が既に WASM に依存しているため、新しい制約ではない。

### Confirmation

手順 1〜5 は完了済み。実施した検証は次のとおり。

**JS 実装との全件突き合わせ** (移植の完了判定に使った)

| 対象 | 件数 | 結果 |
|---|---|---|
| `extract_all_stats` | 15,504 | 不一致 0 |
| `extract_skill_bonuses` | 15,504（うち非空 2,576 / 4,056 エントリ） | 不一致 0 |
| 検索（14 ケース） | クエリ / 絞り込み / ソート / フィルタ / ページング | 件数・並び順とも一致 |
| JA→EN 変換 | 1,646 | 不一致 0 |

`extract_all_stats` / `extract_skill_bonuses` は WASM 境界を JS から呼ぶ形でも
全件一致を確認しており、`wasm-bindgen` を跨いだ戻り値の形まで含めて等価である。

**移植後に残した検証**

* `equip_stats::tests::conformance_with_js_over_all_items` および
  `augment_ja_to_en` の全件比較 — 移植の判定に使ったが、**移植元の JS 実装と
  期待値生成ハーネスを削除したため現在は実行できない**。期待値 JSON を用意すれば
  動く形で残してあり、抽出ロジックを大きく変えるときに変更前後を突き合わせる用途で使える。
  環境変数 (`JS_STATS` / `AUG_CONVERTED`) 未設定ならスキップする。
* `add_covers_every_field_without_omission` / `set_from_map_roundtrips_every_field` —
  78 項目の加算・キー対応の漏れを検出する。実際に 1 項目外して落ちることを確認済み。
* `rust/tests/` の 3 本（連携ボーナス / WS ダメージ / JA→EN 変換）。
  JS 時代は CI に入っていなかったが、`cargo test` で走るようになった。

**手順の完了確認**

* `items.json` への fetch はコード上に残っていない。`web/js/equip-stats.js` と
  `web/js/item-search.js` は削除済み。
* `items.json` の出力先を `build/` に移し、Pages の配信対象から外した。
  これで [ADR 0009](0009-embed-item-data-in-binary.md) の「二重に配らない」前提を満たす。

**未検証のもの**

* 検索の応答性は実測していない。キーストロークごとに WASM 境界を越えて最大 50 件を
  返す設計のままで、体感で問題が出たら選択肢 2 に退避する。
* 名前ソートは JS の `localeCompare` に対しコードポイント順としたため一致しない。
  意図的な非互換として `docs/tech-debt/equip-stats-js-quirks.md` に記録している。

## Pros and Cons of the Options

### 1. 解釈と検索の両方を Rust に移す（採用）

* Good: データと、それを読む処理が同じ場所に揃う。
  [ADR 0009](0009-embed-item-data-in-binary.md) の「配布物からデータファイルを消す」と
  一貫する。
* Good: 解釈の実装が 1 つになる。
* Bad: 移植量が最大になる（解釈 455 行 + 検索 418 行 + 対訳表 88 行）。
* Bad: 検索がキーストロークごとに WASM 境界を越える。

### 2. 解釈だけ Rust に移し、検索用の軽量インデックスを別途配信する

アイテム ID と名前だけの小さな JSON を配信し、検索は JS が従来どおり行う。
選択した装備の stats 取得だけを WASM に問い合わせる。

* Good: 移植量が 418 行分減る。日本語の正規化処理を書き直さずに済む。
* Good: 検索が WASM 境界を越えない。応答性の不確実性がない。
* Bad: **第 2 のデータソースが生まれる。** 埋め込んだデータとインデックスの同期ずれが
  新しい問題になる。生成フローにも段が増える。
* Bad: [ADR 0009](0009-embed-item-data-in-binary.md) で配布物からデータファイルを
  消すと決めた直後に、別のデータファイルを足すことになる。
* Bad: 検索結果の表示に必要な情報（ジョブ・レベル・スロット等）をインデックスに
  どこまで含めるかで、結局サイズが膨らみうる。

### 3. ビルド時に全装備の stats を抽出しておく

* Good: 実行時の解釈コストが消える。
* Bad: **部分解にしかならない。** ユーザーのカスタム入力は実行時にしか解釈できないため、
  実行時パーサは残る。「解釈をどこに置くか」という問いは解決しない。
* Bad: 生成物が 1 つ増え、[ADR 0003](0003-items-json-generated-in-ci.md) の
  ビルドパイプラインに段が増える。

## More Information

* この決定の前提となるデータ配置: [ADR 0009](0009-embed-item-data-in-binary.md)
* 全体構成と型による厳密さのドライバ: [ADR 0001](0001-rust-wasm-static-site.md)
* 移植対象だったもの (いずれも削除済み): `web/js/equip-stats.js`、`web/js/item-search.js`、
  `web/js/constants.js` の `AUGMENT_JA_TO_EN`、`web/js/utils.js` の `convertAugmentJaToEn`、
  `web/test/*.test.js`
* 移植先: `rust/src/equip_stats.rs`、`rust/src/item_search.rs`、`rust/tests/`
* 移植中に見つかった JS 実装の癖と、意図的に受け入れた非互換:
  `docs/tech-debt/equip-stats-js-quirks.md`
* テストが実装の定数をミラーしていた問題: `docs/tech-debt/mirrored-constants-in-tests.md`
