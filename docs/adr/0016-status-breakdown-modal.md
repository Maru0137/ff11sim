---
status: accepted
date: 2026-08-13
decision-makers: Akira Maruoka
---

# 0016. ステータス / プロパティ値のソース別内訳をモーダル表示する

## Context and Problem Statement

装備編集画面のステータス値・プロパティセット値（[ADR 0015](0015-property-sets.md)）は
合計値しか表示されず、合計の検算や寄与元の確認ができなかった。行 = 寄与元
（装備 16 部位 / 種族 / メインジョブ / サポートジョブ / メリポ / JP / ギフト / ML）、
列 = 項目の内訳テーブルを「内訳」ボタン → モーダルで表示するにあたり、
(a) キャラ由来の寄与値の算出方式、(b) どの寄与元にも帰属できない項
（防御の VIT×1.5 + Lv + α、魔防の基準値 100 など）の扱い、(c) 小数を持つ
基本ステータス（種族/メイン/サポは合算後に切り捨て）の表示、(d) WASM が返す
内訳データの形、を決める必要がある。

## Decision Drivers

* 内訳と合計値の計算が将来乖離（二重集計ドリフト）しないこと — 片方だけ修正される事故が最も怖い
* 検算用途なので、表の各列が「なぜその合計になるか」を正直に説明すること
* 装備の解釈は Rust 側に置く方針（[ADR 0010](0010-equipment-interpretation-in-rust.md)）と整合すること
* StatusResult の全項目（約 150）を分解する必要はなく、UI に出る列だけ賄えばよいこと
* ジョブ特性のメイン/サポ採用（絶対値の大きい方）は実計算と同一ロジックで振り分けること

## Considered Options

1. キャラ由来の寄与値の算出方式
    - 1.1. JS 側の差分法（profile からメリポ/ML 等を 1 要素ずつ抜いて再計算し差を取る）
    - 1.2. Rust に内訳 API を追加し、既存の集計関数を分解版への委譲にリファクタする
2. 帰属できない項（レベル・ステータス・スキル由来）の扱い
    - 2.1. 剰余方式: 基礎 = 合計 −（装備 + 既知のキャラ行）
    - 2.2. 「基礎」行を解析的に計算し、Σ行 == 合計の恒等式はテストで担保する
3. 基本ステータスの小数（種族 D の STR 37.5、サポは 0.25 刻み）の表示
    - 3.1. 逐次切り捨てで整数に配分する（列を足すと必ず合計に一致）
    - 3.2. 小数のまま表示し、「合算後に切り捨て」の脚注を付ける
4. WASM が返す内訳データの形
    - 4.1. ソースごとの固定 struct（StatusResult のサブセットを 7 レコード分）
    - 4.2. 疎マップ `rows: ソース → 列キー → 値`（列キーは UI と 1:1、0 は含まない）

## Decision Outcome

選択: **1.2 + 2.2 + 3.2 + 4.2**。

- `Chara::status()` / `job_trait_total()` を `status_parts()` / `job_trait_breakdown()`
  への委譲に書き換え、武器スキル解決も `combat_skills` として抽出して
  `chara_to_status_result` と内訳 (`breakdown::chara_breakdown`) で共有する。
  集計ロジックの実体は常に 1 箇所。
- 「基礎」行は `calc_defense(vit, lv, 0)` のように装備項 0 で解析的に計算する。
  剰余は使わない（帰属ミスを黙って吸収し、検算が検証にならないため）。
- 内訳の対象列は「ステータステーブルの項目 + カタログの charKey 参照項目」に限定。
  gift の `physical_attack` → 列キー `attack` のような写像は Rust 側で行い、
  JS には UI の列と 1:1 のキーだけを見せる。
- カタログ（[ADR 0015](0015-property-sets.md)）の各項目に `breakdown` メタ
  (`equipKey` / `charKey`) を追加。メタなし = 内訳非対応として列から除外し、
  非対応は `BREAKDOWN_UNSUPPORTED_IDS` に明示する（実効魔命などのスロット依存合成値）。
- スキル値列（同日改訂で対応）: breakdown メタに `skillKey`（魔法スキル: 固定キー）/
  `weaponSlot`（武器スキル: スキル種別は装備中の武器から実行時解決）を追加。
  装備行は `calculateEquipSetBonuses` の `per_slot_skill_bonuses`（全スロット別
  スキルボーナス）から引き、武器スキル列では他の武器スロットの武器スキルボーナスを
  除外する（equip-bonuses のバケツ分けと同一規則）。キャラ由来行は Rust の
  `skill_<Key>` / `main_weapon_skill` 等の列で、`effective_skill_parts` により
  基礎（素キャップまでの値）/ メリポ / ML に分解する（メリポ・ML はキャップ
  引き上げのため、寄与は「キャップを 素 → +メリポ → +ML の順に広げたときの
  表示値の増分」として帰属。キャラ値がキャップに届いていない分は 0）。
  `effective_skill` は分解版への委譲。魔法系はギフト行も生成する。
  表示値が '-'（ジョブ未習得 / 武器未装備）の列は装備ボーナスも適用されないため
  装備行も抑制する。
- 装備 16 部位の行は JS 側で完結する（`calculateEquipSetBonuses` の
  `per_slot_stats`）。抽出は元からスロット単位なので Rust 変更は不要。
- モーダルの合計行はパネル表示値（`StatusView.values` / `propertyValues`）を
  そのまま参照し、内訳の合計とパネル表示の定義齟齬を構造的に防ぐ。
  内訳の計算はモーダルを開いた時に遅延実行する。

### Consequences

* Good: 内訳と合計が同じ関数（status_parts / job_trait_breakdown / combat_skills）から
  出るため、計算式の変更が自動的に両方へ反映される。
* Good: 基礎行が解析値なので、新しい寄与源の追加漏れは恒等式テストの失敗として顕在化する。
* Good: 小数表示はゲーム仕様（0.5/0.25 刻み → 合算後切り捨て）に忠実で、情報が失われない。
* Bad: `chara_to_status_result` に新しい寄与源を足すときは `chara_breakdown` にも
  同じ行を足す必要がある（両ファイルの doc コメントと恒等式テストで担保）。
* Bad: 内訳列の追加には Rust（列キー）と catalog（メタ）の両方の変更が要る。
* Neutral: 種族/メイン/サポの小数行があるため、列の単純合計と合計行は最大 0.75 ずれる
  （脚注で説明）。

### Confirmation

* `rust/src/breakdown.rs` の `mod tests`: 恒等式（基本 9 ステは
  `floor(Σキャラ行) + 装備 == StatusResult`、防御/回避/魔防/プロパティ列は
  `Σ行 + 装備 == StatusResult`、スキル値列は `Σキャラ行 == 表示スキル値`）、
  特性のメイン/サポ振り分け、メリポ/ギフトの行帰属を検証。
* `web/src/propsets/catalog.test.ts`: 全カタログ項目が `breakdown` メタを持つか
  `BREAKDOWN_UNSUPPORTED_IDS` に明示されていることを強制（新項目追加時に対応可否の判断を要求）。
* `web/src/status/breakdown.test.ts`: 列組み立て（非対応項目の除外、ユーザー定義項目、
  デスの tenacity 非適用、スキル値列の skillKey / weaponSlot メタ）と行×列モデル
  （0 → '-'、負値保持、小数保持、合計行の参照、武器スキル列の他武器スロット除外、
  表示 '-' 時の装備行抑制）を検証。
* `web/src/equip/equip-bonuses.test.ts` / `web/src/propsets/user-item-values.test.ts`:
  スロット別集計（per_slot_stats の総和一致、per_slot_skill_bonuses のスロット別保持）を
  実 WASM で検証。
* `tests/smoke.spec.js`「内訳モーダルが開ける」: JS→WASM シグネチャとテーブル描画の生存確認。

## Pros and Cons of the Options

### 1.1. JS 側の差分法

* Good: Rust の変更が不要
* Bad: ギフト/JP は `total_jp_spent()` 経由でジョブ特性ランクにも効くため、
  1 要素を抜いた再計算では寄与が分離できない（BLU の特性効果アップ等）
* Bad: 再計算 7 回 + 差分の意味づけが暗黙的で、計算変更に追従できない

### 1.2. Rust 内訳 API + 委譲リファクタ（採用）

* Good: 集計の実体が 1 箇所になり、既存の数値回帰テストが委譲リファクタの
  挙動不変を証明する（f32 加算順序を保てば floor 境界も不変）
* Good: 特性の採用判定（絶対値比較、同値はメイン）を実計算と共有できる
* Bad: wasm-pack の再ビルドが必要。関数追加時は web/js/wasm.js と
  web/src/wasm.ts の再エクスポートも更新する（[ADR 0012](0012-react-ui-framework.md) の不変条件）

### 2.1. 剰余方式

* Good: 実装が最小で、表は常に合計と一致する
* Bad: ソースの帰属バグ（ギフトの入れ忘れ等）を基礎行が黙って吸収し、
  「常に一致する検算」は検証として機能しない

### 2.2. 解析的な基礎行 + 恒等式テスト（採用）

* Good: 帰属ミスがテスト失敗（またはランタイムでの合計不一致）として見える
* Bad: 基礎項の式（防御の α、回避のスキル項など）を breakdown 側にも書く必要がある
  （calc_* 関数を装備項 0 で呼ぶ形に限定して重複を最小化）

### 3.1. 整数に配分

* Good: 列の合計が常に一致して見た目が揃う
* Bad: 配分順序という恣意性が入り、実際の値（37.5 等）と 1 ずれる行が出る

### 3.2. 小数のまま表示（採用）

* Good: ゲーム仕様の実値で情報が失われない。既存テストのコメント自体が
  この表現（floor(37.50+45.00+13.50)）で書かれている
* Bad: 列の単純合計と合計行が最大 0.75 ずれ、脚注での説明が要る

### 4.1. ソースごとの固定 struct

* Good: 型が明示的
* Bad: フィールド一覧が StatusResult（約 150 項目）と将来ドリフトする。
  GiftBonuses の内部名と UI 列名の写像が JS に漏れる

### 4.2. 疎マップ（採用）

* Good: 列キーが UI と 1:1 で、JS 側は写像なしで引くだけ。0 を含まないので転送が小さい
* Bad: キーが文字列契約になる（catalog の charKey と Rust の列キーの対応は
  カタログ側の doc コメントと恒等式テストで担保）

## More Information

* [ADR 0010](0010-equipment-interpretation-in-rust.md) — 装備解釈を Rust に置く方針
* [ADR 0014](0014-equipset-grid-modal.md) — Mantine Modal は React 専用モーダルに使い、開閉はローカル state
* [ADR 0015](0015-property-sets.md) — プロパティセットとカタログ（本 ADR はカタログに breakdown メタを追加）
* [魔命の用語規約と計算仕様](../knowledge/status/magic_accuracy.md) — 内訳非対応の実効魔命 (`macc_*`) の定義
* モックレビュー: https://claude.ai/code/artifact/056afaa7-d584-4eb8-9ddb-b18a13a8577f
* 改訂 (2026-08-13): スキル値列（武器スキル/魔法スキル）の内訳対応を追加
  （`skillKey` / `weaponSlot` メタ、`per_slot_skill_bonuses`、Rust のスキル値列）。
