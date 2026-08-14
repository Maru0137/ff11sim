---
status: accepted
date: 2026-08-14
decision-makers: Akira Maruoka
---

# 0018. 装備の解釈責務を equip モジュールに集約し、WASM 境界を装備セット単位にする

## Context and Problem Statement

[ADR 0010](0010-equipment-interpretation-in-rust.md) は「装備の解釈が 1 箇所にあること」を
決定要因に挙げ、解釈を `equip_stats.rs` に移した。しかし現在、装備の解釈は 5 箇所に散っている。

| 置き場所 | 持っている責務 |
| --- | --- |
| `rust/src/items.rs` | 装備データ・オグメントデータの参照 |
| `rust/src/equip_stats.rs` | 英語説明文 → 数値（固定 26 種・全一致合算） |
| `rust/src/item_search.rs` | 日本語テキスト → 任意名の値（最初の 1 件） |
| `web/src/equip/equip-bonuses.ts` | 3 ソース（説明文・オグメント・カスタム説明）の組み立てとスロット別集計 |
| `web/src/propsets/user-item-values.ts` | 同じ 3 ソースの組み立て（日本語のまま） |

分散の原因は、**Rust に「装備 1 件」を表す型が無いこと**である。`Item` は items.json の
1 レコードにすぎず、「アイテム + 選択したオグメント + カスタム説明」という装着状態を
表す型は web 側の `EquipSlotData` にしか存在しない。そのため 3 ソースの組み立ては
web 側でしか書けず、経路 A（ステータス計算）と経路 B（ユーザー定義プロパティ、
[ADR 0015](0015-property-sets.md)）で二重化している。

症状として、日本語説明文の条件ラベル（`ペット:` など）を解釈対象から外す修正が
`item_search.rs`——検索モジュール——に入った。解釈規則の置き場所が構造的に自明でない。
装備の解釈をどのモジュールが持つかを決める必要がある。

## Decision Drivers

* 装備の解釈が 1 箇所にあること（[ADR 0010](0010-equipment-interpretation-in-rust.md) の
  決定要因の再確認。当時の懸念は「JS 実装と wasm.rs の手書き転記が並存し、後者が黙って腐る」
  だったが、いま腐りかけているのは経路 A と経路 B の並存）
* 新しい解釈規則の置き場所が自明であること。今回の条件ラベル対応のように、
  規則を追加する側が置き場所を都度判断しなくて済むこと
* 同じ入力（同じ装備の同じ 3 ソース）に対する組み立てが二重化しないこと
* WASM 境界の往復回数。[ADR 0015](0015-property-sets.md) は
  「項目数 × スロット数 × 3 テキストの WASM 呼び出し」を Bad として受け入れている
* UI 互換の維持。`per_slot_stats` / `slot_stats` / `skill_bonus_*` の形は
  [ADR 0016](0016-status-breakdown-modal.md) の内訳モーダルとステータス表示が参照している
* 移植量が個人開発として現実的であること（[ADR 0010](0010-equipment-interpretation-in-rust.md)
  と同じ制約。一度にすべてを移さない）

## Considered Options

1. 現状維持。解釈規則の置き場所は都度判断する
2. 解釈関数を `equip_stats.rs` に集約する（関数の移動のみ。型は作らず、WASM 境界はテキスト単位のまま）
3. `equip.rs` に装備を表す型（`Equip` / `EquipSet`）を置き、解釈をその型の振る舞いとして提供する
    - 3.1. WASM 境界はテキスト単位のまま。3 ソースの組み立ては web 側に残す
    - 3.2. WASM 境界を装備セット単位にし、3 ソースの組み立てとオグメント解決も Rust へ移す

選択肢 2 と 3 は「関数を寄せるだけか、装備という概念に型を与えるか」で分かれる。
3.1 と 3.2 は「web 側に残る組み立てループを許容するか」で分かれる。

## Decision Outcome

選択: **3.2. equip.rs に型を置き、WASM 境界を装備セット単位にする**（採用）。

装備の解釈を「`Equip` に問い合わせると解釈が返る」形にする。解釈の実装（正規表現・
条件ラベル走査）は下位モジュールに残し、`equip.rs` は「どのテキストを集めて、どう合算するか」
を持つ。

**概念モデル**

型の置き方を決めるために、何が同一性を持ち、何が属性で決まるかを先に定める。

| 概念 | 種別 | 同一性 / 内容 |
| --- | --- | --- |
| `EquipSet` | エンティティ（集約ルート） | `{character, job, name}` で同一視される。リネームしても同じセットであり続け、保存・端末間同期（[ADR 0006](0006-login-sync-conflict-resolution.md)）・共有（[ADR 0008](0008-equipset-sharing.md)）というライフサイクルを持つ |
| `Equip` | 値オブジェクト | 属性がすべて等しければ交換可能。識別子を持たない。スロット位置は `EquipSet` が持つマップのキーであり、`Equip` の属性ではない |
| `Item` / `AugmentRank` | マスタデータ（集約の外） | ゲーム側の定義。`Item` の同一性は `id`。読み取り専用 |
| `custom_description` | 値オブジェクト | ユーザー入力の文字列 |
| `EquipStats` | 値オブジェクト | 解釈の結果 |

ここから次の帰結が出る。

- **`Equip` は `Item` を内包せず、`item_id` で参照する。** オグメントも同様に、持つのは
  実体ではなく `aug_path` / `aug_rank` という「選択」で、実体は `augments.json` 側のマスタ。
  集約をまたぐものは id で参照する、という原則にそろえる。
  web の `EquipSlotData` が `name_en` / `name_ja` / `description_ja` / `skill` を抱えているのは
  マスタの部分コピーであり、`items.json` の更新で localStorage の値とズレる
  （`equip-sets-store.ts` に `skill` を後から補完する互換処理があるのはその副作用）。
  Rust 側は同じ轍を踏まない。
- **`Equip` は不変で、ミューテータを持たない。** 解釈（`Equip::stats()` など）は
  自身とマスタだけで決まる純関数として書く。
- **スロット横断の集計は `EquipSet` の責務。** 武器スロット装備の武器スキルをそのスロット専用に
  振り分ける規則は、単一の `Equip` では決められず集約ルートが知るべき情報である。

**モジュール構成**

```
equip.rs        Equip / EquipSet と、その解釈 API（本 ADR で新設）
  ↓ 解釈の実装を使う
equip_stats.rs  説明文 → 数値（英語の固定 26 種抽出・日本語の任意名抽出・条件ラベル走査）
items.rs        items.json / augments.json の参照
item_search.rs  検索（説明文ステータスソートで equip_stats の抽出を呼ぶ）
```

**具体的なルール**

- `rust/src/equip.rs` を新設し、次の型を置く。
  - `Equip` — 装備 1 枠。解釈に必要なフィールドのみ持つ
    （`item_id` / `aug_path` / `aug_rank` / `custom_description`）。
    web の `EquipSlotData` にある `name_ja` / `description_ja` / `skill` は
    `item_id` から引ける UI キャッシュなので持たない。
  - `EquipSet` — スロットキー → `Option<Equip>`。
- 解釈は `Equip` / `EquipSet` の振る舞いとして公開する。
  - 3 ソースの解決（アイテム説明文・オグメント文・カスタム説明）は `Equip` が行う。
    オグメント文の解決（`aug_path` / `aug_rank` → テキスト）は `items::augments_by_item_id`
    を使い、web の `getAugmentText` と同じ結果を返す。
  - スロット別集計（武器スロット装備の武器スキルはそのスロット専用、それ以外は全スロット共通）は
    `EquipSet` が行う。現在 `equip-bonuses.ts` が持っている規則をそのまま移す。
- 抽出の実装は `equip_stats.rs` に集約する。`item_search.rs` の
  `extract_stat_from_description` と `conditional_label_scopes`（および対応するテスト）を
  `equip_stats.rs` へ移し、`item_search.rs` は説明文ステータスソートでそれを呼ぶだけにする。
- WASM 境界は装備セット単位にする。`equip_set_bonuses(equipSet)` /
  `equip_set_property_values(equipSet, terms)` を追加し、web 側の 2 つの組み立てループを
  1 回の呼び出しに置き換える。`get_item_by_id` などデータ参照 API は変更しない。
- `getAugmentText`（web）は装備カードとモーダルの表示にも使っているため残す
  （[EquipGrid.tsx](../../web/src/equip/EquipGrid.tsx) /
  [EquipSelectModal.tsx](../../web/src/equip/EquipSelectModal.tsx)）。解釈の入力としては使わない。
- 英語経路（`description_en` + JA→EN 変換）と日本語経路（生の日本語）の統一は
  **本 ADR では決めない**。本 ADR は置き場所だけを決める。統一は別 ADR で扱う
  （More Information 参照）。

**移行の順序**

[ADR 0010](0010-equipment-interpretation-in-rust.md) と同じく段階的に進める。
各段階でテストが通る状態を保つ。

1. `item_search.rs` の抽出関数を `equip_stats.rs` へ移し、抽出の実装を 1 モジュールに集約する
   （振る舞いは不変）
2. `equip.rs` を新設し、`EquipSet` のステータス集計を実装して `equip-bonuses.ts` を
   WASM 呼び出しに置き換える
3. `EquipSet` のプロパティ値算出を実装し、`user-item-values.ts` を WASM 呼び出しに置き換える
4. 不要になった WASM 関数（`extract_all_stats` / `extract_skill_bonuses` /
   `extract_named_stat` / `sum_stats` / `empty_stats`）を削除する。
   Rust 内部の同名関数（`equip_stats::extract_all_stats` など）は残る

手順 2 と 3 の間は経路 A と B が別の粒度で並存する。ここは移行期間として許容する。

**2026-08-14 時点で手順 1〜4 はすべて完了した。** 実績:

- 手順 2 で web 側の JA→EN 変換（`convertAugmentJaToEn` と 88 件の `AUGMENT_JA_TO_EN`）が
  未使用になったため削除した。変換表は Rust 側の 1 つだけになった。
- 手順 4 では `empty_stats` も削除した。未ロード時のガードは
  `equip_set_bonuses({slots:{}})`（空セット）で代替できるため。これで結果の形が常に同じになり、
  参照側が未ロード時だけ形の違いを気にせずに済む。
- 装備関連の WASM 境界は `equip_set_bonuses` と `equip_set_property_values` の 2 つだけになった。
- `sum_stats` の削除に伴い `EquipStats::set_from_map` / `set_by_key`（キー名のマップから
  構造体を復元する、`entries()` の逆操作）も未使用になったため削除した。JS 側が中間の
  ステータスオブジェクトを持つ場面が無くなり、マップ → 構造体の変換自体が不要になったため。
  対応する `set_from_map_roundtrips_every_field` テストも検証対象ごと消えた
  （[ADR 0010](0010-equipment-interpretation-in-rust.md) の Confirmation を追従）。

### Consequences

* Good: 解釈規則の置き場所が自明になる。今回の条件ラベル対応のような修正が検索モジュールに
  紛れ込む構造的な理由が消える。
* Good: 3 ソースの組み立てとオグメント解決の二重化が解消する
  （`equip-bonuses.ts` と `user-item-values.ts` の同型ループが 1 箇所になる）。
* Good: WASM 往復が装備セットあたり 1 回に減り、[ADR 0015](0015-property-sets.md) が
  Bad に挙げた「項目数 × スロット数 × 3 テキストの WASM 呼び出し」が解消する。
* Good: 経路 A と B の差（言語・抽出方式・条件ラベルの扱い）が Rust の 1 モジュール内に
  並んで見えるようになり、次の統一判断の材料になる。
* Good: JA→EN 変換表（88 件）の二重管理が解消する。手順 2 で web 側が未使用になり削除した。
  `docs/tech-debt/mirrored-constants-in-tests.md` が挙げる「定義を 2 箇所に持つ」型の
  問題が 1 つ減る。
* Bad: web から Rust への移動量が大きい。とくに `equip-bonuses.ts` のスロット別集計規則
  （武器スロット × 武器スキルの振り分け）は暗黙知が多く、移す際に落としやすい。
* Bad: `Equip` と web の `EquipSlotData` が二重定義になる。後者は localStorage / Supabase の
  永続形でもあるため 1 つにはできない。フィールド追加時に両方を触る必要が生じる。
* Bad: 移行期間中は経路が並存し、その間の不整合は自動では検出されない。
* Neutral: 装備の「解釈」と「検索」を別モジュールに分ける
  [ADR 0010](0010-equipment-interpretation-in-rust.md) の分割は維持する。
  `item_search.rs` は検索専用に戻るだけで、責務は変わらない。
* Neutral: [ADR 0010](0010-equipment-interpretation-in-rust.md) の
  「ユーザーのカスタム入力は実行時にしか解釈できないため、JS は文字列をそのまま WASM に渡す」は
  維持される。渡す単位が文字列から `Equip` を含む構造体に変わるだけで、
  カスタム入力を実行時に解釈する前提は変わらない。

### Confirmation

現在あるチェック（移行の各段階で通ることを条件にする）:

* `cargo test` — `rust/src/equip_stats.rs` の抽出テスト 38 件。内訳は英語側の項目ごとの
  手書きアサーション 33 件（うち 1 件は下記の全件突き合わせで、通常はスキップされる）と、
  手順 1 で `item_search.rs` から移した日本語側の 5 件
  （`extract_stat_handles_colon_and_fullwidth` / `extract_stat_handles_japanese_property_names` /
  `extract_stat_skips_conditional_label_scope` / `conditional_label_scope_ends_at_line_break` /
  `conditional_label_scope_ignores_ascii_labels`）。
* `cargo test` — `rust/src/equip.rs` の 18 件。`Equip` の値としての同値性、3 ソースの解決
  （アイテム説明文・JA→EN 変換・空ソースのスキップ・オグメントは経路とランク両方が要る）、
  スキル振り分けの 4 分岐（武器スロット × 武器スキル / 武器スロット × 非武器スキル /
  非武器スロット × 武器スキル / `range` → `ranged` バケツ）、`per_slot_stats` の総和一致、
  装備のあるスロットは抽出結果が空でもキーを持つこと。
  `property_values` はスロット・ソース間の合算、日本語のまま抽出すること、疎であること、
  テキストごとに最初の一致 1 件だけを取ることを検証する。
* `npm run test:unit`:
  - `web/src/equip/equip-bonuses.test.ts` — WASM 境界を通した集計の 7 ケース
    （カスタム説明の JA→EN 変換・複数スロット合算・武器スキルのスロット別バケツ・
    `per_slot_skill_bonuses` / `slot_stats` / `per_slot_stats`）。
  - `web/src/propsets/user-item-values.test.ts` — 実 WASM でのスロット横断合算、日本語テキストを
    JA→EN 変換せず抽出すること、条件ラベル配下を拾わないこと。
* `npm run test:smoke` — 「オーグメント選択でテキストとステータスが更新される」
  「スロット検索で装備を選び、装備セットを保存できる」
  「カスタムプロパティセットを作成でき、選択が装備セットの保存で記憶される」
  「内訳モーダルが開ける (ステータス / プロパティセット)」の 4 ケースが、
  経路 A / B の両方をブラウザ上で通す。

検証されていないもの:

* `equip_stats.rs` の `conformance_with_js_over_all_items` は **`JS_STATS` 環境変数が無いと
  スキップする**。移植元の JS 実装と期待値生成ハーネスは削除済みで、現在この全件突き合わせは
  走っていない。ただしテスト本体は期待値 JSON を与えれば動く形で残っているため、
  抽出ロジックを大きく変えるときの前後比較には使える。
* スロット別集計規則（武器スロット × 武器スキルの振り分け）の**継続的な**全件検証は無い。
  自動テストがカバーするのは上記のケースに限られる。
  手順 2 と 3 では移行時の一度きりの確認として、旧 JS 実装を別名で残したまま新旧の出力を
  突き合わせる使い捨てテストを書き、それぞれ実データ 4,015 装備（16 スロットに配って比較）、
  オグメント 200 装備 × 3 ランク、カスタム説明、空スロットで全一致を確認してから
  旧実装を削除した（手順 3 は 18 個のプロパティ名で比較）。
  この使い捨てテストはリポジトリに残していない。

## Pros and Cons of the Options

### 1. 現状維持

* Good: 変更コストがゼロ。
* Bad: 解釈規則の置き場所が判断依存のままになる。実際に条件ラベル対応が検索モジュールに入った。
* Bad: 経路 A と B の二重化が残り、片方だけ直したときのズレが増え続ける
  （現に同じ装備でステータス表示とユーザー定義プロパティが違う値を出しうる）。

### 2. 解釈関数を equip_stats.rs に集約する

* Good: 変更が小さい。関数の移動と参照の差し替えだけで済む。
* Good: 「説明文の解釈は equip_stats」という一文で置き場所を説明できるようになる。
* Bad: 3 ソースの組み立ては web 側に残るため、二重化は解消しない。
* Bad: 「装備」という概念が Rust に無いままなので、`equip_stats` は
  「文字列を受け取る関数の置き場」以上にならない。装備に紐づく解釈
  （オグメント解決、スロット依存の扱い）を置く場所が引き続き無い。

### 3. equip.rs に型を置き、解釈をその振る舞いにする（採用）

* Good: 装備という概念が型として存在するので、装備に紐づく解釈の置き場所が一意に決まる。
* Good: 呼ぶ側が「どのテキストをどう組み立てるか」を知らなくてよくなる。
* Bad: 型を新設するぶん、選択肢 2 より変更量が大きい。

#### 3.1. WASM 境界はテキスト単位のまま

* Good: web 側の変更が不要。Rust 内部の整理だけで完結する。
* Bad: 3 ソースの組み立てが web 側に残るため、型を作った利点の大半（二重化の解消・
  オグメント解決の一元化・往復回数の削減）が得られない。
* Bad: `Equip` が Rust 内で誰にも使われない型になりかねない。

#### 3.2. WASM 境界を装備セット単位にする（採用）

* Good: 二重化・オグメント解決・往復回数の 3 つが同時に解消する。
* Good: web 側は「装備セットを渡して結果を受け取る」だけになり、
  [ADR 0010](0010-equipment-interpretation-in-rust.md) の「解釈は Rust 側」がより素直な形になる。
* Bad: `equip-bonuses.ts` のスロット別集計規則を Rust へ移す必要があり、移行の山が大きい。
* Bad: 移行期間中は経路が並存する。

## More Information

* [ADR 0009](0009-embed-item-data-in-binary.md) — 装備データを WASM バイナリに埋め込む決定。
  Rust 側がデータを持つ前提は本 ADR でも変わらない。
* [ADR 0010](0010-equipment-interpretation-in-rust.md) — 装備の解釈と検索を Rust に移す決定。
  本 ADR はその「解釈は 1 箇所」という決定要因を、モジュール構成として実現し直すもの。
  同 ADR の移植範囲・移行順序の記述は本 ADR で置き換わらない（完了済みの経緯として残る）。
* [ADR 0015](0015-property-sets.md) — ユーザー定義プロパティ。経路 B の出所であり、
  Bad に挙げた WASM 呼び出し回数は本 ADR の手順 3 で解消する。
* [ADR 0016](0016-status-breakdown-modal.md) — 内訳モーダル。`per_slot_stats` /
  `per_slot_skill_bonuses` の形はこの ADR が参照するため、移行で壊さない。
* フォローアップ: **英語経路と日本語経路の統一**。現在、同じ装備でも
  経路 A（`description_en` + JA→EN 変換、固定 26 種、条件セグメント未対応）と
  経路 B（生の日本語、任意名、条件ラベル対応）で異なる値が出る。統一先は日本語側が有力
  （オグメントとカスタム説明は元から日本語で、経路 A のためだけに
  `convertAugmentJaToEn` の変換テーブルを通しており、そこが欠損源になる。
  またユーザー定義プロパティは任意語を扱うため原理的に英語へ寄せられない）。
  条件ラベルの扱いは [docs/knowledge/items/description_labels.md](../knowledge/items/description_labels.md)、
  英語側に残る既知の癖は [docs/tech-debt/equip-stats-js-quirks.md](../tech-debt/equip-stats-js-quirks.md) を参照。
  本 ADR の完了後に別 ADR で扱う。
