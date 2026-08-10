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

* データはSSOTで定義され、Rust と JavaScript がそれを参照すること
* データの欠損・追加漏れを可能な限り早い段階で検出すること
* サーバーを持たない制約（[ADR 0001](0001-rust-wasm-static-site.md)）の下で配れること。
  配信時にデータを加工する場所がないため、両者が読むのは静的な成果物でなければならない
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

なお「DB やサーバーを単一ソースとし、実行時に API で配る」案は、
[ADR 0001](0001-rust-wasm-static-site.md) のサーバーレス志向により
この ADR に至る前に検討対象から外れている。

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
* Good: 静的ホスティングだけで配れる。Rust 側はバイナリ埋め込み、Web 側は同一オリジンの
  静的ファイル fetch であり、どちらもサーバーを必要としない。
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

## Pros and Cons of the Options

### 1. Rust のソース内に定数として持ち、Web へは WASM API 経由で公開する

* Good: データが Rust の型そのものになり、コンパイル時に完全に検証される。
  JSON のパース失敗という実行時エラーが存在しない。
* Good: 共有の仕組みを別途作らなくてよい。
* Bad: Web 側がデータを読むために WASM の初期化完了を待つ必要がある。
  装備検索ページのように計算を伴わない画面まで WASM に依存させてしまう。
* Bad: データ変更のたびに Rust の再ビルドと WASM の再生成が要る。
* Bad: 表データの差分が Rust ソースの差分に埋もれ、レビューで読みにくい。

### 2. JavaScript 側に持ち、Rust へはビルド時にコード生成する

* Good: Web 側は追加の仕組みなしにそのまま読める。
* Good: 生成された Rust コードには型が付くため、コンパイル時検証が効く。
* Bad: コード生成ステップがビルドに増え、
  [ADR 0001](0001-rust-wasm-static-site.md) のビルド複雑度の制約に逆行する。
* Bad: 生成物をコミットするか否かという別の判断が発生し、
  生成し忘れによる不整合が起こりうる。

### 3. `/data/*.json` に単一ソースを置き、両方から読む（採用）

* Good: どちらの言語も相手の都合を知らずに読める。依存の向きが一方向にならない。
* Good: JSON なのでデータ変更が差分レビューに載る。
* Bad: JSON 自体はスキーマを持たないため、構造の正しさは各言語側の deserialize に委ねられる。
* Bad: Web 側は fetch が要るため、読み込みが非同期になる。

#### 3.1. Rust も実行時にファイルを読み込む

* Good: JSON を編集するだけで反映され、再ビルドが要らない。
* Bad: WASM には実行時のファイルシステムがない。fetch した内容を渡す口を `wasm.rs` に
  作ることになり、コアが実行形態に依存し始める
  （[ADR 0001](0001-rust-wasm-static-site.md) の「コアは wasm-bindgen に依存しない」に反する）。
  この制約はサーバーレス志向 → クライアント側で計算 → WASM という
  [ADR 0001](0001-rust-wasm-static-site.md) の帰結であり、本 ADR 単独の都合ではない。
* Bad: データファイルの配置が実行時の前提になり、テスト実行時のカレントディレクトリに左右される。

#### 3.2. Rust は `include_str!` でバイナリに埋め込む（採用）

* Good: 実行形態を問わず同じように動く。WASM でもネイティブでも CLI でも追加の配線が要らない。
* Good: データがバイナリに含まれるため、配布物が 1 つで完結する。
* Bad: JSON を編集しても再ビルドまで反映されない。
* Bad: パース失敗がコンパイル時ではなく、初回アクセス時の panic になる。

## More Information

* 全体構成: [ADR 0001](0001-rust-wasm-static-site.md)
* 装備データ（`items.json`）はサイズと出所が異なるため別扱い:
  [ADR 0003](0003-items-json-generated-in-ci.md)
* オーグメントデータも別扱い: [ADR 0004](0004-augment-data-managed-separately.md)
