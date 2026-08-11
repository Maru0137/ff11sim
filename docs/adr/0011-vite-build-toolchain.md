---
status: accepted
date: 2026-08-11
decision-makers: Akira Maruoka
---

# 0011. フロントエンドのビルドツールチェーンとして Vite を採用する

## Context and Problem Statement

Web フロントエンドは素の HTML + ES Modules で書かれ、ビルドステップを持たない。
`web/` ディレクトリがそのまま GitHub Pages にアップロードされる
（[ADR 0001](0001-rust-wasm-static-site.md)）。ADR 0001 はこの「フレームワークを
使わない素の HTML + ES Modules」が同 ADR の決定ではないことを明記しており、
ツールチェーンの選定は未決定のままだった。

現状の構成には具体的な弱点がある。唯一の外部ランタイム依存である supabase-js が
`web/js/supabase-client.js` で esm.sh CDN からバージョン未固定のまま import されて
おり、ロックファイルによる固定も脆弱性監査も効かない。また、ローカル開発は静的
サーバ頼みで、変更のたびに手動リロードが必要になる。インライン script の
モジュール化を機に、ビルドツールチェーンを導入するか、するなら何を使うかを
決める必要がある。

なお、UI フレームワーク（React / Vue / Svelte 等）の採用は別軸の決定であり、
本 ADR の範囲外とする（検討する時点で別途 ADR を作成する）。

## Decision Drivers

* **外部依存の固定と監査** — supabase-js を lockfile で固定し、CDN への
  ランタイム依存（可用性・サプライチェーン）を断ちたい
* **個人開発として維持可能なビルド複雑度に収めること** — ADR 0001 と同じ制約。
  ビルドは既に Rust + wasm-pack + Python の多段であり、これ以上の複雑化は
  最小限にしたい
* **開発体験** — dev サーバ（自動リロード）と、将来の TypeScript 化・
  ユニットテスト（Vitest）導入の土台
* **GitHub Pages 静的配信との整合** — 成果物は静的ファイルのままであること
* 転送量削減は主目的ではない — ページ重量は WASM（約 9.7 MB）が支配的で、
  JS のバンドルによる削減効果は小さい

## Considered Options

1. 現状維持（ビルドなし・素の ES Modules を静的配信）
2. バンドラを導入する
    - 2.1. Vite
    - 2.2. webpack
    - 2.3. esbuild / Rollup を単体で使う

1 と 2 は「ビルドステップを持つか」の軸、2.1〜2.3 はツールの軸で分かれる。

## Decision Outcome

選択: **2 + 2.1. Vite を採用する**。

ビルドなし構成の単純さは実利だが、supabase-js の CDN 依存を解消する手段がなく、
dev サーバ・TypeScript・ユニットテストといった将来の改善がすべて塞がれたままに
なる。バンドラの中では、dev サーバ・マルチページ（index.html / search.html の
2 エントリ）・ビルドを 1 ツール・最小設定で賄える Vite が、「維持可能な
ビルド複雑度」の制約に最も適合する。

- 依存は npm（package.json + lockfile）で管理し、supabase-js の esm.sh import は
  npm 依存に置き換える
- 開発・ビルド・プレビューは Vite（`vite` / `vite build` / `vite preview`）に
  統一し、GitHub Pages へはビルド成果物をアップロードする
- エントリは index.html / search.html の 2 HTML（`rollupOptions.input`）
- インライン script のモジュール化（ES Modules 分割）はバンドラ非依存の作業で
  あり、本決定に先行して現行構成のまま実施できる

### Consequences

* Good: supabase-js がロックファイルで固定され、実行時の CDN 依存が消える。
  `npm audit` の対象にもなる
* Good: dev サーバ（HMR）が使えるようになり、将来の TypeScript / Vitest 導入の
  土台が揃う
* Bad: 「`web/` をそのまま配信する」単純さを失う。CI 2 本
  （test.yml / deploy.yml）へのビルドステップ追加、Playwright の webServer
  変更、web/README の開発手順（静的サーバ 4 通り）の書き換えが必要
* Bad: wasm-pack 出力（web/pkg）・データ JSON のシンボリックリンク
  （[ADR 0002](0002-shared-table-data-json.md)）・config.js の CI 注入
  （[ADR 0007](0007-supabase-anon-key-ci-injection.md)）を Vite の構成に
  適合させる作業が発生する
* Neutral: 転送量はほぼ変わらない（WASM が支配的）。バンドルは目的ではなく
  副産物

### Confirmation

本 ADR の時点で Vite は未導入である（インライン script のモジュール化を先行して
実施中）。導入後の検証手段は次を想定する:

* CI（test.yml / deploy.yml）が `vite build` を実行し、ビルド不能な状態では
  デプロイに進めないこと
* Playwright スモークテストがビルド成果物（`vite preview`）に対して走ること

導入時に本セクションを実態に合わせて更新すること。

## Pros and Cons of the Options

### 1. 現状維持(ビルドなし・素の ES Modules を静的配信)

* Good: 最も単純。どの静的サーバでも動き、デプロイは `web/` のアップロードのみ
* Good: ビルド起因の不具合・設定メンテナンスが存在しない
* Bad: supabase-js の CDN 依存（バージョン未固定・実行時の外部到達性・
  サプライチェーン）を解消できない。これが最大の却下理由
* Bad: dev サーバ・TypeScript・ユニットテストへの道が塞がったまま

### 2. バンドラを導入する(採用)

* Good: 依存を npm で固定・監査できる
* Good: 開発体験と将来の拡張（TS / テスト）の土台が得られる
* Bad: ビルドステップが増え、CI・デプロイ・開発手順の改修が必要

#### 2.1. Vite(採用)

* Good: dev サーバ・マルチページ・ビルドが 1 ツールで揃い、設定が最小で済む
* Good: Vitest とシームレスに統合でき、将来のユニットテスト導入が自然
* Good: WASM（`--target web` の wasm-pack 出力）や top-level await を
  ターゲット設定（esnext）で扱える
* Bad: Rollup ベースの規約に乗るため、config.js 注入やシンボリックリンク等の
  既存の変則的な構成は public/asset 方式への適合が要る

#### 2.2. webpack

* Good: 実績とプラグインの蓄積が最も厚い
* Bad: 設定量が多く、dev サーバも含め構成の維持コストが Vite より高い。
  個人開発の「維持可能な複雑度」に反する

#### 2.3. esbuild / Rollup を単体で使う

* Good: ツールとしては最小・最速で、依存も少ない
* Bad: dev サーバ・マルチページ対応・アセット処理を自前で組み合わせる必要が
  あり、結局 Vite が同梱しているものを手作りすることになる

## More Information

* 前提となる静的サイト構成: [ADR 0001](0001-rust-wasm-static-site.md)
  （本 ADR は同 ADR の Neutral 項「別途 ADR 化する余地がある」を実行するもの）
* Vite 構成に適合させる対象: [ADR 0002](0002-shared-table-data-json.md)
  （データ JSON のシンボリックリンク）、
  [ADR 0007](0007-supabase-anon-key-ci-injection.md)（config.js の CI 注入）
* 先行するモジュール化の背景: [docs/tech-debt/inline-script-monolith.md](../tech-debt/inline-script-monolith.md)
* UI フレームワークの採用は別決定。検討する時点で別 ADR を作成する
