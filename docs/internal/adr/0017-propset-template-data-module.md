---
status: accepted
date: 2026-08-13
decision-makers: Akira Maruoka
---

# 0017. プロパティセットテンプレートを宣言的データモジュールで定義する

## Context and Problem Statement

プロパティセットのテンプレート（19 種）は StatusTables.tsx に約 720 行の固定 JSX として
直書きされていた（[ADR 0015](0015-property-sets.md) は「表示 JSX は変更せず」を前提に
していた）。その後、魔命スキル/実効魔命列の追加、レンジ行の弓術/射撃限定表示、
呪歌の楽器スキル列切替、風水鈴スキル列の条件表示など、同型テーブルへの列追加と
条件分岐が JSX 内に増殖し、テンプレートの追加・変更のたびに 19 個の類似 JSX を
複製・整合させる必要が生じた。テンプレート定義をどう外部化するかを決める。

## Decision Drivers

* テンプレートの追加・変更を「定義の編集」だけで済ませたい（同型 JSX の複製をなくす）
* DOM 互換の維持: `#subtab-*` の div 構造とセルの `id`（= StatusView.values の id）は
  旧実装から引き継いでおり、smoke テストも参照している
* 条件表示（レンジ行/楽器スキル列/風水鈴列）を宣言的に表現できること
* valueId のタイポなど、JSX の型チェックが効かなくなる分の検出手段があること
* 実行時ロードや非エンジニアによる編集は現時点で不要（YAGNI）

## Considered Options

1. JSX 直書きを維持する
2. 宣言的なテンプレート定義 + 汎用レンダラに分離する
    - 2.1. 型付き TS データモジュール（`template-defs.ts`）
    - 2.2. JSON ファイル（`data/*.json`）+ 起動時 fetch

選択肢 2 の分岐は「定義をコードとして型検査するか、データとして実行時ロードするか」。

## Decision Outcome

選択: **2.1. 型付き TS データモジュール**（採用）。

- `web/src/status/template-defs.ts` に全 19 テンプレートを純データ
  （`TemplatePropsetDef`: id / label / group / tables）として定義する。
  他モジュールを import しない純データモジュールとして保つ。
- セル値は `StatusView.values` の id を参照し、描画時の DOM id にもそのまま使う
  （`null` はダッシュ固定セル）。
- 条件表示はフラグ名参照（`TemplateFlag = 'rangedWs' | 'songString' | 'songWind'
  | 'geoHandbell'`）。`visibleIf` はテーブル・行・列のどのレベルにも付けられ、
  StatusTables.tsx の `SubtabContents` が StatusView 由来の props からフラグを解決する。
- 描画は汎用レンダラ（`TemplateTable`）1 つ。`TEMPLATE_PROPSETS` /
  `TEMPLATE_PROPSET_GROUPS` は template-defs から StatusTables 経由で再エクスポートし、
  参照側（StatusPanel / PropsetManageModal）は変更しない。
- [ADR 0015](0015-property-sets.md) の「表示 JSX（SubtabContents）は変更せず」は
  本 ADR で置き換える。表示内容・DOM id の互換は維持する。

### Consequences

* Good: テンプレートの変更が template-defs.ts（+ 値の算出は compute.ts、複製対応は
  catalog.ts の TEMPLATE_ITEM_IDS）に閉じる。StatusTables.tsx は約 720 行 → 約 80 行の
  レンダラになった。
* Good: 定義が純データなので、構造の整合（セル数 = 列数、DOM id の一意性、
  TEMPLATE_ITEM_IDS との対応）をユニットテストで機械検証できる。
* Bad: JSX の型チェックが効かなくなる。`visibleIf` のフラグ名は `TemplateFlag` union で
  型検査されるが、valueId が compute.ts の出力に実在するかは型でもテストでも
  検証されない（下記 Confirmation の未検証項目）。
* Bad: レイアウトの自由度がスキーマの表現力に制限される。スキーマ外のテーブル
  （colSpan 見出し以外の特殊構造など）が必要になったらスキーマ拡張が要る。
* Neutral: 条件フラグの追加は TemplateFlag / StatusView / SubtabContents の
  3 箇所への追記になる。

### Confirmation

* `web/src/status/template-defs.test.ts`: テンプレート id の一意性、グループが
  定義済みであること、全テーブルで各行のセル数 = 列数、valueId（DOM id）の
  全テンプレート横断での一意性、`TEMPLATE_ITEM_IDS` との 1:1 対応を検証する。
* `npm run typecheck`: `visibleIf` のフラグ名（`TemplateFlag` union）と
  定義の構造（`TemplatePropsetDef`）を型検査する。
* `tests/smoke.spec.js`「保存済みキャラクターのステータスが 0 でなく表示される」:
  `#propsetSelect` でのテンプレート選択 → `#subtab-melee-auto` の active クラス切替
  （DOM 互換の生存確認）を検証する。

検証されていないもの:

* valueId が compute.ts の出力（StatusView.values）に実在するかは自動検証がない
  （計算に WASM が必要なため）。タイポ時は実行時に該当セルが '-' 表示になる。

## Pros and Cons of the Options

### 1. JSX 直書きを維持する

* Good: JSX の構造がそのまま見え、タグ構造は型チェックされる。レイアウトの自由度が最大。
* Bad: 19 テンプレートの同型テーブルを複製し続けることになり、列追加のたびに
  複数タブへ同じ編集を繰り返す（実際に「WSダメ → WSD の改名が 1 タブだけ漏れる」
  類の非対称が発生した）。
* Bad: 条件表示が JSX 内の三項演算子・`&&` として増殖し、テンプレート全体の
  見通しが悪化する。

### 2.1. 型付き TS データモジュール（採用）

* Good: 定義がデータになり、構造検証をテスト化できる。フラグ名・スキーマは
  型検査が効く。
* Good: ビルド時に静的 import されるため、fetch 失敗などの実行時失敗モードを
  追加しない。
* Bad: 編集には TS の知識が要る（現時点の編集者は開発者のみなので許容）。
* Bad: 実行時の差し替え・ユーザー定義テンプレートには使えない（必要になったら
  スキーマを JSON 化して 2.2 へ移行する余地は残る）。

### 2.2. JSON ファイル + 起動時 fetch

* Good: 非エンジニアの編集や実行時差し替えが可能。skills.json 等の既存
  データファイルパターン（[ADR 0009](0009-embed-item-data-in-binary.md) 以前からの
  `data/` 配信）に乗る。
* Bad: 型検査が効かず、スキーマ検証を実行時バリデーション + テストで
  自前実装することになる。
* Bad: fetch の失敗モードとロード順の依存が増える。差し替え需要が現状ないため
  コストに見合わない。

## More Information

* [ADR 0015](0015-property-sets.md) — プロパティセットの導入。本 ADR は同 ADR の
  「表示 JSX（SubtabContents）は変更せず」を置き換える（表示内容の互換は維持）。
* [ADR 0016](0016-status-breakdown-modal.md) — 内訳モーダル。テンプレートの列と
  カタログ項目の対応（TEMPLATE_ITEM_IDS）は内訳の列構成にも使われる。
