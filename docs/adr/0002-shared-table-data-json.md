---
status: accepted
date: 2026-08-02
decision-makers: Akira Maruoka
---

# 0002. テーブルデータを `/data/*.json` に単一ソース化し、Rust と Web の両方から読む

## Context and Problem Statement

ジョブ名・種族名・ステータスグレード表・ジョブ×スキルのランク行列といったテーブルデータは、
Rust の計算側と Web の UI 側の両方が必要とする。両者に別々の定義を持つと必ずずれが生じ、
しかも「UI の表示は正しいが計算が古い表を使っている」という発見しにくい不整合になる。
どこを単一の真実の源とし、両言語からどう参照するかを決める必要がある。

## Decision Drivers

* Rust と JavaScript が同一の値を参照すること（定義の二重化を避ける）
* データの欠損・追加漏れを可能な限り早い段階で検出すること
* ビルドステップを増やさないこと（[ADR 0001](0001-rust-wasm-static-site.md) の複雑度制約）
* 差分レビューでデータ変更が読めること

## Considered Options

1. Rust のソース内に定数として持ち、Web へは WASM API 経由で公開する
2. JavaScript 側に持ち、Rust へはビルド時にコード生成する
3. `/data/*.json` に単一ソースを置き、両方から読む
    - 3.1. Rust も実行時にファイルを読み込む
    - 3.2. Rust は `include_str!` でバイナリに埋め込む

1 と 2 は「どちらかの言語を正とする」案、3 は「言語外を正とする」案。
3.1 と 3.2 は Rust 側の読み込みタイミングで分かれる。

## Decision Outcome

選択: **3 + 3.2. `/data/*.json` を単一ソースとし、Rust は `include_str!` で埋め込む**（採用）。

WASM 経由でのデータ公開（選択肢 1）は、UI が起動時に WASM の初期化完了を待つ必要が生じ、
`search.html` のような計算を伴わないページまで WASM に依存させてしまう。
JSON を言語外の共通ソースとすれば、どちらの側も相手の都合を知らずに読める。

- 全 JSON は `{"version": u32, "data": ...}` のラッパ形式に統一する。
- Rust 側は `rust/src/data_loader.rs` に集約し、`include_str!` + `LazyLock` で読む。
  パース失敗は `expect` で panic させる（データ不正のまま動かさない）。
- Web 側は `web/js/constants.js` が top-level await で `./data/<name>.json` を fetch する。
  `web/data/` 配下にリポジトリルートの `data/` への symlink を張って参照する。
- ジョブ × スキル、種族 × ステータスといった行列は `EnumMap<K, V>` で受ける。
  `EnumMap` の deserialize は全バリアントの存在を要求するため、
  **ジョブやスキルを enum に追加したのに JSON を更新し忘れると parse 時に落ちる**。
  これをデータ完全性の静的検証として意図的に利用する。
- 対象範囲はメタデータと係数・グレード表まで。ジョブ特性とギフトの効果値テーブルは
  この決定の対象外であり、Rust ソース内に置いたままとする（別 ADR で扱う余地がある）。

### Consequences

* Good: 定義が 1 箇所になり、Rust と Web のずれが原理的に起きない。
* Good: enum 追加時のデータ更新漏れが `EnumMap` の deserialize で検出される。
  実際 `data/job_status_grades.json` は MP を持たないジョブを `null` として
  全 22 ジョブ × 全ステータスを明示している。
* Good: JSON なので差分レビューでデータ変更が読める。
* Bad: Rust 側はビルド時埋め込みのため、JSON を編集しただけでは反映されず再ビルドが要る。
* Bad: `web/data/` の symlink に依存する。チェックアウト環境やホスティングが
  symlink を実体化しない場合に壊れるが、これを検出する仕組みはない。
* Bad: JSON が壊れた場合の失敗はコンパイル時ではなく実行時 panic になる。
* Neutral: ラッパの `version` フィールドは持っているが、Rust 側では現状未使用
  （`data_loader.rs` の `DataFile` で `#[allow(dead_code)]`）。将来のマイグレーション用の枠。

### Confirmation

`rust/src/data_loader.rs` の `#[cfg(test)] mod tests` が以下を検証する:

* `jobs_meta_covers_all_jobs` / `races_meta_covers_all_races` / `skills_meta_covers_all_skills`:
  `Vec` 形式のメタデータが `Job` / `Race` / `SkillKind` の全バリアントを過不足なく含むこと。
* `equipment_slots_count`: 装備スロットが 16 件であること。

`EnumMap` 形式の static（`RACE_STATUS_GRADES` / `JOB_STATUS_GRADES` / `JOB_SKILL_RANKS` /
`GRADE_COEFFICIENTS` / `SKILL_CAP_CONTROL_POINTS`）は専用テストを持たないが、
`rust/src/skills.rs` の `skill_cap` / `job_skill_rank` を経由して
`test_skill_cap_control_points` などのテストが参照するため、`LazyLock` が初期化され、
JSON に欠損があればテスト実行時に panic する。これらのテストは
[ADR 0001](0001-rust-wasm-static-site.md) の通り CI の `cargo test` で走る。

検証されていないもの:

* `web/data/` の symlink が有効かどうか。
* Rust 側と Web 側が同じ `version` の JSON を見ているかどうか（`version` は未使用）。
* Web 側 (`constants.js`) が読んだ値の妥当性。Web 側にはこの種のテストがない。

## More Information

* 全体構成: [ADR 0001](0001-rust-wasm-static-site.md)
* 装備データ（`items.json`）はサイズと出所が異なるため別扱い:
  [ADR 0003](0003-items-json-generated-in-ci.md)
* オーグメントデータも別扱い: [ADR 0004](0004-augment-data-managed-separately.md)
