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
        - 5.1.1. 選択のたびに保存（upsert）する
        - 5.1.2. 装備セットの保存時のみレコードに載せる
    - 5.2. ローカル専用の localStorage キー（storage facade の外）

## Decision Outcome

選択: **1.2 + 2.2 + 3.2 + 4.2 + 5.1.2**
（5 は当初 5.2 を採用。端末間で選択が引き継がれない不満が実運用で出たため、
2026-08-13 の改訂で 5.1.2 に変更）。

- テンプレート = 現行 19 タブ。表示 JSX（SubtabContents）は変更せず、
  選択 id を `template:<subtab-id>` で参照する
  （※ 表示 JSX 非変更の前提はその後 [ADR 0017](0017-propset-template-data-module.md) で
  宣言的データモジュールに置換。表示内容・DOM id の互換は維持）。
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
  いずれも日本語のまま扱う（JA→EN 変換を通さない）。
  3 ソースの組み立てと抽出は当初 web 側 + `extract_named_stat`（WASM）だったが、
  2026-08-14 に Rust の `EquipSet::property_values` へ移した
  （[ADR 0018](0018-equip-module-owns-interpretation.md) 手順 3。
  web が持つのは「プロパティ名 → 表示 id (`user:<term>`)」の対応だけ）。
- 条件ラベル（`ペット:` `潜在能力:` など）のコロン直後から行末までは
  抽出対象から外す（2026-08-14 追加）。本体に常時乗る値ではないため
  （`防21 ペット:命中+3 モクシャ+3` の命中・モクシャはどちらもペットのもの）。
  **`コンビネーション:` は 2026-08-15 に例外として除外対象から外した**
  （[ADR 0019](0019-japanese-as-single-interpretation-source.md)）。
  除去はステータス計算と共通の処理（`equip_stats::strip_conditional_labels`）で行い、
  両経路が同じ規則になる。
  ラベル判定は「非 ASCII 文字を含むこと」。英語説明文の `DMG:+165 Delay:+240 STR+10` を
  誤認しないための条件で、日本語説明文ではコロン付きラベルが常に対象・条件を表す
  （[docs/knowledge/items/description_labels.md](../knowledge/items/description_labels.md)）。
- 選択記憶は装備セットレコードの `propset_selection`（値は `template:subtab-*` または
  カスタム UUID）。装備セットの保存時のみ書き込まれ、ログイン時は他のセット内容と
  同様に端末間同期される。保存していない選択変更はセット切替・リロードで失われる。
  リネーム・削除・複製はレコードと一体で追随するため専用の移行処理は無い。
  shared_equipsets の共有ペイロードはフィールドを明示列挙して構築するため
  （share-ui.ts）、選択は共有に含まれない。共有閲覧モードでは記憶しない。

### Consequences

* Good: ユーザーが用途別表示を自作でき、テンプレートは従来どおり動く。
* Good: 抽出ロジックが Rust 1 箇所に保たれ、検索ソート（desc_stat）と同一挙動になる。
* Good: セット切替は StatusView.propertyValues（全項目を毎回計算）を引くだけで即時。
* Good: ログイン時は選択記憶も装備セットと一緒に端末間で引き継がれる（2026-08-13 改訂）。
* Bad: 全カタログ項目 + 全ユーザー定義項目を毎回計算するため、ユーザー定義項目が
  増えると再計算コストが線形に増える（項目数 × スロット数 × 3 テキストの抽出）。
  WASM 呼び出し回数だけは装備セットあたり 1 回になった
  （2026-08-14、[ADR 0018](0018-equip-module-owns-interpretation.md) 手順 3）。
  抽出そのものの回数は変わらない。
* Bad: カタログのリゾルバは compute.ts の式と一部重複する（derived で緩和）。
  compute.ts 側の表示式を変えるときはカタログとの不一致に注意が必要。
* Bad: 「二刀流効果アップ」のような数値を伴わない表記ゆれは拾えない
  （augments.json のような補完データで将来対応。[ADR 0004](0004-augment-data-managed-separately.md) 参照）。
* Neutral: 1 ユーザー 1 行のため多端末同時編集はドキュメント全体の後勝ちになる
  （既存 repo の全件置換と同じ粒度）。
* Neutral: 選択記憶は「最後に保存した時点」のスナップショットになる。閲覧のために
  一時的に切り替えた選択が残らないのはスロット変更と同じ「保存で確定」モデルで一貫する。

### Confirmation

* `cargo test`（rust/src/equip_stats.rs の
  `extract_stat_handles_japanese_property_names`）が日本語プロパティ名の抽出仕様
  （全角正規化・途中一致・最初の一致のみ）を検証する。
  条件ラベルの除外は `extract_stat_skips_conditional_label_scope`（ラベル配下を拾わない・
  ラベルより前は拾う）、`conditional_label_scope_ends_at_line_break`（折り返し行は本体に戻る）、
  `conditional_label_scope_ignores_ascii_labels`（英語の `DMG:` 等を誤認しない）で検証する。
* `npm run test:unit`:
  - `web/src/propsets/catalog.test.ts` — カタログ id の一意性、`user:`/`template:`
    プレフィクスとの非衝突、除外項目（基本 9 + 左テーブル既出）の不在、
    `TEMPLATE_ITEM_IDS` の参照整合。
  - `web/src/propsets/types.test.ts` — PropsetDoc の正規化と項目削除ヘルパー。
  - `web/src/propsets/user-item-values.test.ts` — 実 WASM でのスロット横断合算と、
    日本語テキストを JA→EN 変換せず抽出すること。
* `npm run test:smoke`（tests/smoke.spec.js「カスタムプロパティセットを作成でき、
  選択が装備セットの保存で記憶される」）が 管理モーダルでの作成 → グリッド表示 →
  ユーザー定義項目の抽出値表示 → 保存 → リロード後の選択復元 →
  未保存の選択変更が残らないこと を通しで検証する。

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

### 5.1. 装備セットレコードに載せて同期する（2026-08-13 改訂で採用）

* Good: 端末間で選択まで引き継がれる。
* Good: リネーム・削除・複製がレコードと一体になり、キー移行・破棄の専用処理
  （旧 selection-prefs.ts）が不要になる。新規セット作成中の選択も保存時に一緒に残る。
* Neutral: 「共有ペイロードに漏れる」は当初の却下理由だったが、共有ペイロードは
  share-ui.ts がフィールドを明示列挙して構築するため実際には漏れない（改訂時に再評価）。

#### 5.1.1. 選択のたびに upsert する

* Good: 選択した瞬間に記憶され、保存操作が要らない（旧 5.2 と同じ使用感）。
* Bad: ドロップダウン変更のたびに saveEquipSets（ログイン時は全件 upsert）が走る。
  Decision Drivers の「UI 状態の書き込みが全件 upsert を誘発しないこと」に反する。

#### 5.1.2. 装備セットの保存時のみ載せる（採用）

* Good: 書き込みはユーザーの保存操作に相乗りするだけで、追加の upsert が発生しない。
* Good: スロット変更と同じ「保存で確定」モデルに揃い、セマンティクスが一貫する。
* Bad: 保存せずに変えた選択はセット切替・リロードで失われる
  （選択した瞬間に記憶される旧挙動からの後退）。

### 5.2. ローカル専用 localStorage キー（当初採用 → 2026-08-13 改訂で 5.1.2 に置換）

* Good: 書き込みが軽く、共有データを汚さない。
* Good: 消えても実害のない UI 状態として扱える（[ADR 0005](0005-localstorage-default-supabase-optin.md)
  の UI プリファレンス層）。
* Bad: 端末間では選択が引き継がれない。「消失許容」と割り切ったが実運用で不満となり、
  改訂の動機になった。
* Bad: リネーム移行・削除破棄の専用処理が必要だった。

## More Information

* 前提: [ADR 0005](0005-localstorage-default-supabase-optin.md)（永続化の層構造。
  本 ADR で「UI プリファレンス層」の注記を追加）、
  [ADR 0006](0006-login-sync-conflict-resolution.md)（同期の競合解決 = Supabase 優先）、
  [ADR 0010](0010-equipment-interpretation-in-rust.md)（装備解釈は Rust）、
  [ADR 0014](0014-equipset-grid-modal.md)（Mantine Modal / モーダル z-index / 共有モードの構造的除外）
* 残課題: 数値を伴わない表記ゆれ（「二刀流効果アップ」等）の補完データ
  （[ADR 0004](0004-augment-data-managed-separately.md) の方式に倣った別ファイル）を導入するか。
* 後続: [ADR 0016](0016-status-breakdown-modal.md)（カタログに breakdown メタを追加）、
  [ADR 0017](0017-propset-template-data-module.md)（テンプレート描画を宣言的データ
  モジュールに置換）。
* 改訂 (2026-08-13): 選択記憶を 5.2（ローカル専用 localStorage）から 5.1.2
  （装備セットレコードに載せ、保存時のみ書き込み）に変更。別端末で選択が
  引き継がれない不満への対応。旧 `ff11sim_propset_selection` キーからの移行は
  行わない（各セットの初回保存時にレコードへ記憶され直す）。
