---
status: accepted
date: 2026-08-02
decision-makers: Akira Maruoka
---

# 0001. 計算コアを Rust の library crate として実装し、WASM とネイティブの両方に配る

## Context and Problem Statement

ff11sim の中核は FFXI のステータス・ダメージ計算である。この式は丸めの位置と順序に敏感な構造を持つ。これを少ない実装コストで厳密に再現するには、整数と実数を型で区別できる言語が要る。

同時に、この計算機能は 2 つの経路から使いたい。1 つは Web ブラウザ（誰でも URL を開くだけで
使える）、もう 1 つは CLI やスクリプト（装備の組み合わせを一括で回すシミュレーション）である。
さらに、個人開発として運用を抱えたくないため、可能な限りサーバーレスに寄せたい。
つまり計算はサーバーではなくクライアント側で完結させることが前提になる。
実装言語と、この 3 つの要求を同時に満たす実行形態をどう成立させるかを決める必要がある。

## Decision Drivers

* **整数と実数を型で区別し、切り捨ての位置を式のとおりに書けること** —
  少ない実装コストで厳密に再現するための前提
* **Web ブラウザから利用できること** — インストール不要で誰でも使えること
* **同じコア実装を CLI やスクリプトからも実行できること** — 一括シミュレーションに使えること
* **可能な限りサーバーレスであること** — 計算をクライアント側で完結させ、
  運用コストと保守負荷をゼロに保つこと
* 計算ロジックの正しさを自動テストで固定できること（回帰を検出できること）
* ゲームの概念（ジョブ・種族・スキル・グレード）を型で表現し、網羅漏れを検出できること
* 個人開発として維持可能なビルド複雑度に収めること

## Considered Options

1. すべて JavaScript / TypeScript で実装する
2. 計算コアを Rust の library crate として実装し、複数の実行形態に配る
    - 2.1. WASM のみを提供する
    - 2.2. ネイティブ CLI のみを提供する
    - 2.3. WASM とネイティブの両方を提供する
3. 計算コアを Rust で実装し、サーバー (Web API) として動かす

1 と 2 / 3 は言語の軸、2.1〜2.3 は配布形態の軸で分かれる。

## Decision Outcome

選択: **2 + 2.3. Rust の library crate を WASM とネイティブの両方に配る**（採用）。

JavaScript の数値は f64 のみで、整数と実数の区別が言語に存在しない。丸めの位置を守る責任が
すべて実装者の規約になり、「ここは中間値なのでまだ切り捨ててはいけない」という制約を
コードに書き残せない。Rust なら `i32` と `f32` の使い分けが型で強制される。実際
`calc_status` は `f32` を返し、`chara.rs` で種族・メインジョブ・サポートジョブの 3 項を
合算してから **一度だけ** `floor` する。この「合算前に丸めない」という仕様が型に現れている。

サーバー案 (3) はサーバーレス志向に正面から反するため早い段階で外れる。計算をクライアント側で
完結させると決めた以上、ブラウザで動く形式、すなわち WASM が必要になる。そのうえで
ブラウザ利用と CLI 利用の両方を満たすのは 2.3 だけである。しかも Rust では
`crate-type = ["cdylib", "rlib"]` の 1 行で両立でき、追加コストがほぼない。

- 計算とゲーム仕様のモデリングは `rust/src/` の library crate に置く。
  **このコアは `wasm-bindgen` に依存してはならない。**
- `rust/Cargo.toml` の `crate-type = ["cdylib", "rlib"]` により、WASM (cdylib) と
  ネイティブ (rlib) を同じソースから生成する。
- JavaScript 向けの API は `rust/src/wasm.rs` にアダプタとして集約する。ここは型変換と
  serde シリアライズに徹し、計算ロジックを持たない。
- ネイティブ側の入口は `rust/src/main.rs`（CLI）と `rust/examples/`（使い捨ての
  シミュレーションスクリプト）。
- WASM は `wasm-pack build --target web --out-dir ../web/pkg` で生成する。
  生成物 `web/pkg/` は `.gitignore` 済みで、リポジトリには含めない。
- ホスティングは GitHub Pages。`.github/workflows/deploy.yml` が `main` への push を
  トリガーにビルドから配信までを行う。
- サーバーが存在しないため、秘匿したいロジックや重い一括集計はこの構成には置けない。
  それらが必要になった時点でこの ADR を見直す。

### Consequences

* Good: `i32` と `f32` の使い分けが型で強制され、丸めの位置を式のとおりに書ける。
  `status.rs` の `calc_status` が `f32` を返し、`chara.rs:72` が
  `(status_race + status_main_job + status_support_job).floor() as i32` と
  一度だけ丸める構造が、型として読み取れる。
* Good: 計算ロジックを `cargo test` で固定できる。ジョブ特性・ギフト・攻撃/命中・
  WS ダメージ・装備解釈・装備検索などの期待値を検証している。
* Good: 同じコアを WASM・CLI・`cargo test`・`examples` の 4 経路から利用できる。
  ブラウザを起動せずに計算を検証できる。
* Good: `enum` と `EnumMap` でジョブ・種族・スキルを表現でき、網羅漏れが型で落ちる
  （[ADR 0002](0002-shared-table-data-json.md) 参照）。
* Good: ホスティング費用と運用がゼロ。障害対応の対象がフロントエンドだけになる。
* Bad: ビルドが Rust + wasm-pack + Python + GitHub Actions の多段になり、
  ローカル開発でも `wasm-pack` のインストールが必須。
* Bad: 初回アクセス時に WASM バイナリのダウンロードが発生する。
* Bad: **ネイティブ側の入口は現状ほぼ空である。** `main.rs` は `Hello, world!` のままで
  CLI は未実装、`clap` は依存に入っているが `race.rs` の `ValueEnum` derive にしか
  使われていない。3 つ目のドライバは「構造として可能」という段階にとどまっている。
* Bad: サーバー側処理が一切ないため、データの集約・ランキング・重い探索は実装できない。
* Neutral: アプリケーション状態はすべてクライアントに存在する。永続化は別の問題として
  [ADR 0005](0005-localstorage-default-supabase-optin.md) で扱う。
* Neutral: UI 側はフレームワークを使わない素の HTML + ES Modules で書かれているが、
  これは本 ADR の決定ではない（別途 ADR 化する余地がある）。

### Confirmation

* `.github/workflows/test.yml` が `cargo test --lib --tests` を実行する。
  `pull_request` で単独に走るほか、`deploy.yml` が `workflow_call` で呼び
  `build` ジョブが `needs: test` で受けるため、失敗すれば `wasm-pack build` にも
  deploy にも進まない。計算ロジックの回帰はマージ前と配信前の両方で止まる。
* `rust/Cargo.toml` の `crate-type = ["cdylib", "rlib"]` が両形態のビルドを担保する。
  `cargo test` はネイティブターゲットでビルドされるため、コアが WASM 専用 API に
  依存し始めればここで検出される。実際 `#[cfg(target_arch = "wasm32")]` を使っているのは
  `wasm.rs` の `init()` のみで、他のモジュールはネイティブでもそのままコンパイルされる。

検証されていないもの:

* WASM 境界（`rust/src/wasm.rs`）を JS 側から呼ぶ結合テストは CI で自動化されていない。
  移植時（[ADR 0010](0010-equipment-interpretation-in-rust.md)）に `--target nodejs`
  でビルドして JS から呼び、全 15,504 件を JS 実装と突き合わせて一致を確認したが、
  これは手動実行であり CI には入っていない。境界より内側は `cargo test` が押さえている。
  なお `web/test/*.test.js` は Rust へ移植して削除済みで、移植先は `cargo test` で走る。
* CLI 経路が実際に機能するかは未確認。`main.rs` に中身がなく、
  `rust/examples/status_calculator.rs` は存在しない `ff11sim::prelude` を import しているため、
  `examples` を含むフルビルド（`cargo test`）はローカルで失敗する。
  現時点でこのファイルは git 未追跡のため CI には影響していない。
* WASM のビルドサイズやロード時間を監視する仕組みはない。

フォローアップ候補: `main.rs` に CLI を実装し、`prelude` を用意して `examples` を
ビルド可能にする。3 つ目のドライバを実体化させ、CI で `examples` を含むフルビルドを走らせる。

## Pros and Cons of the Options

### 1. すべて JavaScript / TypeScript で実装する

* Good: ビルドが単純。WASM のロードも追加ツールチェーンも要らない。
* Good: UI と同じ言語のため、デバッグの経路が 1 本で済む。
* Bad: 数値型が f64 のみ。整数の確定値と 0.5 刻みの中間値を型で区別できず、
  「まだ丸めてはいけない」という制約を規約でしか守れない。これが最大の却下理由。
* Bad: enum の網羅性がコンパイル時に検査されず、ジョブやスキルの追加漏れがテスト頼りになる。
* Bad: コアを CLI から回すには Node 実行環境を別途整えることになり、
  かつ上記の型の利点は得られない。

### 2. 計算コアを Rust の library crate として実装する（採用）

* Good: `i32` / `f32` の区別、`enum` の網羅性検査、標準のテスト機構が揃う。
* Good: コアを実行形態から独立させられる。
* Bad: ツールチェーンが増え、UI とは別言語になる。境界での型変換コストが発生する。

#### 2.1. WASM のみを提供する

* Good: 成果物が 1 つで最も単純。
* Bad: コアを CLI やスクリプトから回せず、3 つ目のドライバを満たさない。

#### 2.2. ネイティブ CLI のみを提供する

* Good: ビルドが最も単純で、配布物も 1 つ。
* Bad: ブラウザから使えず、2 つ目のドライバを満たさない。利用にインストールが要る。

#### 2.3. WASM とネイティブの両方を提供する（採用）

* Good: `crate-type = ["cdylib", "rlib"]` の 1 行で両立し、追加コストがほぼない。
* Good: コアがどちらの実行形態にも依存しないことが、両方をビルドし続けることで
  構造的に強制される。
* Bad: コアを `wasm-bindgen` に依存させない規律が要る。境界を破ると片方のビルドが壊れる。
* Bad: 2 つの入口を維持する必要があり、実際いま CLI 側が手つかずのまま放置されている。

### 3. 計算コアを Rust で実装し、サーバーとして動かす

* Good: 重い一括計算や秘匿したいロジックを置ける。クライアントを薄くできる。
* Bad: サーバーの運用コストと保守負荷が発生し、サーバーレス志向に正面から反する。
* Bad: 計算がクライアント側で完結せず、オフラインや通信不良で使えなくなる。
* Bad: CLI から使う場合も HTTP を経由することになり遠回りになる。

## More Information

* データテーブルの共有方法: [ADR 0002](0002-shared-table-data-json.md)
* 装備データの供給: [ADR 0003](0003-items-json-generated-in-ci.md)
* 永続化: [ADR 0005](0005-localstorage-default-supabase-optin.md)
* 計算式の仕様ドキュメント: [docs/knowledge/status/](../knowledge/status/)
