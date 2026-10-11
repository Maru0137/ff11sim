# ff11sim
A simulator of FINAL FANTASY XI

FINAL FANTASY XI のキャラクター・装備・ダメージのシミュレータです。種族・ジョブ・レベル・装備から、
ゲームの実際の計算式でステータスを求めます。

## 使い方

ブラウザで [https://maru0137.github.io/ff11sim/](https://maru0137.github.io/ff11sim/) を開きます。インストールは不要です。

1. 「装備検索」で装備を探す
2. 「装備セット」で装備を組み合わせて登録する
3. 「キャラクター」で種族・ジョブ・レベルと装備セットを選び、ステータスを確かめる

データはブラウザ（localStorage）に保存されます。ログインすると端末間で同期でき、装備セットは URL で共有できます。

## セットアップ

clone してから変更を push するまでの流れは [CONTRIBUTING.md](CONTRIBUTING.md) にまとめています。
最低限、リポジトリのルートで次の 4 つを実行してください。

開発ツール（pre-commit）を取得します。

```bash
uv sync
```

commit 前の検査を有効化します（1 度だけ）。

```bash
uv run pre-commit install
```

装備データを生成します（git 管理外のため clone 直後は存在しません）。

```bash
scripts/build_data.sh
```

WASM をビルドします。

```bash
wasm-pack build rust --target web --out-dir ../web/pkg
```

- 開発の流れ全般: [CONTRIBUTING.md](CONTRIBUTING.md)
- Web フロントエンドの詳細: [web/README.md](web/README.md)
- 設計判断の記録: [docs/internal/adr/](docs/internal/adr/)

## ライセンス

現時点では OSS として公開する方針ではないため、ライセンスは定めていません。
