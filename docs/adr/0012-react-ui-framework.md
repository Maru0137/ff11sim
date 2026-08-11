---
status: accepted
date: 2026-08-11
decision-makers: Akira Maruoka
---

# 0012. フロントエンド UI フレームワークとして React を採用する

## Context and Problem Statement

Web フロントエンドは素の ES Modules 約 3,500 行（index.html / search.html の
2 エントリ）で書かれ、状態と DOM の同期は手動の DOM 操作
（`createElement` / `innerHTML` が約 90 箇所）で行われている。
インライン script のモジュール化と Vite 導入
（[ADR 0011](0011-vite-build-toolchain.md)）は完了したが、UI フレームワークの
採用は別軸の決定として保留されていた。

今後、計算結果やシミュレーション結果をテーブル・グラフで表現する可能性が高く、
可視化ライブラリのエコシステムまで含めて UI フレームワークを決める必要がある。

## Decision Drivers

* **状態→DOM 同期の宣言化** — 手動 DOM 同期は過去に状態と DOM の不整合による
  本番障害を起こした
  （[docs/tech-debt/inline-script-monolith.md](../tech-debt/inline-script-monolith.md)）
* **可視化（テーブル・グラフ）エコシステム** — ソート・フィルタ付きの結果
  テーブル（ヘッドレステーブル）、ダメージ分布・DPS 曲線などのグラフを見据える
* **段階的移行が可能なこと** — 動いている約 3,500 行を全面書き換えせず、
  ページ・ウィジェット単位で置き換えたい
* **個人開発として維持可能な複雑度**（ADR 0001 / 0011 と同じ制約）
* **情報量・AI 支援** — ドキュメント・コミュニティ・AI コード支援の学習データの厚み
* バンドルサイズは重視しない — ページ重量は WASM（約 9.7 MB）が支配的で、
  フレームワークのランタイムサイズ差は実害にならない

## Considered Options

1. 現状維持（フレームワークなし・素の ES Modules を継続）
2. UI フレームワークを導入する
    - 2.1. React
    - 2.2. Vue 3
    - 2.3. Svelte 5
    - 2.4. Preact（+ Signals）

1 と 2 は「フレームワークを導入するか」の軸、2.1〜2.4 はフレームワークの軸で
分かれる。このほか Lit（Shadow DOM とスタイル分離の流儀が既存構成と噛み合わない）
と Rust 製フレームワーク（Leptos / Yew / Dioxus — UI 全面書き換えが必須で
「段階的移行」ドライバーに反し、supabase-js との連携も JS 境界を挟んで複雑化する）
は予備検討で除外した。

## Decision Outcome

選択: **2 + 2.1. React を採用する**。

決め手は可視化エコシステム。ヘッドレステーブル（TanStack Table / AG Grid）と
宣言的チャート（Recharts / visx / Nivo）の両方で最も選択肢が厚く、将来の
可視化要件に対する拡張余地が最大になる。情報量・AI 支援・TypeScript 統合も
最も枯れている。「2 ページのフォーム中心アプリに対して重量級」という減点は
あるが、可視化ドライバーを重視してこれを受け入れる。

- `@vitejs/plugin-react` を追加し、既存の vanilla JS モジュールと共存させながら
  ウィジェット単位で段階的に移行する（`createRoot` による部分マウント）。
  移行の順序・進め方は本 ADR の範囲外とする
- 新規 UI（可視化を含む）は React コンポーネントとして実装する
- React とその周辺ライブラリは npm 依存として lockfile で固定する
  （[ADR 0011](0011-vite-build-toolchain.md) の方針に従う）

### Consequences

* Good: 状態→DOM の手動同期が宣言的 UI に置き換わり、不整合バグの温床が
  解消に向かう
* Good: 可視化ライブラリの選択肢が最大になり、結果テーブル・グラフを
  自作せずに済む
* Good: 情報量・AI 支援が最大で、個人開発の実装速度に効く
* Bad: 現在の規模（2 ページ・フォームとテーブル中心）に対しては重量級。
  hooks の規約（依存配列・再レンダリング制御）という React 固有の複雑度を
  持ち込む
* Bad: 既存 HTML → JSX の移植コストが Vue / Svelte（HTML ベース）より高い
* Neutral: ランタイムサイズの増加は WASM が支配的なため実害なし

### Confirmation

* Playwright スモークテスト（`npm run test:smoke`、8 本）は
  `playwright.config.js` の webServer が `npm run build` 成果物を本番同様の
  `/ff11sim/` サブパスで配信して検証する。段階的移行中の回帰
  （マウント失敗・参照漏れ等）はここで検出される
* React 依存は `package.json` + lockfile で固定され、`npm audit` の対象になる
  （ADR 0011 と同じ仕組み）
* フレームワーク選択そのものを機械的に強制するチェックはない。新規 UI が
  React で書かれていることはレビューで確認する

検証されていないもの:

* 本 ADR 作成時点で React は未導入。導入 PR で上記スモークが通ることをもって
  確認する

## Pros and Cons of the Options

### 1. 現状維持（フレームワークなし）

* Good: 移行コストゼロ。dev サーバ・依存固定は ADR 0011 で解決済み
* Bad: 手動 DOM 同期のコードが増え続け、過去の本番障害と同型のバグの温床が残る
* Bad: 結果テーブルのソート・フィルタや、グラフと状態の同期をすべて手書きする
  ことになり、可視化ドライバーと最も相性が悪い

### 2. UI フレームワークを導入する（採用）

* Good: 状態→DOM 同期が宣言化され、UI をコンポーネント単位でテストできる
* Good: ヘッドレステーブル・チャートラッパー等のエコシステムに乗れる
* Bad: 依存とビルド設定が増える。JSX / SFC など固有記法への移植コストが発生する

#### 2.1. React（採用）

* Good: 可視化エコシステムが最強（TanStack Table / AG Grid / Recharts /
  visx / Nivo / react-window）
* Good: 情報量・AI 支援・TypeScript 統合が最も枯れており、長期安定性が高い
* Good: `createRoot` の部分マウントで既存 DOM と共存でき、段階的移行が可能
* Bad: 記述量と規約の複雑度は 4 案中最大。現在の規模に対して重量級
* Bad: 既存 HTML → JSX の変換が必要

#### 2.2. Vue 3

* Good: テンプレートが HTML に近く、既存マークアップからの移植が最も素直。
  日本語ドキュメント・コミュニティも充実
* Good: TanStack Table / AG Grid の公式対応と vue-echarts があり、可視化にも
  穴がない
* Bad: Options API / Composition API など流儀の選択肢が多く、一人開発では
  「流儀を決める」コストがある
* Bad: 宣言的チャートの選択肢は React に一段劣る

#### 2.3. Svelte 5

* Good: 記述量が最少で HTML ベース。「維持可能な複雑度」には最も適合
* Good: TanStack Table v9（2026-08 安定版）で Svelte 5 が公式サポートされ、
  ヘッドレステーブルの懸念は解消済み
* Bad: 宣言的チャートの選択肢が最も薄く、凝った可視化では自作部分が増える
* Bad: エコシステムが小さく、Svelte 4→5 で API が大きく変わった経緯もあり
  メジャーバージョン間の移行リスクが相対的に高い

#### 2.4. Preact（+ Signals）

* Good: 約 4KB で React API 互換。React の知見を流用できる
* Bad: 可視化を React エコシステムに依存すると `preact/compat` を常時経由する
  ことになり、Recharts 等の複雑なライブラリで compat 起因の問題を踏むリスクが
  ある。可視化重視なら本家 React を選ぶ方が素直

## More Information

* 前提となる構成: [ADR 0001](0001-rust-wasm-static-site.md)（Rust + WASM 静的
  サイト）、[ADR 0011](0011-vite-build-toolchain.md)（本 ADR は同 ADR が
  保留した「UI フレームワークの採用は別 ADR」を実行するもの）
* 手動 DOM 同期による本番障害の記録:
  [docs/tech-debt/inline-script-monolith.md](../tech-debt/inline-script-monolith.md)
* TanStack Table v9 の Svelte 5 対応（案 2.3 の評価根拠）:
  <https://tanstack.com/blog/announcing-tanstack-table-v9>
* 移行順序の計画と TypeScript 化との順序関係:
  [docs/roadmap/react-migration.md](../roadmap/react-migration.md)
