# ff11sim
A simulator of FINAL FANTASY XI

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
- 設計判断の記録: [docs/adr/](docs/adr/)
