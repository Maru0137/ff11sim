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
