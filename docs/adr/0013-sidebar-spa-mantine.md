---
status: accepted
date: 2026-08-12
decision-makers: Akira Maruoka
---

# 0013. 左サイドバーナビゲーションへ再構成し、装備検索を SPA に統合、Mantine を段階導入する

## Context and Problem Statement

Web UI は index.html（水平タブ 2 つ: キャラクター管理 / 装備セット）と
search.html（装備検索）の 2 エントリ構成
（[ADR 0011](0011-vite-build-toolchain.md)）だが、装備検索へはトップページから
辿る導線が無く、事実上の孤立ページになっていた。機能を 3 つ（キャラクター /
装備セット / 装備検索）並べる左サイドバーへ UI を再構成するにあたり、
(a) 装備検索ページをどう統合するか、(b) サイドバーのレイアウト
（折りたたみ・モバイル対応を含む）を何で実装するかを決める必要がある。

## Decision Drivers

* **ナビゲーションの一元化** — 3 機能を同格に並べ、機能間の行き来を 1 クリックにする
* **既存挙動の維持** — ビュー切替で検索条件や編集中の状態を失わない
  （現行はタブの display:none 切替で state が保持される）。共有閲覧モード
  （?share=）で編集 UI とナビが出ないこと
* **静的配信の制約** — GitHub Pages 配信のため History API ルーティングは
  404 になる（[ADR 0001](0001-rust-wasm-static-site.md)）
* **旧 URL 互換** — search.html へのブックマークを壊さない
* **段階的移行が可能なこと・維持可能な複雑度**（[ADR 0012](0012-react-ui-framework.md)
  と同じ制約）— 動いている既存 CSS (約 1,000 行) を全面書き換えしない
* モバイル・折りたたみ（アイコンレール）対応を自作でどこまで持つか

## Considered Options

1. マルチページ構成を維持する（サイドバーはページ間リンク）
2. 装備検索を index.html の SPA に統合する
    - 2.1. ルーティング: location.hash の手動同期
    - 2.2. ルーティング: React Router（HashRouter）
3. レイアウトを素の CSS で自作する
4. レイアウトに Mantine を段階導入する（layout-only から開始）
5. レイアウトに Tailwind CSS + shadcn/ui を導入する

1〜2 は装備検索の統合方式の軸、3〜5 はサイドバーレイアウトの実装手段の軸で
直交する。

## Decision Outcome

選択: **2 + 2.1 + 4. SPA 統合（hash 手動同期）+ Mantine 段階導入**。

統合方式は、ビュー間で状態を保持したまま 1 クリック遷移できる SPA 統合が
ドライバーに最も適合する。ルーティングは 3 ビューに対してライブラリは過剰で、
React Router はビューを unmount する志向のため「display:none 切替で state
保持」と相性が悪く、hash の手動同期（`web/src/routing.ts`、約 40 行）とした。
レイアウトは、折りたたみ幅切替・ブレークポイント・モバイルのオーバーレイを
自作せず AppShell に委譲でき、既存 CSS と共存させながらコンポーネントを
後から段階的に置き換えられる Mantine を採用した。

- ルートは `#/characters` / `#/equipsets` / `#/search`。不正・空 hash は
  characters（共有閲覧モードは equipsets 固定）にフォールバックする
- 検索ビューは初回表示まで遅延マウントする（SearchPage は mount 時に
  全件検索を走らせるため）
- search.html は `location.replace('./#/search')` のリダイレクトシムとして
  残し、Vite の 2 エントリ構成（ADR 0011）は維持する
- Mantine の初期スコープはレイアウトのみ（AppShell / NavLink / Burger /
  Overlay / Tooltip）。既存のパネル・フォーム・モーダル CSS は温存する
- CSS は `@mantine/core/styles.css` → `styles/index.css` の順に main.tsx で
  import し、既存スタイルを後勝ちにする。テーマ（`web/src/theme.ts`）が既存
  配色（#f0c040 / #1a1a2e）を Mantine の CSS 変数へ反映する
- PostCSS preset（postcss-preset-mantine）は導入しない。prebuilt の
  styles.css を使う現スコープでは不要で、自前 CSS で Mantine の mixin を
  使い始める段階で再検討する
- 共有閲覧モード（?share=）はサイドバー（AppShell）自体をレンダリングしない
  分岐にする。CSS で隠す方式はセレクタの張り替え忘れでナビが露出する事故が
  起こり得るため、構造的に排除する
- 装備検索の DOM/CSS は `.search-page` スコープに閉じ、index.css との
  セレクタ衝突（裸の `select`、`.btn` 系、`.container`）を構造的に回避する

### Consequences

* Good: 3 機能がサイドバーで一元化され、hash による deep link
  （`#/search` 等）とブックマーク互換（search.html シム）が両立する
* Good: 折りたたみ（56px アイコンレール）とモバイル（ハンバーガー +
  オーバーレイ）を AppShell の機構で実現でき、自作レスポンシブ層を持たない
* Good: 今後 Mantine コンポーネント（モーダル・セレクト等）への段階移行の
  土台ができる
* Bad: Mantine という比較的大きな依存が増える（ただしページ重量は WASM が
  支配的 — ADR 0012 と同じ評価）
* Bad: 移行完了まで Mantine のスタイルシステムと手書き CSS が混在し、
  カスケード順（import 順）に暗黙の依存を持つ
* Bad: index.css の裸の `select` / `input` グローバルルールが残っており、
  Mantine の input 系コンポーネントを使い始める際にクラス限定化が必要
  （既知の残課題）
* Neutral: AppShell の 56px アイコンレールは Mantine 標準機能ではなく
  `navbar.width` の切替で実現している（`collapsed.desktop` は幅 0 の
  完全非表示専用）

### Confirmation

* Playwright スモークテスト（`tests/smoke.spec.js`、11 本）が検証する:
  - search.html → `#/search` へのリダイレクト（URL アサート付きで検索実行・
    ページネーションまで確認）
  - サイドバー経由のビュー切替（`[data-nav="equipsets"]` クリックで装備
    セット操作が通ること）
  - deep link（`./#/search` 直アクセスで検索フォーム表示 → ナビクリックで
    hash 遷移）
  - モバイル幅 375px でハンバーガー開閉とビュー切替
  - 共有 URL（?share=）でサイドバーのナビが出ないこと
    （`[data-nav]` の toHaveCount(0)）
* hash 解決の純関数は `web/src/routing.test.ts`（Vitest）が検証する
* Mantine 依存は package.json + lockfile で固定され `npm audit` の対象
  （[ADR 0011](0011-vite-build-toolchain.md) と同じ仕組み）
* 「新規レイアウト UI に Mantine を使う」こと自体を強制する機械的チェックは
  ない。レビューで確認する

## Pros and Cons of the Options

### 1. マルチページ構成を維持する

* Good: 実装コスト最小。既存の 2 エントリをそのまま使える
* Bad: サイドバーのクリックがページ遷移になり、ビュー間の状態
  （検索条件・編集中の内容）が毎回失われる
* Bad: サイドバー・認証 UI・テーマを両ページに二重実装することになる

### 2. 装備検索を SPA に統合する（採用）

* Good: ビュー切替が即時で、display:none 切替により state が保持される
* Good: レイアウト・認証 UI が 1 箇所になる
* Bad: 検索ビューの CSS を index.css と衝突しないよう隔離する作業が必要
  （`.search-page` スコープ化で対応）

#### 2.1. location.hash の手動同期（採用）

* Good: 依存ゼロ・約 40 行。静的配信でも deep link が機能する
* Good: ビューのマウント管理を自前で握れるため、「アンマウントしない」
  既存挙動をそのまま維持できる
* Bad: ネスト・パラメータ付きルート等が必要になったら作り直しになる
  （現時点の 3 ビューでは不要 — YAGNI）

#### 2.2. React Router（HashRouter）

* Good: ルート定義が宣言的で、将来のルート追加に強い
* Bad: ルート切替でコンポーネントを unmount する志向のため、state 保持には
  別途の仕掛け（state externalization 等）が必要になり、かえって複雑化する
* Bad: 3 ビューに対して依存・概念数が過剰

### 3. レイアウトを素の CSS で自作する

* Good: 依存が増えず、既存 CSS と完全に地続き
* Bad: 折りたたみアニメーション・ブレークポイント・モバイルオーバーレイ・
  main のオフセット管理をすべて手書きし、以後も自前で保守することになる
* Bad: 今後のコンポーネント刷新（モーダル・セレクト等）に土台を提供しない

### 4. Mantine を段階導入する（採用）

* Good: AppShell がサイドバーレイアウト（幅切替・breakpoint・モバイル時の
  オーバーレイ化）を宣言的に提供する
* Good: React ファースト・TypeScript ネイティブで、テーマ（CSS 変数）に
  既存配色を注入すれば手書き CSS と共存できる。コンポーネント単位の段階
  移行が可能（ADR 0012 の「段階的移行」ドライバーと整合）
* Bad: 依存サイズと、Mantine 流のスタイル API という新しい概念が増える
* Bad: グローバル styles.css が既存要素に影響し得るため、import 順の管理と
  導入時の目視確認が必要だった

### 5. Tailwind CSS + shadcn/ui

* Good: ユーティリティ CSS で自由度が最も高く、shadcn/ui の Sidebar
  コンポーネントも存在する
* Bad: 既存の約 1,000 行のセマンティック CSS と設計思想が根本的に異なり、
  「既存 CSS を温存して段階移行」という方針と噛み合わない
* Bad: PostCSS / 設定ファイル・class 命名規約の学習と、コンポーネントを
  コピーして保守する shadcn/ui のモデルは一人開発の保守コストとして過剰

## More Information

* 前提となる構成: [ADR 0001](0001-rust-wasm-static-site.md)（静的配信 =
  History API 不可の根拠）、[ADR 0011](0011-vite-build-toolchain.md)
  （2 エントリ構成 — 本 ADR で search.html は実体を持たないリダイレクト
  シムになったが、エントリとしては維持）、
  [ADR 0012](0012-react-ui-framework.md)（React 採用。本 ADR はその上の
  レイアウト/コンポーネント層の決定）
* 残課題（コンポーネント段階移行時に対応）: index.css の裸の `select` /
  `input` ルールのクラス限定化、postcss-preset-mantine の導入判断
