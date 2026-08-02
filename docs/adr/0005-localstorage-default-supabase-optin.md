---
status: accepted
date: 2026-08-02
decision-makers: Akira Maruoka
---

# 0005. 永続化は localStorage を既定とし、Supabase をオプトインにする（Repository で切り替える）

## Context and Problem Statement

キャラクターと装備セットはユーザーが時間をかけて作るデータであり、端末をまたいで
使いたいという要求がある。一方でログインを必須にすると、試しに触ってみたいだけの
ユーザーを門前払いすることになる。サーバーを持たない構成
（[ADR 0001](0001-rust-wasm-static-site.md)）の下で、どこに保存し、
認証の有無をアプリケーションコードにどう見せるかを決める必要がある。

## Decision Drivers

* ログインなしで全機能が使えること（導入障壁をゼロに保つ）
* ログインすれば端末間でデータを共有できること
* UI コードが認証状態の分岐で汚れないこと
* 個人開発として維持する実装量に収まること

## Considered Options

1. localStorage のみを使う（クラウド保存をあきらめる）
2. Supabase のみを使う（ログインを必須にする）
3. 両方に対応する
    - 3.1. 呼び出し側で認証状態を見て分岐する
    - 3.2. Repository 抽象を挟み、facade の背後に隠す

3.1 と 3.2 は「分岐をどこに置くか」で分かれる。

## Decision Outcome

選択: **3 + 3.2. Repository 抽象を挟み、facade の背後に隠す**（採用）。

未ログインで使えることは譲れず（選択肢 2 を却下）、端末間共有の要求も実在する
（選択肢 1 を却下）。両対応が前提になると、分岐を呼び出し側に散らす 3.1 は
永続化を触るすべての箇所に認証の知識を要求してしまうため、境界を 1 箇所に閉じ込める。

- 呼び出し側は `web/js/storage.js` の `loadCharacters` / `saveCharacters` /
  `loadEquipSets` / `saveEquipSets` のみを使う。認証状態を見てはならない。
- `storage.js` は `getCharacterRepo()` / `getEquipSetRepo()` に委譲するだけの facade とし、
  自身は localStorage を直接触らない。
- 各 repo ファクトリが `getCurrentUser()` の有無で `Local*Repo` / `Supabase*Repo` を返す。
- Repository のインターフェースは `list()` と `save(配列)` の 2 つだけに絞る。
  Supabase 側は `save` のたびに既存一覧との差分を取り、upsert と delete を発行する。
- 識別キーは character が `(user_id, name)`、equipset が
  `(user_id, name, character_name, job)`。Supabase 側の unique 制約と一致させる。
- API はすべて async。呼び出し側は `await` が必須。

### Consequences

* Good: 未ログインでもキャラクター作成・装備セット・計算のすべてが動く。
* Good: UI 側は認証状態を意識せず、同じ関数を呼ぶだけで済む。
* Good: 保存先の追加・変更が repo の追加で済む。
* Bad: 永続化の実装を 2 系統維持し続ける必要がある。片方だけ修正する事故が起こりうる。
* Bad: `save` が「全件を渡して差分を取る」粒度のため、1 件の編集でも全件の upsert が飛ぶ。
  equipset の削除は複合キーのため 1 件ずつ DELETE を発行しており、
  データ量に比例してリクエストが増える。
* Bad: `list()` は Supabase 側でエラーが起きたとき `console.error` して空配列を返す。
  呼び出し側からは「データが 0 件」と区別がつかず、保存操作と組み合わさると
  意図しない削除につながる可能性がある。
* Bad: localStorage 側は容量上限（一般に数 MB）があり、装備セットが増えると
  保存に失敗しうるが、失敗を検知して通知する処理はない。
* Neutral: 並び順は Supabase では `position` 列で保持し、localStorage では配列順で保持する。
* Neutral: 同期の方向と競合解決は本 ADR の範囲外とし、
  [ADR 0006](0006-login-sync-conflict-resolution.md) で決める。

### Confirmation

* `web/js/storage.js` は repo に委譲するのみで、localStorage を直接参照していない
  （実装で確認済み）。
* `web/js/repositories/character-repo.js` / `equipset-repo.js` の
  `getCharacterRepo()` / `getEquipSetRepo()` が `getCurrentUser()` の結果で
  実装を切り替える。

検証されていないもの:

* Repository の自動テストは存在しない。CI で走るのは `cargo test` のみであり
  （[ADR 0001](0001-rust-wasm-static-site.md)）、Web 側の永続化はテスト対象外。
* facade を迂回して localStorage を直接読む箇所が新たに追加されても検出できない。
  現状 `web/js/sync.js` は同期処理の性質上、意図的に localStorage を直接読んでいる。

## More Information

* 全体構成: [ADR 0001](0001-rust-wasm-static-site.md)
* ログイン時のデータ移行: [ADR 0006](0006-login-sync-conflict-resolution.md)
* Supabase の鍵管理と RLS: [ADR 0007](0007-supabase-anon-key-ci-injection.md)
