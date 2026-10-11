# ff11sim

エージェント向けの指示。人間向けの開発手順は [CONTRIBUTING.md](CONTRIBUTING.md) にあり、ここでは繰り返さない。

## プロジェクトの概要

FINAL FANTASY XI のキャラクター・装備・ダメージのシミュレータ。計算のコアは Rust で、WASM にして
ブラウザ UI（React / TypeScript）から呼ぶ。ゲームの実際の計算式でステータスを求める。

1. 装備データベースを検索する
2. 装備セットを作って登録する
3. 種族・ジョブ・レベル・装備セットからキャラクターのステータスを計算する
4. 3 のキャラクターでダメージをシミュレートする

## 置き場の規約

| 文書 | 置き場 | 作るとき |
|---|---|---|
| ADR（MADR 形式） | `docs/internal/adr/` | 戻しにくい設計判断をしたとき |
| 仕様書 | `docs/internal/specs/` | 最初の仕様書を書くとき |
| 用語集 | ルートの `CONTEXT.md` | 最初の用語が決まったとき |
| ドメイン知識（ゲーム仕様・計算式・検証結果） | `docs/internal/knowledge/` | 知見を得たとき |
| 移行などの計画 | `docs/internal/roadmap/` | 複数の変更にまたがる計画を立てたとき |
| 既知の負債・意図的に残した非互換 | `docs/internal/tech-debt/` | 負債を見つけた・残すと決めたとき |
| 外部に公開するドキュメントのソース | `docs/public/` | — |

## 文書の運用

- **ドメイン知識はリポジトリに記録する**: ゲーム仕様・計算式・検証結果などのナレッジは
  `docs/internal/knowledge/` の適切なパスに Markdown で記録する
  (例: `docs/internal/knowledge/status/magic_accuracy.md`)。エージェントのセッションメモリには
  リポジトリに書けない情報のみを置き、リポジトリが記録すべき内容をメモリに残さない。
- **設計判断は ADR に記録・追従する**: 設計上の決定は `docs/internal/adr/` に MADR 形式で記録する。
  設計に影響する変更を行うときは既存 ADR を確認し、実装との食い違いが生じる場合は
  同じ変更の中で ADR を改訂 (または新規作成) して追従させる。

## ビルドとテストの注意点

手順は [CONTRIBUTING.md](CONTRIBUTING.md) の「ビルド手順」「テスト手順」にある。エージェントが踏みやすい点だけ挙げる。

- Rust をビルドする前に `scripts/build_data.sh` で `build/items.json` を生成する。装備データは
  `include_str!` でバイナリに埋め込むため、無いとコンパイルが通らない。
- 全テストは `cargo test --release --manifest-path rust/Cargo.toml` で回す。`rust/tests/ja_en_conformance.rs` は
  debug だと約 150 秒、release だと約 8 秒かかる。
- Rust を変えたら、ブラウザで確かめる前に WASM を作り直す（`wasm-pack build rust --target web --out-dir ../web/pkg`）。

## アーキテクチャ

### Rust（`rust/src/`）

| モジュール | 役割 |
|---|---|
| `race.rs` / `job.rs` | 種族（ステータスのグレード表）、ジョブとジョブ特性 |
| `status.rs` | ステータス計算の基礎（グレード、メリットポイント、マスターレベル補正） |
| `skills.rs` | スキル種別とスキルキャップ |
| `job_points.rs` / `gift.rs` | ジョブポイントと、その累計で得るギフト |
| `chara.rs` | 種族・ジョブ・レベル・装備からステータスを求めるキャラクター |
| `character_profile.rs` | 名前・全ジョブのレベル・メリット・ジョブポイントなど、保存するプロファイル |
| `breakdown.rs` | ステータス / プロパティ値のソース別内訳（ADR 0016） |
| `items.rs` / `data_loader.rs` | 装備データとテーブルデータ（`data/*.json`）の埋め込みと読み込み |
| `item_search.rs` | 装備検索 |
| `equip.rs` / `equip_stats.rs` | 装備と装備セット、装備の説明文からの数値抽出（ADR 0018 / 0019） |
| `wasm.rs` | JS から呼ぶ WASM の境界 |

### Web（`web/`）

React / TypeScript の SPA（`web/src/`）を Vite でビルドし、GitHub Pages で配信する。詳細は
[web/README.md](web/README.md)。
