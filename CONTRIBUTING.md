# CONTRIBUTING

## 動作手順

### 0. prerequisite

| ツール | 用途 | 備考 |
|---|---|---|
| Rust | コア実装のビルド・テスト | [rustup](https://rustup.rs/) で導入。版は `rust-toolchain.toml` で固定 |
| wasm-pack@0.14.0 | ブラウザ向け WASM のビルド | 次のコマンドで導入 |
| Python 3 | 装備データ生成スクリプト | 標準ライブラリのみ使用 |
| Node.js + npm | JS の lint・スモークテスト・ローカル配信 | `package.json` / `package-lock.json` で版を固定 |
| [uv](https://docs.astral.sh/uv/) | 開発ツール (pre-commit) の導入 | `pyproject.toml` / `uv.lock` で版を固定 |

wasm-pack は Cargo の依存としては管理できません（`[dependencies]` はライブラリ用で、
依存クレートの実行ファイルはビルドされないため）。版を明示して導入してください。
CI も同じ版を使っています。上げるときは両方を揃えて変更します。

```bash
cargo install --locked wasm-pack@0.14.0
```

### 1. 初回セットアップ

```bash
git clone https://github.com/Maru0137/ff11sim.git && cd ff11sim
```

開発ツールを取得します（`.venv/` が作られます）。

```bash
uv sync
```

JS の開発ツールを取得します（`node_modules/` が作られます）。

```bash
npm ci
```

pre-commit を有効化します。

```bash
uv run pre-commit install
```

**git は clone 時にフックを自動で有効化しません**（任意のコードが実行されてしまうため）。
そのためこの 1 行は各自で実行する必要があります。

### 2. ビルド手順

装備データを生成します。

```bash
scripts/build_data.sh
```

`build/items.json` は上流の [Windower/Resources](https://github.com/Windower/Resources) から
生成する派生物で、git では管理していません（[ADR 0003](docs/adr/0003-items-json-generated-in-ci.md)）。
**Rust のビルドより先に実行する必要があります。** 装備データは `include_str!` でバイナリに
埋め込むため（[ADR 0009](docs/adr/0009-embed-item-data-in-binary.md)）、これが無いとコンパイルが通りません。

生成物は 3 つです。

| 出力 | 内容 |
|---|---|
| `build/items.json` | 装備データ本体（約 8.6MB） |
| `build/_build_metadata.json` | 何から作られたかの記録（上流の blob SHA・commit・時刻） |
| `temp_resources/*.lua` | 上流からのダウンロードキャッシュ |

上流の内容と変換スクリプトが前回と同じなら、ダウンロードと `items.json` の生成を
どちらもスキップします。作り直したいときは `--force` を付けてください。

WASM をビルドします。Rust を変更したら、そのつど実行が必要です。

```bash
wasm-pack build rust --target web --out-dir ../web/pkg
```

`--out-dir` はクレート (`rust/`) からの相対パスです。生成先は `web/pkg/` になります。

### 3. テスト手順

計算ロジックのテスト（Rust）。`build/items.json` の生成が前提になります。

```bash
cargo test --manifest-path rust/Cargo.toml
```

JS の静的検査。未定義識別子（削除したモジュールの参照残りなど）を検出します。
`index.html` 内のインライン script も対象です。

```bash
npm run lint
```

スモークテスト。ページを実際に開き「コンソールエラーが無い」「ステータスが 0 でない」を
確認します。Rust のテストが通っていても JS の参照漏れや WASM の初期化失敗で
ページが動かないことがあり、その層はここでしか検出できません。
WASM のビルド（手順 2）が前提になります。初回のみブラウザの取得が必要です。

```bash
npx playwright install chromium
```

```bash
npm run test:smoke
```

### 4. ローカルでの Web ページ確認

`web/` を静的配信します。

```bash
npm run serve
```

http://localhost:8000 を開きます。ポートは 8000 を推奨します
（Supabase のリダイレクト許可リストに登録済みで、ログイン機能を試せるため）。

ログイン機能を試す場合のみ、追加で `web/js/config.js` の用意が必要です。
詳細は [web/README.md](web/README.md) を参照してください。

## ディレクトリ構成

```
.
├── rust/                  コア実装
│   ├── src/               ステータス計算・ダメージ計算・装備解釈・WASM バインディング
│   └── tests/             統合テスト
├── web/                   ブラウザ UI (静的サイト。GitHub Pages で配信)
│   ├── index.html         キャラクター・装備セット・ステータス
│   ├── search.html        装備検索
│   ├── js/                UI・保存・Supabase 連携
│   └── data/              UI が読むデータ (大半は data/ への symlink)
├── data/                  Rust と Web が共有するテーブルデータ (ADR 0002)
├── scripts/               装備データの生成・検証
├── docs/
│   ├── adr/               設計判断の記録
│   ├── knowledge/         FF11 のゲーム仕様と計算式などのナレッジベース
│   └── tech-debt/         既知の負債・意図的に残した非互換
├── supabase/              スキーマ定義
└── .github/workflows/     CI (test.yml / deploy.yml)
```

git 管理外の生成物です。

| ディレクトリ | 中身 | 作られ方 |
|---|---|---|
| `build/` | `items.json` | `scripts/build_data.sh` |
| `web/pkg/` | WASM モジュール | `wasm-pack build` |
| `temp_resources/` | 上流 Lua のキャッシュ | `scripts/build_data.sh` |
| `rust/target/` | Rust のビルド成果物 | `cargo build` |
| `.venv/` | pre-commit | `uv sync` |

## 開発での注意点

### フォーマッティング

clippy を先、fmt を後に実行してください。

```bash
cargo clippy --manifest-path rust/Cargo.toml --lib --tests
```

```bash
cargo fmt --manifest-path rust/Cargo.toml
```

**この順序が必要です。** `cargo clippy --fix` や手直しは整形を崩すことがあり
（derive の分割、行長の変化、コメント位置のずれ）、fmt を先にかけると取りこぼします。

commit すると pre-commit が走り、`.rs` の変更が含まれていれば `cargo fmt --check` が実行されます。
整形されていないと commit が中断されるので、上記を実行して `git add` し直してください。
急ぎで迂回する場合は `git commit --no-verify` が使えます。

clippy は pre-commit に含めていません。ビルドを伴い commit のたびに待たされるためです。
CI では実行しています。

### コミットメッセージ

`[Category] Description` の形式です。

```
[Fix] 装備の属性耐性 (耐火/耐氷 等) を正しく抽出
```

- **"why" を書く**。"what" は diff で分かります
- 本文では、なぜその方法を選んだか・他の案を採らなかった理由を残してください
- よく使われるカテゴリ: `[Feature]` `[Fix]` `[Refactor]` `[Web]` `[Docs]` `[Test]` `[CI]` `[Chore]`

### ADRの運用

アーキテクチャの選択、依存関係の追加、横断的な規約など、**非自明でトレードオフのある決定**は
[docs/adr/](docs/adr/) に記録します。既存の ADR と矛盾する変更を入れる場合は、
先に該当 ADR を読み、必要なら supersede してください。

### ナレッジベースの運用

計算式を実装・修正したときは、根拠（wiki の URL など）と具体的な計算例を
[docs/knowledge/](docs/knowledge/) に残してください。後から式の妥当性を検証できなくなるためです。

### CI/CD

`main` への push で [deploy.yml](.github/workflows/deploy.yml) が走り、GitHub Pages へ
自動デプロイされます。同ワークフローは上流の更新を取り込むため毎日 00:00 (JST) にも実行されます。

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| Rust のビルドが `include_str!` で失敗する / 装備検索が空になる | `scripts/build_data.sh` を実行して `build/items.json` を生成する |
| WASM が読み込めない | `web/pkg/` があるか確認。無ければ `wasm-pack build` を実行する |
| Rust を直したのにブラウザに反映されない | WASM の再ビルドが必要。ブラウザのキャッシュも確認する |
| commit が `cargo fmt --check` で止まる | `cargo fmt --manifest-path rust/Cargo.toml` して `git add` し直す |
| `pre-commit: command not found` | `uv sync` を実行して `.venv/` を作り直す |
| `rust-toolchain.toml` の版が使われない | 環境変数 `RUSTUP_TOOLCHAIN` を確認する。設定されているとファイルより優先される |
