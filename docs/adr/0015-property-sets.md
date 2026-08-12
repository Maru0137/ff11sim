---
status: accepted
date: 2026-08-12
decision-makers: Akira Maruoka
---

# 0015. 用途別ステータス表示をプロパティセットとしてユーザー定義可能にする

## Context and Problem Statement

装備編集画面の用途別ステータス表示は 19 個のサブタブとしてハードコードされており
（[ADR 0014](0014-equipset-grid-modal.md) 時点の構成）、ユーザーが自分の用途に
合わせて表示項目を選ぶことができなかった。表示する項目リスト（プロパティセット）を
ユーザーが定義・登録でき、装備セットごとに最後の選択を記憶する仕組みを載せるにあたり、
(a) 表示 UI、(b) 項目値の出所、(c) ユーザー定義項目の抽出方式、(d) 永続化の形、
(e) 選択記憶の置き場所を決める必要がある。

## Decision Drivers

* 既存 19 タブ（テンプレート）の表示内容を壊さないこと
* タブ数が 19 + ユーザー定義で増え続けても UI が破綻しないこと
* 装備の解釈（説明文からの抽出）は Rust 側に置くこと（[ADR 0010](0010-equipment-interpretation-in-rust.md)）
* 装備セット同様、ログイン時は端末間で共有できること（[ADR 0005](0005-localstorage-default-supabase-optin.md)）
* 「最後に選択したセット」のような UI 状態の書き込みが Supabase への全件 upsert を誘発しないこと

## Considered Options

1. 表示 UI
    - 1.1. サブタブバーのままユーザー定義タブを足す
    - 1.2. ドロップダウン（テンプレート/カスタムの optgroup）+ 管理モーダル
2. カスタムセットの項目値の出所
    - 2.1. StatusView.values の既存 DOM id（statAaStp など）を参照する
    - 2.2. カタログにリゾルバ関数を持たせ {equip, totalStats, derived} から算出する
3. ユーザー定義項目（文字列）の抽出方式
    - 3.1. JA→EN 変換（AUGMENT_JA_TO_EN）を通して既存 extract_all_stats に流す
    - 3.2. item_search の任意名抽出を extract_named_stat として WASM 公開し、日本語のまま抽出する
4. 永続化の形
    - 4.1. 装備セット式: セットごとに行へ分解し、名前を識別キーにする
    - 4.2. 1 ユーザー 1 行の jsonb ドキュメント（sets + userItems）、セットは UUID で識別
5. 装備セットごとの選択記憶の置き場所
    - 5.1. 装備セットレコード（data jsonb）に載せて同期する
    - 5.2. ローカル専用の localStorage キー（storage facade の外）

## Decision Outcome

選択: **1.2 + 2.2 + 3.2 + 4.2 + 5.2**。

- テンプレート = 現行 19 タブ。表示 JSX（SubtabContents）は変更せず、
  選択 id を `template:<subtab-id>` で参照する。
  「複製して編集」は catalog.ts の `TEMPLATE_ITEM_IDS` でフラットな項目リストに
  変換する（表形式は再現しない。基本 9 ステ等カタログ外の行は落ちる）。
- 旧タブバーにあった魔法系タブのジョブ連動フィルタ（該当魔法スキルを持たない
  ジョブ構成では非表示）は撤去し、全テンプレートを常時選択可能にする。
  ジョブ構成の一時変更でドロップダウンの構成や選択記憶の復元が揺れるのを避けるため。
  該当スキルを持たない場合、スキル値は従来どおり '-' 表示になる。
- カタログ（web/src/propsets/catalog.ts）には基本 9 項目（HP〜CHR）と左の常時表示
  テーブル既出の項目（防御力/回避/魔防/魔回避/ヘイスト/被ダメージ系）は載せない。
- ユーザー定義項目は「文字列+N」の完全前方一致・各テキスト最初の一致のみの最小仕様。
  抽出対象は description_ja + オーグメント文 + カスタム説明の 3 ソースで、
  いずれも日本語のまま extract_named_stat に渡す（JA→EN 変換を通さない）。
- 選択記憶は `ff11sim_propset_selection`（キー: `character|job|name`）。装備セットの
  リネームで移行、削除で破棄する。共有閲覧モードでは記憶しない。

### Consequences

* Good: ユーザーが用途別表示を自作でき、テンプレートは従来どおり動く。
* Good: 抽出ロジックが Rust 1 箇所に保たれ、検索ソート（desc_stat）と同一挙動になる。
* Good: セット切替は StatusView.propertyValues（全項目を毎回計算）を引くだけで即時。
* Bad: 全カタログ項目 + 全ユーザー定義項目を毎回計算するため、ユーザー定義項目が
  増えると再計算コストが線形に増える（項目数 × スロット数 × 3 テキストの WASM 呼び出し）。
* Bad: カタログのリゾルバは compute.ts の式と一部重複する（derived で緩和）。
  compute.ts 側の表示式を変えるときはカタログとの不一致に注意が必要。
* Bad: 「二刀流効果アップ」のような数値を伴わない表記ゆれは拾えない
  （augments.json のような補完データで将来対応。[ADR 0004](0004-augment-data-managed-separately.md) 参照）。
* Neutral: 1 ユーザー 1 行のため多端末同時編集はドキュメント全体の後勝ちになる
  （既存 repo の全件置換と同じ粒度）。
* Neutral: 選択記憶はローカル専用のため端末間では引き継がれない（消失許容の UI 状態と割り切る）。

### Confirmation

* `cargo test`（rust/src/item_search.rs の
  `extract_stat_handles_japanese_property_names`）が日本語プロパティ名の抽出仕様
  （全角正規化・途中一致・最初の一致のみ）を検証する。
* `npm run test:unit`:
  - `web/src/propsets/catalog.test.ts` — カタログ id の一意性、`user:`/`template:`
    プレフィクスとの非衝突、除外項目（基本 9 + 左テーブル既出）の不在、
    `TEMPLATE_ITEM_IDS` の参照整合。
  - `web/src/propsets/types.test.ts` — PropsetDoc の正規化と項目削除ヘルパー。
  - `web/src/propsets/user-item-values.test.ts` — 実 WASM でのスロット横断合算と、
    日本語テキストを JA→EN 変換せず抽出すること。
* `npm run test:smoke`（tests/smoke.spec.js「カスタムプロパティセットを作成でき、
  選択が装備セットごとに記憶される」）が 管理モーダルでの作成 → グリッド表示 →
  ユーザー定義項目の抽出値表示 → リロード後の選択復元 を通しで検証する。

検証されていないもの:

* Supabase 側 repo と sync ブロックの自動テストは無い（[ADR 0005](0005-localstorage-default-supabase-optin.md) と同様）。
* migration `supabase/migrations/003_property_sets.sql` は SQL Editor での手動適用が必要。

## Pros and Cons of the Options

### 1.1. サブタブバーのままユーザー定義タブを足す

* Good: 既存 UI の延長で実装が小さい。
* Bad: 19 + ユーザー定義でタブが折り返し、一覧性が崩れる。
* Bad: タブの編集・削除 UI をタブバー上に同居させることになり操作が窮屈。

### 1.2. ドロップダウン + 管理モーダル（採用）

* Good: 件数が増えても場所を取らない。テンプレート/カスタムを optgroup で区別できる。
* Good: 編集・削除・複製の操作をモーダルに分離できる。
* Bad: 全選択肢を一覧できず、切替が 2 クリックになる。

### 2.1. StatusView.values の既存 DOM id を参照する

* Good: 値の定義が 1 箇所（compute.ts）に閉じる。
* Bad: 同一プロパティがサブタブごとに別 id（statAaStp / statMwsStp / ...）で重複しており、
  どれを「正」とするかがテンプレート描画の内部実装への結合になる。
* Bad: テンプレート側の id 変更でカタログが静かに '-' になる。

### 2.2. リゾルバ関数で算出する（採用）

* Good: 論理プロパティ 1 つにつき定義が 1 つ。テンプレートに無い項目も出せる。
* Good: 表示整形（numOrDash 等）を compute.ts と共有できる（status/format.ts）。
* Bad: compute.ts の合成値（魔命合計など）と式が重複する。derived として
  compute.ts から受け取ることで緩和するが、二重定義の芽は残る。

### 3.1. JA→EN 変換して extract_all_stats に流す

* Good: 既存パイプラインそのままで新規 WASM 公開が要らない。
* Bad: extract_all_stats は既知の固定キー（78 項目）しか返せず、
  「二刀流」のような任意名は変換テーブルに無い限り拾えない。本要件と根本的に合わない。

### 3.2. extract_named_stat を WASM 公開する（採用）

* Good: item_search の抽出（全角正規化・日本語対応済み）をそのまま使い、
  検索ソートと挙動が一致する。実装は委譲 1 関数のみ。
* Good: 解釈が Rust 側に保たれる（[ADR 0010](0010-equipment-interpretation-in-rust.md)）。
* Bad: 各テキスト最初の一致のみで、1 装備の説明文中に同名が 2 回出ても合算しない
  （実データではほぼ無い並びのため許容）。

### 4.1. 装備セット式の行分解・名前キー

* Good: 既存 repo の形をそのまま踏襲できる。
* Bad: 選択記憶がセットを参照するため、リネーム = キー変更で参照が壊れる
  （装備セットで実際に起きている問題の再生産）。
* Bad: 共有・並び順が不要なのに複合キー削除ループなどの機構だけ増える。

### 4.2. 1 ユーザー 1 行 jsonb ドキュメント + UUID 識別（採用）

* Good: repo が load/save の 2 関数で済み、UUID 参照はリネームで壊れない。
* Good: ユーザー定義項目を同じドキュメントに同居でき、同期ブロックが 1 つで済む。
* Bad: 1 セットの編集でもドキュメント全体を書き込む（既存 repo の全件置換と同粒度）。

### 5.1. 装備セットレコードに載せて同期する

* Good: 端末間で選択まで引き継がれる。
* Bad: ドロップダウン変更のたびに saveEquipSets（ログイン時は全件 upsert）が走る。
* Bad: UI 状態が shared_equipsets の共有ペイロードに漏れる。

### 5.2. ローカル専用 localStorage キー（採用）

* Good: 書き込みが軽く、共有データを汚さない。
* Good: 消えても実害のない UI 状態として扱える（[ADR 0005](0005-localstorage-default-supabase-optin.md)
  の UI プリファレンス層）。
* Bad: 端末間では選択が引き継がれない。

## More Information

* 前提: [ADR 0005](0005-localstorage-default-supabase-optin.md)（永続化の層構造。
  本 ADR で「UI プリファレンス層」の注記を追加）、
  [ADR 0006](0006-login-sync-conflict-resolution.md)（同期の競合解決 = Supabase 優先）、
  [ADR 0010](0010-equipment-interpretation-in-rust.md)（装備解釈は Rust）、
  [ADR 0014](0014-equipset-grid-modal.md)（Mantine Modal / モーダル z-index / 共有モードの構造的除外）
* 残課題: 数値を伴わない表記ゆれ（「二刀流効果アップ」等）の補完データ
  （[ADR 0004](0004-augment-data-managed-separately.md) の方式に倣った別ファイル）を導入するか。
