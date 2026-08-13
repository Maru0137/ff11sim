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
- 例外として **UI プリファレンス層** を認める（[ADR 0015](0015-property-sets.md) で追記）:
  消えても実害のない端末ローカルの UI 状態（サイドバー開閉 `ff11sim_nav_collapsed`）は
  facade を通さず localStorage に直接保存し、Supabase には同期しない。頻繁な書き込みが
  全件 upsert を誘発しないようにするためで、ユーザーデータ本体には適用しない。
  （プロパティセット選択記憶は当初この層だったが、[ADR 0015](0015-property-sets.md) の
  2026-08-13 改訂で装備セットレコードへ移動した。）

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
  現状 `web/js/sync.js` は同期処理の性質上、また UI プリファレンス層
  （`ff11sim_nav_collapsed`）は上記の例外規定により、
  意図的に localStorage を直接読み書きしている。

## Pros and Cons of the Options

### 1. localStorage のみを使う

* Good: 実装が最も単純。認証もネットワークも要らない。
* Good: [ADR 0001](0001-rust-wasm-static-site.md) のサーバーレス志向と完全に一致する。
* Bad: 端末間でデータを共有できない。ブラウザのデータを消すと失われる。
* Bad: 装備セット共有 ([ADR 0008](0008-equipset-sharing.md)) のような機能を後から載せられない。

### 2. Supabase のみを使う（ログインを必須にする）

* Good: 保存先が 1 つで、実装とテストの対象が半分になる。
* Good: 端末間共有が自然に得られる。
* Bad: 試しに触ってみたいだけのユーザーにログインを強制する。導入障壁として大きい。
* Bad: Supabase が落ちるとアプリが一切使えなくなる。
* Bad: 未ログインでも成立すべき装備検索ページの位置づけが破綻する。

### 3. 両方に対応する（採用）

* Good: 未ログインで全機能が使え、ログインすれば端末間で共有できる。
* Good: Supabase に障害があってもゲストとしては使い続けられる。
* Bad: 永続化の実装を 2 系統維持し続ける必要がある。
* Bad: 2 つの保存先の間でデータをどう移すかという別の問題が生じる
  （[ADR 0006](0006-login-sync-conflict-resolution.md)）。

#### 3.1. 呼び出し側で認証状態を見て分岐する

* Good: 抽象化の層が増えず、処理の流れがその場で読める。
* Bad: 永続化を触るすべての箇所に認証の知識が要る。追加のたびに分岐を書き忘れる余地が残る。
* Bad: 保存先を増やすときに全呼び出し箇所を直すことになる。

#### 3.2. Repository 抽象を挟み、facade の背後に隠す（採用）

* Good: UI 側は認証状態を意識せず、同じ関数を呼ぶだけで済む。
* Good: 分岐が repo ファクトリの 1 箇所に閉じ、保存先の追加が repo の追加で済む。
* Bad: API がすべて async になり、既存の同期呼び出しを `await` に書き換える必要があった。
* Bad: インターフェースを `list` / `save` の 2 つに絞ったため、
  1 件だけ更新したい場合も全件を渡すことになる。

## More Information

* 全体構成: [ADR 0001](0001-rust-wasm-static-site.md)
* ログイン時のデータ移行: [ADR 0006](0006-login-sync-conflict-resolution.md)
* Supabase の鍵管理と RLS: [ADR 0007](0007-supabase-anon-key-ci-injection.md)
