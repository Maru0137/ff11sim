# ff11sim
A simulator of FINAL FANTASY XI

## 開発環境のセットアップ

[pre-commit](https://pre-commit.com/) を使い、commit 前に Rust コードの整形を検査します。
クローン後に 1 度だけ、次を実行してください。

```bash
# pre-commit 本体 (いずれか 1 つ)
uv tool install pre-commit    # または: pipx install pre-commit / brew install pre-commit

# このリポジトリでフックを有効化
pre-commit install
```

整形されていないと commit が中断されるので、`cd rust && cargo fmt` して `git add` し直してください。
急ぎで迂回したい場合は `git commit --no-verify` が使えます。

git は clone 時にフックを自動で有効化しません（任意のコードが実行されてしまうため）。
そのため `pre-commit install` は各自で実行する必要があります。

- Web フロントエンドと WASM のビルド: [web/README.md](web/README.md)
- 設計判断の記録: [docs/adr/](docs/adr/)
