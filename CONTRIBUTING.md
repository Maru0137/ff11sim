# CONTRIBUTING

clone してから変更を push するまでの流れをまとめます。

**コマンドはすべてリポジトリのルートで実行します。** `cd` は不要です。

## 0. 必要なツール

| ツール | 用途 | 備考 |
|---|---|---|
| Rust (stable) | コア実装のビルド・テスト | [rustup](https://rustup.rs/) で導入 |
| wasm-pack | ブラウザ向け WASM のビルド | `cargo install wasm-pack` |
| Python 3 | 装備データ生成スクリプト | 標準ライブラリのみ使用。追加パッケージ不要 |
| pre-commit | commit 前の自動検査 | 次節を参照 |

pre-commit 本体は次のいずれかで導入します。

```bash
uv tool install pre-commit
```

```bash
pipx install pre-commit
```

```bash
brew install pre-commit
```

## 1. 初回セットアップ

```bash
git clone https://github.com/Maru0137/ff11sim.git
```

```bash
cd ff11sim
```

commit 前の検査を有効化します（1 度だけ）。

```bash
pre-commit install
```

**git は clone 時にフックを自動で有効化しません**（任意のコードが実行されてしまうため）。
そのためこの 1 行は各自で実行する必要があります。

装備データを生成します。

```bash
scripts/build_web_data.sh
```

`web/data/items.json` は上流の [Windower/Resources](https://github.com/Windower/Resources) から
生成する派生物で、git では管理していません（[ADR 0003](docs/adr/0003-items-json-generated-in-ci.md)）。
**装備検索・装備セット・後述の JS テストはこれが無いと動きません。**
2 回目以降は、上流の内容が変わっていなければスキップされます。

WASM をビルドします。

```bash
wasm-pack build rust --target web --out-dir ../web/pkg
```

`--out-dir` はクレート (`rust/`) からの相対パスです。生成先は `web/pkg/` になります。

## 2. ブランチを切る

`main` に直接コミットせず、用途を接頭辞にしたブランチを切ります。

```bash
git switch -c feature/skillchain-bonus
```

実績のある接頭辞: `feature/` / `fix/` / `docs/` / `chore/`

## 3. 変更する

| 変更対象 | 主な場所 |
|---|---|
| ステータス・ダメージ計算 | `rust/src/` |
| ブラウザ UI | `web/` |
| 装備データ生成 | `scripts/` |
| FF11 のゲーム仕様メモ | `docs/knowledge/` |

計算式を実装・修正したときは、根拠（wiki の URL など）と具体的な計算例を
`docs/knowledge/` に残してください。後から式の妥当性を検証できなくなるためです。

### 設計判断を伴う場合は ADR を書く

アーキテクチャの選択、依存関係の追加、横断的な規約など、**非自明でトレードオフのある決定**は
[docs/adr/](docs/adr/) に記録します。既存の ADR と矛盾する変更を入れる場合は、
先に該当 ADR を読み、必要なら supersede してください。

## 4. 動かして確かめる

Rust を変更したら WASM を再ビルドします。

```bash
wasm-pack build rust --target web --out-dir ../web/pkg
```

`web/` を静的配信します。

```bash
python3 -m http.server 8000 --directory web
```

http://localhost:8000 を開きます。ポートは 8000 を推奨します
（Supabase のリダイレクト許可リストに登録済みで、ログイン機能を試せるため）。

ログイン機能を試す場合のみ、追加で `web/js/config.js` の用意が必要です。
詳細は [web/README.md](web/README.md) を参照してください。

## 5. テストを通す

Rust のテストです。

```bash
cargo test --manifest-path rust/Cargo.toml
```

JS のテストです（`items.json` の生成が前提）。

```bash
node web/test/equip-stats-extraction.test.js
```

```bash
node web/test/skillchain-bonus.test.js
```

```bash
node web/test/ws-damage-sam-elemental.test.js
```

**PR を出しても自動テストは走りません。** 手元で通してから出してください。

方針は次のとおりです。

- **機能追加**: 対応するテストを先に追加し、それが通ることを完了条件とする
- **バグ修正**: まず既存テストで検出できなかった理由を調べ、バグを再現するテストを
  追加してから直す

## 6. 整形する

```bash
cargo clippy --manifest-path rust/Cargo.toml --lib --tests
```

```bash
cargo fmt --manifest-path rust/Cargo.toml
```

**この順序で実行してください。** `cargo clippy --fix` や手直しは整形を崩すことがあり
（derive の分割、行長の変化、コメント位置のずれ）、fmt を先にかけると取りこぼします。

## 7. コミットする

```bash
git add -A
```

```bash
git commit
```

commit すると pre-commit が走り、`.rs` の変更が含まれていれば `cargo fmt --check` が実行されます。

```
cargo fmt --check........................................................Passed
```

整形されていないと commit が中断されます。その場合は次を実行して `git add` し直してください。

```bash
cargo fmt --manifest-path rust/Cargo.toml
```

急ぎで迂回する場合は次が使えます。

```bash
git commit --no-verify
```

clippy は検査に含めていません。ビルドを伴い commit のたびに待たされるためです。

### コミットメッセージ

`[Category] Description` の形式です。

```
[Fix] 装備の属性耐性 (耐火/耐氷 等) を正しく抽出
```

- **"why" を書く**。"what" は diff で分かります
- 本文では、なぜその方法を選んだか・他の案を採らなかった理由を残してください
- よく使われるカテゴリ: `[Feature]` `[Fix]` `[Refactor]` `[Web]` `[Docs]` `[Test]` `[CI]` `[Chore]`

## 8. push して PR を出す

```bash
git push -u origin feature/skillchain-bonus
```

```bash
gh pr create --base main
```

PR には**何をなぜ変えたか**と、**どう検証したか**（実行したテスト、確認した挙動）を書いてください。

## 9. マージ後

`main` への push で [deploy.yml](.github/workflows/deploy.yml) が走り、GitHub Pages へ自動デプロイされます。
このとき装備データも上流から再生成されます。

なお上流の更新を取り込むため、同ワークフローは毎日 00:00 (JST) にも実行されます。
即座に取り込みたい場合は Actions から手動実行できます。

## トラブルシューティング

| 症状 | 対処 |
|---|---|
| 装備検索が空になる / JS テストが落ちる | `scripts/build_web_data.sh` を実行して `web/data/items.json` を生成する |
| WASM が読み込めない | `web/pkg/` があるか確認。無ければ `wasm-pack build` を実行する |
| Rust を直したのにブラウザに反映されない | WASM の再ビルドが必要。ブラウザのキャッシュも確認する |
| commit が `cargo fmt --check` で止まる | `cargo fmt --manifest-path rust/Cargo.toml` して `git add` し直す |
| `pre-commit: command not found` | pre-commit 本体を導入する（「0. 必要なツール」参照） |
