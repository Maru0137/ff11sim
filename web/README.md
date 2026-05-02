# FF11 Simulator Web Frontend

FINAL FANTASY XI character, equipment and damage simulator - Web UI

## ローカル開発サーバーの起動方法

このプロジェクトは静的HTML + WebAssemblyで動作するため、シンプルなHTTPサーバーで実行できます。

### 方法1: Python（推奨）

最も手軽な方法です。Python 3がインストール済みであれば追加のインストール不要。

```bash
cd web
python3 -m http.server 8000
```

ブラウザで http://localhost:8000 にアクセス

### 方法2: Node.js http-server

```bash
# 初回のみインストール
npm install -g http-server

# サーバー起動
cd web
http-server -p 8000
```

ブラウザで http://localhost:8000 にアクセス

### 方法3: Rust basic-http-server

```bash
# 初回のみインストール
cargo install basic-http-server

# サーバー起動
cd web
basic-http-server
```

ブラウザで http://localhost:4000 にアクセス（デフォルトポート）

### 方法4: VSCode Live Server

1. VSCode拡張機能「Live Server」をインストール
2. `web/index.html`を開く
3. 右下の「Go Live」をクリック

## WebAssemblyのビルド

初回起動時やRustコードを変更した場合、WASMを再ビルドする必要があります。

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
├── js/
│   ├── item-search.js  # アイテム検索エンジン
│   └── equip-stats.js  # 装備ステータス抽出
├── data/
│   └── items.json      # 装備データベース（14,921アイテム）
└── pkg/                # WASMモジュール（ビルド後生成）
    ├── ff11sim.js
    ├── ff11sim_bg.wasm
    └── ...
```

## 機能

- **Character Management**: キャラクター作成（種族、ジョブレベル、メリットポイント）
- **Equipment Set**: 装備セット管理（16スロット、ジョブ別グループ化）
- **Status Calculation**: リアルタイムステータス計算（Base + Equipment）
- **Item Search**: 14,921アイテムの高速検索・フィルタリング

## トラブルシューティング

### WASMが読み込めない

- `web/pkg/`ディレクトリが存在するか確認
- Rustコードをビルドしたか確認（上記「WebAssemblyのビルド」参照）

### items.jsonが読み込めない

- HTTPサーバーが`web/`ディレクトリで起動しているか確認
- ブラウザのコンソールでCORSエラーが出ていないか確認

### ステータスが表示されない

- キャラクターとジョブを選択しているか確認
- ブラウザのコンソールでJavaScriptエラーが出ていないか確認
