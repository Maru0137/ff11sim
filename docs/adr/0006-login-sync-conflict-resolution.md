---
status: accepted
date: 2026-08-02
decision-makers: Akira Maruoka
---

# 0006. ログイン時の同期は「一方向アップロード・Supabase 優先・ユーザーごと一度きり」とする

## Context and Problem Statement

[ADR 0005](0005-localstorage-default-supabase-optin.md) により、未ログイン時は
localStorage、ログイン時は Supabase という 2 つの保存先が存在する。ゲストとして
データを作り込んだユーザーがログインすると、それまでのデータが画面から消えたように
見えてしまう。両者をどう突き合わせ、衝突をどう解決するかを決める必要がある。

## Decision Drivers

* ゲスト時に作ったデータがログインで失われたように見えないこと
* 既にクラウドにあるデータを壊さないこと（こちらの方が失うと痛い）
* 実装量が個人開発の範囲に収まること
* 同期処理が複数回走っても壊れないこと

## Considered Options

1. 同期しない（ログインすると別のデータ空間に切り替わる）
2. ログイン時に localStorage → Supabase の一方向アップロードを行う
    - 2.1. 衝突したら Supabase 側を優先し、ローカル側をスキップする
    - 2.2. 衝突したらローカル側を優先し、Supabase 側を上書きする
    - 2.3. 衝突をユーザーに提示して選ばせる
3. 常時双方向同期する（last-write-wins やベクタークロック等）

## Decision Outcome

選択: **2 + 2.1. 一方向アップロード、衝突は Supabase 優先でスキップ**（採用）。
加えて、ユーザーごとに **一度だけ** 実行する。

双方向同期（選択肢 3）は更新時刻の管理と削除の伝播が必要になり、この規模の
アプリケーションには過剰である。衝突解決 UI（2.3）も、実際に衝突が起きる頻度
（同じ名前のキャラクターをローカルとクラウドの両方に持つ場面）に対して割に合わない。
「失って痛いのはクラウド側」という判断から、衝突時はクラウドを守る。

- `web/js/sync.js` が `onAuthChange` を購読し、ログイン検知時に
  `syncLocalToSupabase(user)` を実行する。
- Supabase 側の既存キーを先に取得し、**衝突しないものだけ** を insert する。
  キーは character が `name`、equipset が `character|job|name`。
- 完了後 `localStorage` に `ff11sim_synced_<user.id>` フラグを立て、
  同一ユーザーでは二度と同期しない。別ユーザーでログインすれば再度走る。
- 同期完了時（アップロード 0 件でも）`ff11sim:synced` イベントを発火し、
  画面側が再描画する。
- 多重起動は `_syncing` フラグで抑止する。

### Consequences

* Good: ゲスト時のデータがログイン後も引き継がれ、消えたように見えない。
* Good: 既存のクラウドデータは決して上書きされない。
* Good: 実装が小さく、更新時刻や tombstone を持つ必要がない。
* Bad: 同名で衝突した場合、ローカル側の編集内容は黙って捨てられる。
  ユーザーへの通知は `console.log` のみで、UI 上のフィードバックがない。
* Bad: 同期済みフラグを localStorage に置いているため、別ブラウザ・別プロファイル・
  ストレージクリア後には同じユーザーでも再度同期が走る。
  そのときローカルに古いデータが残っていれば、それが再アップロードされうる
  （衝突しない名前であれば insert される）。
* Bad: 同期は一方向のみ。クラウド側のデータがローカルへ降りてくることはないため、
  ログアウトすると再びローカルのデータだけが見える状態に戻る。
* Bad: 同期が失敗した場合 `console.error` するのみでフラグは立たず、次回ログイン時に
  再試行されるが、部分的に insert 済みの状態から再開することになる（冪等ではない）。
* Neutral: 同期は「移行」であって「継続的な同期」ではないという位置づけ。

### Confirmation

* `web/js/supabase-client.js` の `onAuthStateChange` は、直前のユーザー ID と
  比較して変化があったときだけリスナを呼ぶ。`INITIAL_SESSION` のリプレイによる
  重複起動を防いでいる。
* `web/js/sync.js` の `_syncing` フラグが、同期中の再入を防ぐ。
* `syncFlagKey(user.id)` によるフラグで、同一ユーザーの二重同期を防ぐ。

検証されていないもの:

* 同期の競合解決を検証する自動テストは存在しない。CI は `cargo test` のみを実行する
  （[ADR 0001](0001-rust-wasm-static-site.md)）。
* 同期失敗後の再試行が正しく動くかは確認されていない。

フォローアップ候補: 衝突によりスキップした件数を UI に表示する。少なくとも
「N 件はクラウド側を優先したためアップロードしなかった」ことをユーザーに伝える。

## More Information

* 保存先の切り替え: [ADR 0005](0005-localstorage-default-supabase-optin.md)
* Supabase の鍵管理と RLS: [ADR 0007](0007-supabase-anon-key-ci-injection.md)
