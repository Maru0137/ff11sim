# FF11 Simulator Web Frontend

FINAL FANTASY XI character, equipment and damage simulator - Web UI

## ローカル開発サーバーの起動方法

ビルドツールチェーンは Vite です (`docs/adr/0011`)。事前に「装備データの生成」
「WebAssembly のビルド」(後述) と `npm ci` を済ませてください。

```bash
npm run dev
```

ブラウザで http://localhost:8000 にアクセス（ファイル変更は自動リロード）。

配信物 (バンドル済み) の動作を確認したい場合:

```bash
npm run build    # dist/ に生成
npm run preview  # dist/ を http://localhost:8000 で配信
```

> Vite を経由しない素の静的配信 (`python3 -m http.server` 等) では
> npm 依存 (supabase-js) の import が解決できず動作しません。

## 装備データの生成（最初に必要）

`build/items.json` は上流 ([Windower/Resources](https://github.com/Windower/Resources)) から
生成する派生物で、git 管理外です (`docs/adr/0003`)。**チェックアウト直後は存在しないため、
まずこれを生成してください。**

```bash
scripts/build_data.sh
```

CI と同じコマンドで、以下を一括で行います。

- 上流 Lua のダウンロード (`temp_resources/` にキャッシュ。2 回目以降は内容が同じならスキップ)
- `build/items.json` の生成 (Rust が `include_str!` で埋め込む)
- 件数の検証
- `build/_build_metadata.json` (ビルドのメタ情報) の出力。`web/public/data/` の symlink 経由で配信される

> **`cargo build` / `cargo test` より先に実行する必要があります。**
> 装備データは `include_str!` で Rust のバイナリに埋め込まれるため (`docs/adr/0009`)、
> `build/items.json` が無いとコンパイルが通りません。
> `scripts/scrape_augments.py` も装備名の逆引きに使います。
>
> `web/` の外に置くのは、ブラウザがこれを読まないためです (`docs/adr/0010`)。
> `web/` に置くと Pages に配信され、WASM に埋め込んだものと合わせて
> 同じデータを二重に配ることになります。

## WebAssemblyのビルド

初回起動時やRustコードを変更した場合、WASMを再ビルドする必要があります。
上記の装備データ生成を先に済ませておいてください。

```bash
cd rust

# wasm-packインストール（初回のみ）
cargo install wasm-pack

# WASMビルド
wasm-pack build --target web --out-dir ../web/pkg
```

ビルド成功後、`web/pkg/`ディレクトリにWASMファイルが生成されます。

## Supabase 連携 (ユーザー登録 + クラウド保存)

ログインすると、キャラクターと装備セットを Supabase (Postgres) に保存できる。
未ログイン時は今まで通り localStorage で動作するため、Supabase セットアップは
任意機能。ローカル開発時に認証機能を試したい場合のみ以下を行う。

### 1. Supabase プロジェクト準備 (Web UI)

1. https://supabase.com/dashboard で新規プロジェクト作成
2. Google Cloud Console で OAuth 2.0 Client ID を作成
3. Supabase の Authentication → Providers → Google に Client ID/Secret を登録
4. Authentication → URL Configuration で Redirect URL を追加:
   - `http://localhost:8000/**` (開発)
   - `http://localhost:8888/**` (開発)
   - `https://maru0137.github.io/ff11sim/**` (本番)
5. SQL Editor で `supabase/schema.sql` を貼り付けて実行
6. Project Settings → API から `URL` と `anon` key を取得

### 2. ローカル設定ファイル

```bash
cp web/js/config.example.js web/js/config.js
# config.js を編集して SUPABASE_URL / SUPABASE_ANON_KEY に実値を設定
```

`web/js/config.js` は `.gitignore` 済み (commit されない)。

### 3. 本番 (GitHub Pages) へのデプロイ

GitHub の Repository Settings → Secrets and variables → Actions に登録:
- `SUPABASE_URL`
- `SUPABASE_ANON_KEY`

CI (`.github/workflows/deploy.yml`) が deploy 時に `web/js/config.js` を生成する。

> anon key は公開して問題ない (Supabase RLS でユーザー間データが分離される)。
> 万一漏洩しても他ユーザーのデータには触れない。ただし key ローテーション時の
> 履歴汚染を避けるため、リポジトリには直接 commit しない運用とする。

## ディレクトリ構成

```
web/
├── index.html          # メインページ（Character/Equipment Set/Status）
├── search.html         # 装備検索ページ
├── js/                 # UI・永続化・認証（装備ロジックは Rust 側にある）
├── public/
│   └── data/           # 実行時 fetch されるデータ (大半は data/ への symlink)
│       └── augments.json   # オーグメント定義
└── pkg/                # WASMモジュール（ビルド後生成）
    ├── ff11sim.js
    ├── ff11sim_bg.wasm
    └── ...
```

配信物は `npm run build` が生成する `dist/` で、CI がこれを GitHub Pages に
アップロードします (`.github/workflows/deploy.yml`)。

## 機能

- **Character Management**: キャラクター作成（種族、ジョブレベル、メリットポイント）
- **Equipment Set**: 装備セット管理（16スロット、ジョブ別グループ化）
- **Status Calculation**: リアルタイムステータス計算（Base + Equipment）
- **Item Search**: 全装備アイテムの高速検索・フィルタリング

## トラブルシューティング

### WASMが読み込めない

- `web/pkg/`ディレクトリが存在するか確認
- Rustコードをビルドしたか確認（上記「WebAssemblyのビルド」参照）

### 装備が検索できない・装備セットのステータスが 0 になる

装備データは WASM に埋め込まれている (`docs/adr/0009`) ため、ブラウザは
`items.json` を読まない。データが古い/無い場合は Rust の再ビルドが必要。

```bash
scripts/build_data.sh                                   # データ生成
cd rust && wasm-pack build --target web --out-dir ../web/pkg  # WASM 再ビルド
```

### 公開サイトの装備データを確認したい

`items.json` は配信物に含まれない (`docs/adr/0010`) ため、URL では取得できない。
CI が実行ごとに artifact として残しているので、そこから取得する。

1. 公開サイトの `data/_build_metadata.json` を開き、`commit` を確認する
   - 例: https://maru0137.github.io/ff11sim/data/_build_metadata.json
2. GitHub の Actions から、その commit の実行を開く
3. `items-json` という artifact をダウンロードする（保持期間 30 日）

ローカルで同じものを作りたい場合は `scripts/build_data.sh` を実行すると
`build/items.json` に生成される。`_build_metadata.json` の
`upstream_sources.windower_resources.blobs` が一致していれば内容も同じになる。

### ステータスが表示されない

- キャラクターとジョブを選択しているか確認
- ブラウザのコンソールでJavaScriptエラーが出ていないか確認
