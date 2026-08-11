# ff11sim
A simulator of FINAL FANTASY XI

## セットアップ

clone してから変更を push するまでの流れは [CONTRIBUTING.md](CONTRIBUTING.md) にまとめています。
最低限、次の 3 つが必要です。

```bash
pre-commit install                 # commit 前の検査を有効化 (1 度だけ)
scripts/build_web_data.sh          # 装備データを生成 (git 管理外)
cd rust && wasm-pack build --target web --out-dir ../web/pkg
```

- 開発の流れ全般: [CONTRIBUTING.md](CONTRIBUTING.md)
- Web フロントエンドの詳細: [web/README.md](web/README.md)
- 設計判断の記録: [docs/adr/](docs/adr/)
